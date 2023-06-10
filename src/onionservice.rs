use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, ExecAction, KeyToPath, PodSpec,
            PodTemplateSpec, Probe, SecretVolumeSource, Volume, VolumeMount,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Error, Result};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    status = "OnionServiceStatus",
    version = "v1"
)]
pub struct OnionServiceSpec {
    pub hidden_service_ports: Vec<OnionServiceSpecHiddenServicePort>,

    pub secret_name: String,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceSpecHiddenServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceStatus {}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    OnionService::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {
    pub tor_image: ImageConfig,
}

pub struct ImageConfig {
    pub pull_policy: String,
    pub uri: String,
}

/*
 * ============================================================================
 * Run
 * ============================================================================
 */
#[allow(clippy::missing_panics_doc)]
pub async fn run_controller(config: Config) {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<OnionService>::all(client.clone());
    let config_maps = Api::<ConfigMap>::all(client.clone());
    let deployments = Api::<Deployment>::all(client.clone());

    let context = Arc::new(Context { client, config });

    Controller::new(onion_services, WatcherConfig::default())
        .owns(config_maps, WatcherConfig::default())
        .owns(deployments, WatcherConfig::default())
        .shutdown_on_signal()
        .run(reconciler, error_policy, context)
        .for_each(|_| async {})
        .await;
}

/*
 * ============================================================================
 * Constants
 * ============================================================================
 */
const APP_KUBERNETES_IO_COMPONENT: &str = "onion-service";
const APP_KUBERNETES_IO_NAME: &str = "tor";
const APP_KUBERNETES_IO_MANAGED_BY: &str = "tor-operator";

/*
 * ============================================================================
 * Types
 * ============================================================================
 */
struct Annotations(BTreeMap<String, String>);
struct Labels(BTreeMap<String, String>);
struct ObjectName<'a>(&'a str);
struct ObjectNamespace<'a>(&'a str);
struct Name(String);
struct SelectorLabels(BTreeMap<String, String>);
struct Torrc(String);

/*
 * ============================================================================
 * Context
 * ============================================================================
 */
struct Context {
    client: Client,
    config: Config,
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let object_name = get_object_name(&object)?;
    let object_namespace = get_object_namespace(&object)?;

    let torrc = generate_torrc(&object);

    let annotations = generate_annotations(&torrc);
    let labels = generate_labels(&object_name);
    let name = generate_name(&object_name);
    let selector_labels = generate_selector_labels(&object_name);

    let config_map: ConfigMap = Api::namespaced(ctx.client.clone(), object_namespace.0)
        .patch(
            &name.0,
            &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY).force(),
            &Patch::Apply(&generate_owned_config_map(
                &object,
                &name,
                &annotations,
                &labels,
                &torrc,
            )),
        )
        .await
        .map_err(Error::Kube)?;

    let _deployment: Deployment = Api::namespaced(ctx.client.clone(), object_namespace.0)
        .patch(
            &name.0,
            &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY).force(),
            &Patch::Apply(&generate_owned_deployment(
                &object,
                &ctx.config,
                &name,
                &annotations,
                &labels,
                &selector_labels,
                &config_map,
            )?),
        )
        .await
        .map_err(Error::Kube)?;

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn get_object_name(object: &OnionService) -> Result<ObjectName> {
    Ok(ObjectName(
        object
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
            .as_str(),
    ))
}

fn get_object_namespace(object: &OnionService) -> Result<ObjectNamespace> {
    Ok(ObjectNamespace(
        object
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?
            .as_str(),
    ))
}

fn generate_annotations(torrc: &Torrc) -> Annotations {
    let mut sha = Sha256::new();
    sha.update(&torrc.0);
    let hash = format!("sha256:{:x}", sha.finalize());
    Annotations(BTreeMap::from([(
        "tor.agabani.co.uk/torrc-hash".into(),
        hash,
    )]))
}

fn generate_labels(object_name: &ObjectName) -> Labels {
    Labels(BTreeMap::from([
        (
            "app.kubernetes.io/component".into(),
            APP_KUBERNETES_IO_COMPONENT.into(),
        ),
        ("app.kubernetes.io/instance".into(), object_name.0.into()),
        (
            "app.kubernetes.io/managed-by".into(),
            APP_KUBERNETES_IO_MANAGED_BY.into(),
        ),
        (
            "app.kubernetes.io/name".into(),
            APP_KUBERNETES_IO_NAME.into(),
        ),
    ]))
}

fn generate_name(object_name: &ObjectName) -> Name {
    Name(format!(
        "{APP_KUBERNETES_IO_NAME}-{APP_KUBERNETES_IO_COMPONENT}-{}",
        object_name.0
    ))
}

