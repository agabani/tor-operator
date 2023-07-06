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
        self, error_policy, Annotations, Api, ConditionsExt, DeploymentContainerResources, Labels,
        OBConfig, Object, Resource as KubernetesResource, ResourceName, SelectorLabels, Subset,
        Torrc,
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
/// # `OnionService`
///
/// An `OnionService` is an abstraction of a Tor Onion Service.
///
/// A Tor Onion Service is a service that can only be accessed over Tor.
/// Running a Tor Onion Service gives your users all the security of HTTPS with
/// the added privacy benefits of Tor.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the OnionService", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"OnionBalance Hostname", "type":"string", "description":"The hostname of the OnionBalance", "jsonPath":".spec.onionBalance.onionKey.hostname"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.state"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    status = "OnionServiceStatus",
    version = "v1"
)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpec {
    /// Config Map settings.
    pub config_map: Option<OnionServiceSpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<OnionServiceSpecDeployment>,

    /// OnionBalance the OnionService belongs to.
    ///
    /// Default: nil / none / null / undefined.
    pub onion_balance: Option<OnionServiceSpecOnionBalance>,

    /// OnionKey settings.
    pub onion_key: OnionServiceSpecOnionKey,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<OnionServiceSpecHiddenServicePort>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the OnionService
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecDeployment {
    /// Containers of the Deployment.
    pub containers: Option<OnionServiceSpecDeploymentContainers>,

    /// Name of the Deployment.
    ///
    /// Default: name of the OnionService
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecDeploymentContainers {
    /// Tor container.
    pub tor: Option<OnionServiceSpecDeploymentContainersTor>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecDeploymentContainersTor {
    /// Resources of the container.
    pub resources: Option<DeploymentContainerResources>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionBalance {
    /// OnionKey reference of the OnionBalance.
    pub onion_key: OnionServiceSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionBalanceOnionKey {
    /// Hostname value of the OnionKey.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionKey {
    /// Name of the OnionKey.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecHiddenServicePort {
    /// The target any incoming traffic will be redirect to.
    ///
    /// Example: example.default.svc.cluster.local:80
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    ///
    /// Example: 80
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceStatus {
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
}

impl OnionService {
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
    pub fn deployment_containers_tor_resources(&self) -> Option<&DeploymentContainerResources> {
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
    pub fn onion_key_name(&self) -> ResourceName {
        (&self.spec.onion_key.name).into()
    }

    #[must_use]
    pub fn ports(&self) -> &[OnionServiceSpecHiddenServicePort] {
        &self.spec.ports
    }

    #[must_use]
    pub fn status_conditions(&self) -> Option<&Vec<Condition>> {
        self.status.as_ref().map(|f| f.conditions.as_ref())
    }
}

impl KubernetesResource for OnionService {
    type Spec = OnionServiceSpec;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }
}

impl Object for OnionService {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-service";

    type Status = OnionServiceStatus;

    fn status(&self) -> &Option<Self::Status> {
        &self.status
    }
}

impl Subset for OnionServiceSpec {
    fn is_subset(&self, superset: &Self) -> bool {
        self == superset
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
pub async fn run_controller(client: Client, config: Config, metrics: Metrics) {
    Metrics::kubernetes_api_usage_count::<OnionService>("watch");
    Metrics::kubernetes_api_usage_count::<ConfigMap>("watch");
    Metrics::kubernetes_api_usage_count::<Deployment>("watch");
    Controller::new(
        kube::Api::<OnionService>::all(client.clone()),
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
                    message: "The OnionService is initialized.".into(),
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
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(OnionService::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let ob_config = generate_ob_config(&object);
    let torrc = generate_torrc(&object);

    let annotations = generate_annotations(&torrc);
    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    // OnionKey
    let state = reconcile_onion_key(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
    )
    .await?;

    if let State::Initialized(onion_key) = &state {
        // ConfigMap
        reconcile_config_map(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
            &torrc,
            &ob_config,
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

    // OnionService
    reconcile_onion_service(
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

async fn reconcile_onion_key(api: &Api<OnionKey>, object: &OnionService) -> Result<State> {
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
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    ob_config: &Option<OBConfig>,
) -> Result<()> {
    api.sync(
        object,
        [(
            (),
            generate_config_map(object, annotations, labels, ob_config, torrc),
        )]
        .into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_deployment(
    api: &Api<Deployment>,
    config: &Config,
    object: &OnionService,
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

async fn reconcile_onion_service(
    api: &Api<OnionService>,
    object: &OnionService,
    state: &State,
) -> Result<()> {
    api.update_status(
        object,
        OnionServiceStatus {
            conditions: object
                .status_conditions()
                .unwrap_or(&Vec::new())
                .merge_from(&state.into()),
            hostname: if let State::Initialized(onion_key) = state {
                onion_key.hostname().map(Into::into)
            } else {
                None
            },
        },
    )
    .await
}

fn generate_annotations(torrc: &Torrc) -> Annotations {
    Annotations::new(BTreeMap::from([(torrc.to_annotation_tuple())]))
}

fn generate_ob_config(object: &OnionService) -> Option<OBConfig> {
    object
        .spec
        .onion_balance
        .as_ref()
        .map(|f| OBConfig::new(format!("MasterOnionAddress {}", f.onion_key.hostname)))
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
    Torrc::new(torrc.join("\n"))
}

fn generate_config_map(
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    ob_config: &Option<OBConfig>,
    torrc: &Torrc,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(object.config_map_name().to_string()),
            annotations: Some(annotations.into()),
            labels: Some(labels.into()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some({
            let mut data = BTreeMap::from([("torrc".into(), torrc.into())]);
            if let Some(ob_config) = ob_config {
                data.insert("ob_config".into(), ob_config.into());
            }
            data
        }),
        ..Default::default()
    }
}

fn generate_deployment(
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
                                commands.push("tor -f /usr/local/etc/tor/torrc");
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
                        resources: object.deployment_containers_tor_resources().map(Into::into),
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
                                    let mut items = vec![KeyToPath {
                                        key: "torrc".into(),
                                        mode: Some(0o400),
                                        path: "torrc".into(),
                                    }];
                                    if object.onion_balanced() {
                                        items.push(KeyToPath {
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
