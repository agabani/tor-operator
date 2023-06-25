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
    api::Patch,
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    kubernetes::{
        Annotations, ConfigYaml, KubeCrdResourceExt, KubeResourceExt, Labels, SelectorLabels, Torrc,
    },
    onion_key::OnionKey,
    Error, Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # Onion Balance
///
/// An Onion Balance is an abstraction of a Tor Onion Balance.
///
/// Tor Onion Balance is the best way to load balance Tor Onion Services. The
/// load of introduction and rendezvous requests gets distributed across
/// multiple hosts while also increasing resiliency by eliminating single
/// points of failure.
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
    /// Config Map settings.
    pub config_map: Option<OnionBalanceSpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<OnionBalanceSpecDeployment>,

    /// Onion Key settings.
    pub onion_key: OnionBalanceSpecOnionKey,

    /// Onion Service part of the Onion Balance load balancing.
    pub onion_services: Vec<OnionBalanceSpecOnionService>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the Onion Balance
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecDeployment {
    /// Name of the Deployment.
    ///
    /// Default: name of the Onion Balance
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionKey {
    /// Name of the Onion Key.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionService {
    /// Onion Key reference of the Onion Service.
    pub onion_key: OnionBalanceSpecOnionServiceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionBalanceSpecOnionServiceOnionKey {
    /// Hostname value of the Onion Key.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionBalanceStatus {
    /// Human readable description of state.
    ///
    /// Possible values:
    ///
    /// - onion key not found
    /// - onion key hostname not found
    /// - running
    pub state: String,
}

impl OnionBalance {
    #[must_use]
    fn default_name(&self) -> &str {
        self.try_name().unwrap().into()
    }

    #[must_use]
    pub fn config_map_name(&self) -> &str {
        self.spec
            .config_map
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn deployment_name(&self) -> &str {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_key_name(&self) -> &str {
        &self.spec.onion_key.name
    }
}

impl KubeResourceExt for OnionBalance {}

impl KubeCrdResourceExt for OnionBalance {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-balance";
}

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
pub async fn run_controller(client: Client, config: Config) {
    Controller::new(
        Api::<OnionBalance>::all(client.clone()),
        WatcherConfig::default(),
    )
    .shutdown_on_signal()
    .run(
        reconciler,
        error_policy,
        Arc::new(Context { client, config }),
    )
    .for_each(|_| async {})
    .await;
}

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
 * State
 * ============================================================================
 */
enum State {
    OnionKeyNotFound,
    OnionKeyHostnameNotFound,
    Running(OnionKey),
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::OnionKeyNotFound => write!(f, "onion key not found"),
            State::OnionKeyHostnameNotFound => write!(f, "onion key hostname not found"),
            State::Running(_) => write!(f, "running"),
        }
    }
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<OnionBalance>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let torrc = generate_torrc(&object);
    let config_yaml = generate_config_yaml(&object);

    let annotations = generate_annotations(&config_yaml, &torrc);
    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    // onion key
    let state = reconcile_onion_key(
        //
        &Api::namespaced(ctx.client.clone(), &namespace),
        &object,
    )
    .await?;

    if let State::Running(onion_key) = &state {
        // config map
        reconcile_config_map(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &object,
            &annotations,
            &labels,
            &torrc,
            &config_yaml,
        )
        .await?;

        // deployment
        reconcile_deployment(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &ctx.config,
            &object,
            &annotations,
            &labels,
            &selector_labels,
            onion_key,
        )
        .await?;
    }

    // onion balance
    reconcile_onion_balance(
        &Api::namespaced(ctx.client.clone(), &namespace),
        &object,
        &state,
    )
    .await?;

    tracing::info!("reconciled");

    match state {
        State::Running(_) => Ok(Action::requeue(Duration::from_secs(3600))),
        _ => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

async fn reconcile_onion_key(api: &Api<OnionKey>, object: &OnionBalance) -> Result<State> {
    let Some(onion_key)  = api
        .get_opt(object.onion_key_name())
        .await
        .map_err(Error::Kube)? else {
            return Ok(State::OnionKeyNotFound)
        };

    if onion_key.hostname().is_none() {
        return Ok(State::OnionKeyHostnameNotFound);
    }

    Ok(State::Running(onion_key))
}

async fn reconcile_config_map(
    api: &Api<ConfigMap>,
    object: &OnionBalance,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    config_yaml: &ConfigYaml,
) -> Result<()> {
    // creation
    let config_map = generate_config_map(object, annotations, labels, torrc, config_yaml);

    api.patch(
        &config_map.try_name()?,
        &object.patch_params(),
        &Patch::Apply(&config_map),
    )
    .await
    .map_err(Error::Kube)?;

    // deletion
    let config_maps = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    for config_map in &config_maps {
        let config_map_name = config_map.try_name()?;

        if config_map_name.as_str() != object.config_map_name() {
            api.delete(&config_map_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_deployment(
    api: &Api<Deployment>,
    config: &Config,
    object: &OnionBalance,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Result<()> {
    // creation
    let deployment = generate_deployment(
        object,
        config,
        annotations,
        labels,
        selector_labels,
        onion_key,
    );

    api.patch(
        &deployment.try_name()?,
        &object.patch_params(),
        &Patch::Apply(&deployment),
    )
    .await
    .map_err(Error::Kube)?;

    // deletion
    let deployments = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    for deployment in &deployments {
        let deployment_name = deployment.try_name()?;

        if deployment_name.as_str() != object.deployment_name() {
            api.delete(&deployment_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_onion_balance(
    api: &Api<OnionBalance>,
    object: &OnionBalance,
    state: &State,
) -> Result<()> {
    let state = state.to_string();

    let changed = object
        .status
        .as_ref()
        .map_or(true, |status| status.state != state);

    if changed {
        api.patch_status(
            &object.try_name()?,
            &object.patch_status_params(),
            &object.patch_status(OnionBalanceStatus { state }),
        )
        .await
        .map_err(Error::Kube)?;
    }

    Ok(())
}

fn generate_annotations(config_yaml: &ConfigYaml, torrc: &Torrc) -> Annotations {
    Annotations::new(BTreeMap::from([
        config_yaml.to_annotation_tuple(),
        torrc.to_annotation_tuple(),
    ]))
}

#[allow(unused_variables)]
fn generate_torrc(object: &OnionBalance) -> Torrc {
    Torrc::new(vec!["SocksPort 9050", "ControlPort 127.0.0.1:6666"].join("\n"))
}

fn generate_config_yaml(object: &OnionBalance) -> ConfigYaml {
    ConfigYaml::new(
        vec![
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
        ]
        .join("\n"),
    )
}

fn generate_config_map(
    object: &OnionBalance,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    config_yaml: &ConfigYaml,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(object.config_map_name().to_string()),
            annotations: Some(annotations.into()),
            labels: Some(labels.into()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("torrc".into(), torrc.into()),
            ("config.yaml".into(), config_yaml.into()),
        ])),
        ..Default::default()
    }
}

fn generate_deployment(
    object: &OnionBalance,
    config: &Config,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Deployment {
    Deployment {
        metadata: ObjectMeta {
            name: Some(object.deployment_name().to_string()),
            annotations: Some(annotations.into()),
            labels: Some(labels.into()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(selector_labels.into()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    annotations: Some(annotations.into()),
                    labels: Some(labels.into()),
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
                                    "tor -f /usr/local/etc/tor/torrc",
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
                                secret_name: Some(onion_key.secret_name().to_string()),
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
                                name: Some(object.config_map_name().to_string()),
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
fn error_policy(object: Arc<OnionBalance>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
