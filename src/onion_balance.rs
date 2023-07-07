use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, ExecAction, KeyToPath, PodSpec,
            PodTemplateSpec, Probe, ResourceRequirements, SecretVolumeSource, Volume, VolumeMount,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::apis::meta::v1::{Condition, LabelSelector, Time},
    chrono::Utc,
};
use kube::{
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    kubernetes::{
        self, error_policy, Annotations, Api, ConditionsExt, ConfigYaml, Labels, Object,
        Resource as KubernetesResource, ResourceName, SelectorLabels, Subset, Torrc,
    },
    metrics::Metrics,
    onion_key::OnionKey,
    Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # `OnionBalance`
///
/// An `OnionBalance` is an abstraction of a Tor Onion Balance.
///
/// Tor Onion Balance is the best way to load balance Tor Onion Services. The
/// load of introduction and rendezvous requests gets distributed across
/// multiple hosts while also increasing resiliency by eliminating single
/// points of failure.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionBalance",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the OnionBalance", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"OnionServices", "type":"number", "description":"The hostname of OnionServices", "jsonPath":".status.onionServices"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.state"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    status = "OnionBalanceStatus",
    version = "v1"
)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpec {
    /// Config Map settings.
    pub config_map: Option<OnionBalanceSpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<OnionBalanceSpecDeployment>,

    /// OnionKey settings.
    pub onion_key: OnionBalanceSpecOnionKey,

    /// OnionService part of the OnionBalance load balancing.
    pub onion_services: Vec<OnionBalanceSpecOnionService>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the OnionBalance
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecDeployment {
    /// Containers of the Deployment.
    pub containers: Option<OnionBalanceSpecDeploymentContainers>,

    /// Name of the Deployment.
    ///
    /// Default: name of the OnionBalance
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecDeploymentContainers {
    /// Onion Balance container.
    pub onion_balance: Option<OnionBalanceSpecDeploymentContainersOnionBalance>,

    /// Tor container.
    pub tor: Option<OnionBalanceSpecDeploymentContainersTor>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecDeploymentContainersOnionBalance {
    /// Resources of the container.
    pub resources: Option<ResourceRequirements>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecDeploymentContainersTor {
    /// Resources of the container.
    pub resources: Option<ResourceRequirements>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecOnionKey {
    /// Name of the OnionKey.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecOnionService {
    /// OnionKey reference of the OnionService.
    pub onion_key: OnionBalanceSpecOnionServiceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceSpecOnionServiceOnionKey {
    /// Hostname value of the OnionKey.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionBalanceStatus {
    /// Represents the latest available observations of a deployment's current state.
    ///
    /// ### Initialized
    ///
    /// `Initialized`
    ///
    /// ### OnionKey
    ///
    /// `NotFound`, `HostnameNotFound`, `Ready`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,

    /// OnionKey hostname.
    ///
    /// The hostname is only populated once `state` is "running".
    pub hostname: Option<String>,

    /// Number of OnionServices.
    pub onion_services: i32,
}

impl OnionBalance {
    #[must_use]
    fn default_name(&self) -> ResourceName {
        self.try_name().unwrap()
    }

    #[must_use]
    pub fn config_map_name(&self) -> ResourceName {
        self.spec
            .config_map
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn deployment_containers_onion_balance_resources(&self) -> Option<&ResourceRequirements> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .and_then(|f| f.onion_balance.as_ref())
            .and_then(|f| f.resources.as_ref())
    }

    #[must_use]
    pub fn deployment_containers_tor_resources(&self) -> Option<&ResourceRequirements> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .and_then(|f| f.tor.as_ref())
            .and_then(|f| f.resources.as_ref())
    }

    #[must_use]
    pub fn deployment_name(&self) -> ResourceName {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_key_name(&self) -> ResourceName {
        (&self.spec.onion_key.name).into()
    }

    #[must_use]
    pub fn status_conditions(&self) -> Option<&Vec<Condition>> {
        self.status.as_ref().map(|f| f.conditions.as_ref())
    }
}

impl KubernetesResource for OnionBalance {
    type Spec = OnionBalanceSpec;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }
}

impl Object for OnionBalance {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-balance";

    type Status = OnionBalanceStatus;

    fn status(&self) -> &Option<Self::Status> {
        &self.status
    }
}

impl Subset for OnionBalanceSpec {
    fn is_subset(&self, superset: &Self) -> bool {
        self == superset
    }
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
pub async fn run_controller(client: Client, config: Config, metrics: Metrics) {
    Metrics::kubernetes_api_usage_count::<OnionBalance>("watch");
    Metrics::kubernetes_api_usage_count::<ConfigMap>("watch");
    Metrics::kubernetes_api_usage_count::<Deployment>("watch");
    Controller::new(
        kube::Api::<OnionBalance>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<ConfigMap>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<Deployment>::all(client.clone()),
        WatcherConfig::default(),
    )
    .shutdown_on_signal()
    .run(
        reconciler,
        error_policy,
        Arc::new(Context {
            client,
            config,
            metrics,
        }),
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
    metrics: Metrics,
}

impl kubernetes::Context for Context {
    fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

/*
 * ============================================================================
 * State
 * ============================================================================
 */
enum State {
    OnionKeyNotFound,
    OnionKeyHostnameNotFound,
    Initialized(OnionKey),
}

impl From<&State> for Vec<Condition> {
    fn from(value: &State) -> Self {
        match value {
            State::OnionKeyNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The OnionKey was not found.".into(),
                observed_generation: None,
                reason: "NotFound".into(),
                status: "False".into(),
                type_: "OnionKey".into(),
            }],
            State::OnionKeyHostnameNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The OnionKey does not have a hostname.".into(),
                observed_generation: None,
                reason: "HostnameNotFound".into(),
                status: "False".into(),
                type_: "OnionKey".into(),
            }],
            State::Initialized(_) => vec![
                Condition {
                    last_transition_time: Time(Utc::now()),
                    message: "The OnionKey is ready.".into(),
                    observed_generation: None,
                    reason: "Ready".into(),
                    status: "True".into(),
                    type_: "OnionKey".into(),
                },
                Condition {
                    last_transition_time: Time(Utc::now()),
                    message: "The OnionBalance is initialized.".into(),
                    observed_generation: None,
                    reason: "Initialized".into(),
                    status: "True".into(),
                    type_: "Initialized".into(),
                },
            ],
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
    let _timer = ctx
        .metrics
        .count_and_measure(OnionBalance::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let config_yaml = generate_config_yaml(&object);
    let torrc = generate_torrc(&object);

    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    // OnionKey
    let state = reconcile_onion_key(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
    )
    .await?;

    if let State::Initialized(onion_key) = &state {
        let annotations = Annotations::new()
            .add(&config_yaml)
            .add_opt(&onion_key.hostname())
            .add(&torrc);

        // ConfigMap
        reconcile_config_map(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
            &torrc,
            &config_yaml,
        )
        .await?;

        // Deployment
        reconcile_deployment(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &ctx.config,
            &object,
            &annotations,
            &labels,
            &selector_labels,
            onion_key,
        )
        .await?;
    }

    // OnionBalance
    reconcile_onion_balance(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
        &state,
    )
    .await?;

    tracing::info!("reconciled");

    match state {
        State::Initialized(_) => Ok(Action::requeue(Duration::from_secs(3600))),
        _ => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

async fn reconcile_onion_key(api: &Api<OnionKey>, object: &OnionBalance) -> Result<State> {
    let Some(onion_key)  = api
        .get_opt(&object.onion_key_name())
        .await? else {
            return Ok(State::OnionKeyNotFound)
        };

    if onion_key.hostname().is_none() {
        return Ok(State::OnionKeyHostnameNotFound);
    }

    Ok(State::Initialized(onion_key))
}

async fn reconcile_config_map(
    api: &Api<ConfigMap>,
    object: &OnionBalance,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    config_yaml: &ConfigYaml,
) -> Result<()> {
    api.sync(
        object,
        [(
            (),
            generate_config_map(object, annotations, labels, torrc, config_yaml),
        )]
        .into(),
    )
    .await
    .map(|_| ())
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
    api.sync(
        object,
        [(
            (),
            generate_deployment(
                object,
                config,
                annotations,
                labels,
                selector_labels,
                onion_key,
            ),
        )]
        .into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_onion_balance(
    api: &Api<OnionBalance>,
    object: &OnionBalance,
    state: &State,
) -> Result<()> {
    api.update_status(
        object,
        OnionBalanceStatus {
            conditions: object
                .status_conditions()
                .unwrap_or(&Vec::new())
                .merge_from(&state.into()),
            hostname: if let State::Initialized(onion_key) = state {
                onion_key.hostname().as_ref().map(ToString::to_string)
            } else {
                None
            },
            onion_services: i32::try_from(object.spec.onion_services.len()).unwrap(),
        },
    )
    .await
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
            name: Some(object.config_map_name().into()),
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
            name: Some(object.deployment_name().into()),
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
                            resources: object.deployment_containers_onion_balance_resources().cloned(),
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
                            resources: object.deployment_containers_tor_resources().cloned(),
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
                                secret_name: Some(onion_key.secret_name().into()),
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
                                name: Some(object.config_map_name().into()),
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
                                name: Some(object.config_map_name().into()),
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