fn generate_selector_labels(object_name: &ObjectName) -> SelectorLabels {
    SelectorLabels(BTreeMap::from([
        (
            "app.kubernetes.io/component".into(),
            APP_KUBERNETES_IO_COMPONENT.into(),
        ),
        ("app.kubernetes.io/instance".into(), object_name.0.into()),
        (
            "app.kubernetes.io/name".into(),
            APP_KUBERNETES_IO_NAME.into(),
        ),
    ]))
}

fn generate_torrc(object: &OnionService) -> Torrc {
    let mut torrc = vec!["HiddenServiceDir /var/lib/tor/hidden_service".into()];
    for port in &object.spec.hidden_service_ports {
        torrc.push(format!(
            "HiddenServicePort {} {}",
            port.virtport, port.target
        ));
    }
    Torrc(torrc.join("\n"))
}

fn generate_owned_config_map(
    object: &OnionService,
    name: &Name,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(name.0.clone()),
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(BTreeMap::from([("torrc".into(), torrc.0.clone())])),
        ..Default::default()
    }
}

fn generate_owned_deployment(
    object: &OnionService,
    config: &Config,
    name: &Name,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    config_map: &ConfigMap,
) -> Result<Deployment> {
    Ok(Deployment {
        metadata: ObjectMeta {
            name: Some(name.0.clone()),
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(selector_labels.0.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    annotations: Some(annotations.0.clone()),
                    labels: Some(labels.0.clone()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        args: Some(vec![
                            "-c".into(),
                            vec![
                                "mkdir -p /var/lib/tor/hidden_service",
                                "chmod 400 /var/lib/tor/hidden_service",
                                "cp /etc/secrets/* /var/lib/tor/hidden_service",
                                "tor",
                            ]
                            .join(" && "),
                        ]),
                        command: Some(vec!["/bin/bash".into()]),
                        image: Some(config.tor_image.uri.clone()),
                        image_pull_policy: Some(config.tor_image.pull_policy.clone()),
                        liveness_probe: Some(Probe {
                            exec: Some(ExecAction {
                                command: Some(vec![
                                    "/bin/bash".to_string(),
                                    "-c".to_string(),
                                    "echo > /dev/tcp/127.0.0.1/9050".to_string(),
                                ]),
                            }),
                            failure_threshold: Some(3),
                            period_seconds: Some(10),
                            success_threshold: Some(1),
                            timeout_seconds: Some(1),
                            ..Default::default()
                        }),
                        name: "tor".into(),
                        readiness_probe: Some(Probe {
                            exec: Some(ExecAction {
                                command: Some(vec![
                                    "/bin/bash".to_string(),
                                    "-c".to_string(),
                                    "echo > /dev/tcp/127.0.0.1/9050".to_string(),
                                ]),
                            }),
                            failure_threshold: Some(3),
                            period_seconds: Some(10),
                            success_threshold: Some(1),
                            timeout_seconds: Some(1),
                            ..Default::default()
                        }),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                mount_path: "/etc/secrets".into(),
                                name: "etc-secrets".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                            VolumeMount {
                                mount_path: "/usr/local/etc/tor".into(),
                                name: "usr-local-etc-tor".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }],

                    volumes: Some(vec![
                        Volume {
                            name: "etc-secrets".into(),
                            secret: Some(SecretVolumeSource {
                                default_mode: Some(0o400),
                                items: Some(vec![
                                    KeyToPath {
                                        key: "hostname".into(),
                                        mode: Some(0o400),
                                        path: "hostname".into(),
                                    },
                                    KeyToPath {
                                        key: "hs_ed25519_public_key".into(),
                                        mode: Some(0o400),
                                        path: "hs_ed25519_public_key".into(),
                                    },
                                    KeyToPath {
                                        key: "hs_ed25519_secret_key".into(),
                                        mode: Some(0o400),
                                        path: "hs_ed25519_secret_key".into(),
                                    },
                                ]),
                                optional: Some(false),
                                secret_name: Some(object.spec.secret_name.clone()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "usr-local-etc-tor".into(),
                            config_map: Some(ConfigMapVolumeSource {
                                default_mode: Some(0o400),
                                items: Some(vec![KeyToPath {
                                    key: "torrc".into(),
                                    mode: Some(0o400),
                                    path: "torrc".into(),
                                }]),
                                name: Some(
                                    config_map
                                        .metadata
                                        .name
                                        .as_ref()
                                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
                                        .clone(),
                                ),
                                optional: Some(false),
                            }),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    })
}

/*
 * ============================================================================
 * Error Policy
 * ============================================================================
 */
#[allow(clippy::needless_pass_by_value, unused_variables)]
#[tracing::instrument(skip(object, ctx))]
fn error_policy(object: Arc<OnionService>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
