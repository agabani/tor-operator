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
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::LabelSelector},
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
        self, error_policy, Annotations, Api, Labels, OBConfig, Object,
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
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the onion service", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"Onion Balance Hostname", "type":"string", "description":"The hostname of the onion balance", "jsonPath":".spec.onion_balance.onion_key.hostname"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.state"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    status = "OnionServiceStatus",
    version = "v1"
)]
/// # Onion Service
///
/// An Onion Service is an abstraction of a Tor Onion Service.
///
/// A Tor Onion Service is a service that can only be accessed over Tor.
/// Running a Tor Onion Service gives your users all teh security of HTTPS with
/// the added privacy benefits of Tor.
pub struct OnionServiceSpec {
    /// Config Map settings.
    pub config_map: Option<OnionServiceSpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<OnionServiceSpecDeployment>,

    /// Onion Balance part the Onion Service belongs to.
    ///
    /// Default: nil / none / null / undefined.
    pub onion_balance: Option<OnionServiceSpecOnionBalance>,

    /// Onion Key settings.
    pub onion_key: OnionServiceSpecOnionKey,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<OnionServiceSpecHiddenServicePort>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the Onion Service
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeployment {
    /// Name of the Deployment.
    ///
    /// Default: name of the Onion Service
    pub name: Option<String>,

    /// Resources of the Deployment.
    pub resources: Option<OnionServiceSpecDeploymentResources>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeploymentResources {
    /// Limits of the Resources.
    pub limits: Option<OnionServiceSpecDeploymentResourcesLimits>,

    /// Requests of the Resources.
    pub requests: Option<OnionServiceSpecDeploymentResourcesRequests>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeploymentResourcesLimits {
    /// CPU quantity of the Limits.
    pub cpu: Option<String>,

    /// Memory quantity of the Limits.
    pub memory: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeploymentResourcesRequests {
    /// CPU quantity of the Requests.
    pub cpu: Option<String>,

    /// Memory quantity of the Requests.
    pub memory: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalance {
    /// Onion Key reference of the Onion Balance.
    pub onion_key: OnionServiceSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalanceOnionKey {
    /// Hostname value of the Onion Key.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionKey {
    /// Name of the Onion Key.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceStatus {
    /// Onion key hostname.
    ///
    /// The hostname is only populated once `state` is "running".
    pub hostname: Option<String>,

    /// Human readable description of state.
    ///
    /// Possible values:
    ///
    /// - onion key not found
    /// - onion key hostname not found
    /// - running
    pub state: String,
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
    pub fn deployment_name(&self) -> ResourceName {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn deployment_resources_limits_cpu(&self) -> Option<&str> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.resources.as_ref())
            .and_then(|f| f.limits.as_ref())
            .and_then(|f| f.cpu.as_ref())
            .map(String::as_str)
    }

    #[must_use]
    pub fn deployment_resources_limits_memory(&self) -> Option<&str> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.resources.as_ref())
            .and_then(|f| f.limits.as_ref())
            .and_then(|f| f.memory.as_ref())
            .map(String::as_str)
    }

    #[must_use]
    pub fn deployment_resources_requests_cpu(&self) -> Option<&str> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.resources.as_ref())
            .and_then(|f| f.requests.as_ref())
            .and_then(|f| f.cpu.as_ref())
            .map(String::as_str)
    }

    #[must_use]
    pub fn deployment_resources_requests_memory(&self) -> Option<&str> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.resources.as_ref())
            .and_then(|f| f.requests.as_ref())
            .and_then(|f| f.memory.as_ref())
            .map(String::as_str)
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

    // onion key
    let state = reconcile_onion_key(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
    )
    .await?;

    if let State::Running(onion_key) = &state {
        // config map
        reconcile_config_map(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
            &torrc,
            &ob_config,
        )
        .await?;

        // deployment
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

    // onion service
    reconcile_onion_service(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
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

async fn reconcile_onion_key(api: &Api<OnionKey>, object: &OnionService) -> Result<State> {
    let Some(onion_key)  = api
        .get_opt(&object.onion_key_name())
        .await? else {
            return Ok(State::OnionKeyNotFound)
        };

    if onion_key.hostname().is_none() {
        return Ok(State::OnionKeyHostnameNotFound);
    }

    Ok(State::Running(onion_key))
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
            hostname: if let State::Running(onion_key) = state {
                onion_key.hostname().map(Into::into)
            } else {
                None
            },
            state: state.to_string(),
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
                        resources: Some(ResourceRequirements {
                            limits: Some({
                                let mut map = BTreeMap::new();
                                if let Some(quantity) = object.deployment_resources_limits_cpu() {
                                    map.insert("cpu".into(), Quantity(quantity.into()));
                                }
                                if let Some(quantity) = object.deployment_resources_limits_memory() {
                                    map.insert("memory".into(), Quantity(quantity.into()));
                                }
                                map
                            }),
                            requests: Some({
                                let mut map = BTreeMap::new();
                                if let Some(quantity) = object.deployment_resources_requests_cpu() {
                                    map.insert("cpu".into(), Quantity(quantity.into()));
                                }
                                if let Some(quantity) = object.deployment_resources_requests_memory() {
                                    map.insert("memory".into(), Quantity(quantity.into()));
                                }
                                map
                            }),
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
