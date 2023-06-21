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

use crate::{
    Annotations, ConfigYaml, Error, Labels, ObjectName, ObjectNamespace, Result, SelectorLabels,
    Torrc, APP_KUBERNETES_IO_MANAGED_BY, APP_KUBERNETES_IO_NAME,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionBalance",
    namespaced,
    status = "OnionBalanceStatus",
    version = "v1"
)]
pub struct OnionBalanceSpec {
    pub onion_key: OnionBalanceSpecOnionKey,

    pub onion_services: Vec<OnionBalanceSpecOnionService>,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionKey {
    pub name: String,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionService {
    pub onion_key: OnionBalanceSpecOnionServiceOnionKey,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionServiceOnionKey {
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionBalanceStatus {}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    OnionBalance::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {
    pub onion_balance_image: ImageConfig,
    pub tor_image: ImageConfig,
}

pub struct ImageConfig {
    pub pull_policy: String,
    pub uri: String,
}

/*
 * ============================================================================
 * Controller
 * ============================================================================
 */
#[allow(clippy::missing_panics_doc)]
pub async fn run_controller(config: Config) {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<OnionBalance>::all(client.clone());

    let context = Arc::new(Context { client, config });

    Controller::new(onion_services, WatcherConfig::default())
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
const APP_KUBERNETES_IO_COMPONENT: &str = "onion-balance";

/*
 * ============================================================================
 * Types
 * ============================================================================
 */
struct Name(String);

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
async fn reconciler(object: Arc<OnionBalance>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let object_name = get_object_name(&object)?;
    let object_namespace = get_object_namespace(&object)?;

    let torrc = generate_torrc(&object);
    let config_yaml = generate_config_yaml(&object);

    let annotations = generate_annotations(&torrc, &config_yaml);
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
                &config_yaml,
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

fn get_object_name(object: &OnionBalance) -> Result<ObjectName> {
    Ok(ObjectName(
        object
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
            .as_str(),
    ))
}

fn get_object_namespace(object: &OnionBalance) -> Result<ObjectNamespace> {
    Ok(ObjectNamespace(
        object
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?
            .as_str(),
    ))
}

fn generate_annotations(torrc: &Torrc, config_yaml: &ConfigYaml) -> Annotations {
    let mut sha = Sha256::new();
    sha.update(&torrc.0);
    let torrc_hash = format!("sha256:{:x}", sha.finalize());
    let mut sha = Sha256::new();
    sha.update(&config_yaml.0);
    let config_hash: String = format!("sha256:{:x}", sha.finalize());
    Annotations(BTreeMap::from([
        ("tor.agabani.co.uk/torrc-hash".into(), torrc_hash),
        ("tor.agabani.co.uk/config-hash".into(), config_hash),
    ]))
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

#[allow(unused_variables)]
fn generate_torrc(object: &OnionBalance) -> Torrc {
    let torrc: Vec<&str> = vec!["SocksPort 9050", "ControlPort 127.0.0.1:6666"];
    Torrc(torrc.join("\n"))
}

fn generate_config_yaml(object: &OnionBalance) -> ConfigYaml {
    let config_yaml = vec![
        "services:".into(),
        "- instances:".into(),
        object
            .spec
            .onion_services
            .iter()
            .map(|onion_service| {
                format!(
                    "  - address: {}\n    name: {}",
                    onion_service.onion_key.hostname, onion_service.onion_key.hostname
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        "  key: /var/lib/tor/hidden_service/hs_ed25519_secret_key".into(),
    ];
    ConfigYaml(config_yaml.join("\n"))
}

fn generate_owned_config_map(
    object: &OnionBalance,
    name: &Name,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    config_yaml: &ConfigYaml,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(name.0.clone()),
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("torrc".into(), torrc.0.clone()),
            ("config.yaml".into(), config_yaml.0.clone()),
        ])),
        ..Default::default()
    }
}

fn generate_owned_deployment(
    object: &OnionBalance,
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
                    containers: vec![
                        Container {
                            args: Some(vec![
                                "-c".into(),
                                vec![
                                    "mkdir -p /var/lib/tor/hidden_service",
                                    "chmod 400 /var/lib/tor/hidden_service",
                                    "cp /etc/secrets/* /var/lib/tor/hidden_service",
                                    "onionbalance -v info -c /usr/local/etc/onionbalance/config.yaml -p 6666",
                                ]
                                .join(" && "),
                            ]),
                            command: Some(vec!["/bin/bash".into()]),
                            image: Some(config.onion_balance_image.uri.clone()),
                            image_pull_policy: Some(config.onion_balance_image.pull_policy.clone()),
                            name: "onionbalance".into(),
                            volume_mounts: Some(vec![
                                VolumeMount {
                                    mount_path: "/etc/secrets".into(),
                                    name: "etc-secrets".into(),
                                    read_only: Some(true),
                                    ..Default::default()
                                },
                                VolumeMount {
                                    mount_path: "/usr/local/etc/onionbalance".into(),
                                    name: "usr-local-etc-onionbalance".into(),
                                    read_only: Some(true),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        },
                        Container {
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
                        },
                    ],
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
                                secret_name: Some(object.spec.onion_key.name.clone()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "usr-local-etc-onionbalance".into(),
                            config_map: Some(ConfigMapVolumeSource {
                                default_mode: Some(0o400),
                                items: Some(vec![KeyToPath {
                                    key: "config.yaml".into(),
                                    mode: Some(0o400),
                                    path: "config.yaml".into(),
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
fn error_policy(object: Arc<OnionBalance>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
