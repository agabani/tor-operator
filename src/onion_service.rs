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
    api::{DeleteParams, ListParams, Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    onion_key::OnionKey, Annotations, Error, Labels, OBConfig, ObjectName, ObjectNamespace, Result,
    SelectorLabels, Torrc, APP_KUBERNETES_IO_COMPONENT_KEY, APP_KUBERNETES_IO_INSTANCE_KEY,
    APP_KUBERNETES_IO_MANAGED_BY_KEY, APP_KUBERNETES_IO_MANAGED_BY_VALUE,
    APP_KUBERNETES_IO_NAME_KEY, APP_KUBERNETES_IO_NAME_VALUE, TOR_AGABANI_CO_UK_OWNED_BY_KEY,
    TOR_AGABANI_CO_UK_TORRC_HASH_KEY,
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
    kind = "OnionService",
    namespaced,
    status = "OnionServiceStatus",
    version = "v1"
)]
pub struct OnionServiceSpec {
    pub config_map: OnionServiceSpecConfigMap,

    pub deployment: OnionServiceSpecDeployment,

    pub onion_balance: Option<OnionServiceSpecOnionBalance>,

    pub onion_key: OnionServiceSpecOnionKey,

    pub ports: Vec<OnionServiceSpecHiddenServicePort>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecConfigMap {
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeployment {
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalance {
    pub onion_key: OnionServiceSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalanceOnionKey {
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionKey {
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecHiddenServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceStatus {}

impl OnionService {
    #[must_use]
    pub fn config_map_name(&self) -> &str {
        &self.spec.config_map.name
    }

    #[must_use]
    pub fn deployment_name(&self) -> &str {
        &self.spec.deployment.name
    }

    #[must_use]
    pub fn onion_balanced(&self) -> bool {
        self.spec.onion_balance.is_some()
    }

    #[must_use]
    pub fn onion_balance_onion_key_hostname(&self) -> Option<&str> {
        self.spec
            .onion_balance
            .as_ref()
            .map(|onion_balance| onion_balance.onion_key.hostname.as_str())
    }

    #[must_use]
    pub fn onion_key_name(&self) -> &str {
        &self.spec.onion_key.name
    }

    #[must_use]
    pub fn ports(&self) -> &[OnionServiceSpecHiddenServicePort] {
        &self.spec.ports
    }
}

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
 * Controller
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
const APP_KUBERNETES_IO_COMPONENT_VALUE: &str = "onion-service";

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

    let ob_config = generate_ob_config(&object);
    let torrc = generate_torrc(&object);

    let annotations = generate_annotations(&torrc);
    let labels = generate_labels(&object, &object_name);
    let selector_labels = generate_selector_labels(&object_name);

    let onion_key = reconcile_onion_key(&object, &ctx, &object_namespace).await?;

    let Some(onion_key) = onion_key else {
        tracing::info!("status: waiting for onion key hostname.");
        return Ok(Action::requeue(Duration::from_secs(5)));
    };

    reconcile_config_map(
        &object,
        &ctx,
        &object_namespace,
        &annotations,
        &labels,
        &torrc,
        &ob_config,
    )
    .await?;

    reconcile_deployment(
        &object,
        &ctx,
        &object_namespace,
        &annotations,
        &labels,
        &selector_labels,
        &onion_key,
    )
    .await?;

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
}

async fn reconcile_onion_key(
    object: &OnionService,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
) -> Result<Option<OnionKey>> {
    let onion_keys = Api::<OnionKey>::namespaced(ctx.client.clone(), object_namespace.0);

    let onion_key = onion_keys
        .get(object.onion_key_name())
        .await
        .map_err(Error::Kube)?;

    if onion_key.hostname().is_some() {
        Ok(Some(onion_key))
    } else {
        Ok(None)
    }
}

async fn reconcile_config_map(
    object: &OnionService,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    ob_config: &Option<OBConfig>,
) -> Result<()> {
    let config_maps = Api::<ConfigMap>::namespaced(ctx.client.clone(), object_namespace.0);

    // creation
    let config_map = generate_owned_config_map(object, annotations, labels, ob_config, torrc);

    config_maps
        .patch(
            config_map
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
            &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force(),
            &Patch::Apply(&config_map),
        )
        .await
        .map_err(Error::Kube)?;

    // deletion
    let owned_config_maps = config_maps
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    for config_map in &owned_config_maps {
        let config_map_name = config_map
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?;

        if config_map_name != object.config_map_name() {
            config_maps
                .delete(config_map_name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_deployment(
    object: &OnionService,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Result<()> {
    let deployments = Api::<Deployment>::namespaced(ctx.client.clone(), object_namespace.0);

    let deployment = generate_owned_deployment(
        object,
        &ctx.config,
        annotations,
        labels,
        selector_labels,
        onion_key,
    );

    deployments
        .patch(
            deployment
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
            &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force(),
            &Patch::Apply(&deployment),
        )
        .await
        .map_err(Error::Kube)?;

    // deletion
    let owned_deployments = deployments
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    for deployment in &owned_deployments {
        let deployment_name = deployment
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?;

        if deployment_name != object.deployment_name() {
            deployments
                .delete(deployment_name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
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
    let torrc_hash = format!("sha256:{:x}", sha.finalize());
    Annotations(BTreeMap::from([(
        TOR_AGABANI_CO_UK_TORRC_HASH_KEY.into(),
        torrc_hash,
    )]))
}

fn generate_labels(object: &OnionService, object_name: &ObjectName) -> Labels {
    Labels(BTreeMap::from([
        (
            APP_KUBERNETES_IO_COMPONENT_KEY.into(),
            APP_KUBERNETES_IO_COMPONENT_VALUE.into(),
        ),
        (APP_KUBERNETES_IO_INSTANCE_KEY.into(), object_name.0.into()),
        (
            APP_KUBERNETES_IO_MANAGED_BY_KEY.into(),
            APP_KUBERNETES_IO_MANAGED_BY_VALUE.into(),
        ),
        (
            APP_KUBERNETES_IO_NAME_KEY.into(),
            APP_KUBERNETES_IO_NAME_VALUE.into(),
        ),
        (
            TOR_AGABANI_CO_UK_OWNED_BY_KEY.into(),
            object.metadata.uid.clone().unwrap(),
        ),
    ]))
}

fn generate_selector_labels(object_name: &ObjectName) -> SelectorLabels {
    SelectorLabels(BTreeMap::from([
        (
            APP_KUBERNETES_IO_COMPONENT_KEY.into(),
            APP_KUBERNETES_IO_COMPONENT_VALUE.into(),
        ),
        (APP_KUBERNETES_IO_INSTANCE_KEY.into(), object_name.0.into()),
        (
            APP_KUBERNETES_IO_NAME_KEY.into(),
            APP_KUBERNETES_IO_NAME_VALUE.into(),
        ),
    ]))
}

fn generate_ob_config(object: &OnionService) -> Option<OBConfig> {
    object
        .spec
        .onion_balance
        .as_ref()
        .map(|f| OBConfig(format!("MasterOnionAddress {}", f.onion_key.hostname)))
}

fn generate_torrc(object: &OnionService) -> Torrc {
    let mut torrc = vec!["HiddenServiceDir /var/lib/tor/hidden_service".into()];
    if object.onion_balanced() {
        torrc.push("HiddenServiceOnionbalanceInstance 1".into());
    }
    for port in object.ports() {
        torrc.push(format!(
            "HiddenServicePort {} {}",
            port.virtport, port.target
        ));
    }
    Torrc(torrc.join("\n"))
}

fn generate_owned_config_map(
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    ob_config: &Option<OBConfig>,
    torrc: &Torrc,
) -> ConfigMap {
    let mut x = BTreeMap::from([("torrc".into(), torrc.0.clone())]);
    if let Some(ob_config) = ob_config {
        x.insert("ob_config".into(), ob_config.0.clone());
    }
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(object.config_map_name().to_string()),
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(x),
        ..Default::default()
    }
}

fn generate_owned_deployment(
    object: &OnionService,
    config: &Config,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Deployment {
    Deployment {
        metadata: ObjectMeta {
            name: Some(object.deployment_name().to_string()),
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
                            {
                                let mut commands = vec![
                                    "mkdir -p /var/lib/tor/hidden_service",
                                    "chmod 400 /var/lib/tor/hidden_service",
                                    "cp /etc/secrets/* /var/lib/tor/hidden_service",
                                ];

                                if object.onion_balanced() {
                                    commands.push("cp /etc/configs/ob_config /var/lib/tor/hidden_service/ob_config");
                                }

                                commands.push("mkdir -p /usr/local/etc/tor");
                                commands.push("cp /etc/configs/torrc /usr/local/etc/tor/torrc");
                                commands.push("tor");
                                commands
                            }
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
                                mount_path: "/etc/configs".into(),
                                name: "etc-configs".into(),
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
                                secret_name: Some(onion_key.secret_name().to_string()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "etc-configs".into(),
                            config_map: Some(ConfigMapVolumeSource {
                                default_mode: Some(0o400),
                                items: Some({
                                    let mut items = vec![
                                        KeyToPath {
                                            key: "torrc".into(),
                                            mode: Some(0o400),
                                            path: "torrc".into(),
                                        },
                                    ];
                                    if object.onion_balanced() {
                                        items.push( KeyToPath {
                                            key: "ob_config".into(),
                                            mode: Some(0o400),
                                            path: "ob_config".into(),
                                        });
                                    }
                                   items
                                }),
                                name: Some(object.config_map_name().to_string()),
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
    }
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
