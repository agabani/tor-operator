use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            Affinity, Capabilities, ConfigMap, ConfigMapVolumeSource, Container, ExecAction,
            KeyToPath, LocalObjectReference, PodSecurityContext, PodSpec, PodTemplateSpec, Probe,
            SecretVolumeSource, SecurityContext, Toleration, TopologySpreadConstraint, Volume,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::apis::meta::v1::{Condition, LabelSelector, Time},
    chrono::Utc,
};
use kube::{
    Client, CustomResource, CustomResourceExt, Resource,
    core::ObjectMeta,
    runtime::{Controller, controller::Action, watcher::Config as WatcherConfig},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Result,
    collections::vec_get_or_insert,
    kubernetes::{
        self, Annotations, Api, ConditionsExt, Labels, Object, Resource as KubernetesResource,
        ResourceName, SelectorLabels, Subset, Torrc as KubernetesTorrc, error_policy,
        pod_security_context,
    },
    metrics::Metrics,
    onion_key::OnionKey,
    tor::{Hostname, OBConfig, Torrc},
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
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[kube(
    derive = "Default",
    derive = "PartialEq",
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the OnionService", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"OnionBalance Hostname", "type":"string", "description":"The hostname of the OnionBalance", "jsonPath":".spec.onionBalance.onionKey.hostname"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.summary.Initialized"}"#,
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

    /// `OnionBalance` the `OnionService` belongs to.
    ///
    /// Default: nil / none / null / undefined.
    pub onion_balance: Option<OnionServiceSpecOnionBalance>,

    /// `OnionKey` settings.
    pub onion_key: OnionServiceSpecOnionKey,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<OnionServiceSpecHiddenServicePort>,

    /// Tor torrc settings.
    pub torrc: Option<KubernetesTorrc>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecConfigMap {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: <http://kubernetes.io/docs/user-guide/annotations>
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: <http://kubernetes.io/docs/user-guide/labels>
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the Config Map.
    ///
    /// Default: name of the `OnionService`
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecDeployment {
    /// If specified, the pod's scheduling constraints
    pub affinity: Option<Affinity>,

    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: <http://kubernetes.io/docs/user-guide/annotations>
    pub annotations: Option<BTreeMap<String, String>>,

    /// Containers of the Deployment.
    pub containers: Option<Vec<Container>>,

    /// `ImagePullSecrets` is an optional list of references to secrets in the same namespace to use for pulling any of the images used by this `PodSpec`. If specified, these secrets will be passed to individual puller implementations for them to use. More info: <https://kubernetes.io/docs/concepts/containers/images#specifying-imagepullsecrets-on-a-pod>
    pub image_pull_secrets: Option<Vec<LocalObjectReference>>,

    /// List of initialization containers belonging to the pod. Init containers are executed in order prior to containers being started. If any init container fails, the pod is considered to have failed and is handled according to its restartPolicy. The name for an init container or normal container must be unique among all containers. Init containers may not have Lifecycle actions, Readiness probes, Liveness probes, or Startup probes. The resourceRequirements of an init container are taken into account during scheduling by finding the highest request/limit for each resource type, and then using the max of of that value or the sum of the normal containers. Limits are applied to init containers in a similar fashion. Init containers cannot currently be added or removed. Cannot be updated. More info: <https://kubernetes.io/docs/concepts/workloads/pods/init-containers/>
    pub init_containers: Option<Vec<Container>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: <http://kubernetes.io/docs/user-guide/labels>
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the Deployment.
    ///
    /// Default: name of the `OnionService`
    pub name: Option<String>,

    /// `NodeSelector` is a selector which must be true for the pod to fit on a node. Selector which must match a node's labels for the pod to be scheduled on that node. More info: <https://kubernetes.io/docs/concepts/configuration/assign-pod-node/>
    pub node_selector: Option<std::collections::BTreeMap<String, String>>,

    /// `SecurityContext` holds pod-level security attributes and common container settings. Optional: Defaults to empty.  See type description for default values of each field.
    pub security_context: Option<PodSecurityContext>,

    /// If specified, the pod's tolerations.
    pub tolerations: Option<Vec<Toleration>>,

    /// `TopologySpreadConstraints` describes how a group of pods ought to spread across topology domains. Scheduler will schedule pods in a way which abides by the constraints. All topologySpreadConstraints are `ANDed`.
    pub topology_spread_constraints: Option<Vec<TopologySpreadConstraint>>,

    /// List of volumes that can be mounted by containers belonging to the pod. More info: <https://kubernetes.io/docs/concepts/storage/volumes>
    pub volumes: Option<Vec<Volume>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionBalance {
    /// `OnionKey` reference of the `OnionBalance`.
    pub onion_key: OnionServiceSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionBalanceOnionKey {
    /// Hostname value of the `OnionKey`.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceSpecOnionKey {
    /// Name of the `OnionKey`.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
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
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OnionServiceStatus {
    /// Represents the latest available observations of a deployment's current state.
    ///
    /// ### Initialized
    ///
    /// `Initialized`
    ///
    /// ### `OnionKey`
    ///
    /// `NotFound`, `HostnameNotFound`, `Ready`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,

    /// `OnionKey` hostname.
    ///
    /// The hostname is only populated once `state` is "running".
    pub hostname: Option<String>,

    /// Represents the latest available observations of a deployment's current state.
    #[serde(default)]
    pub summary: BTreeMap<String, String>,
}

impl OnionService {
    #[must_use]
    fn default_name(&self) -> ResourceName {
        self.try_name().unwrap()
    }

    #[must_use]
    pub fn config_map_annotations(&self) -> Option<Annotations> {
        self.spec
            .config_map
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .cloned()
            .map(Into::into)
    }

    #[must_use]
    pub fn config_map_labels(&self) -> Option<Labels> {
        self.spec
            .config_map
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .cloned()
            .map(Into::into)
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
    pub fn deployment_affinity(&self) -> Option<Affinity> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.affinity.as_ref())
            .cloned()
    }

    #[must_use]
    pub fn deployment_annotations(&self) -> Option<Annotations> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .cloned()
            .map(Into::into)
    }

    #[must_use]
    pub fn deployment_containers(&self) -> Vec<Container> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn deployment_image_pull_secrets(&self) -> Option<Vec<LocalObjectReference>> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.image_pull_secrets.as_ref())
            .cloned()
    }

    #[must_use]
    pub fn deployment_init_containers(&self) -> Vec<Container> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.init_containers.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn deployment_labels(&self) -> Option<Labels> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .cloned()
            .map(Into::into)
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
    pub fn deployment_node_selector(&self) -> Option<BTreeMap<String, String>> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.node_selector.as_ref())
            .cloned()
    }

    #[must_use]
    pub fn deployment_security_context(&self) -> PodSecurityContext {
        pod_security_context(
            self.spec
                .deployment
                .as_ref()
                .and_then(|f| f.security_context.as_ref())
                .cloned()
                .unwrap_or_default(),
        )
    }

    #[must_use]
    pub fn deployment_tolerations(&self) -> Option<Vec<Toleration>> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.tolerations.as_ref())
            .cloned()
    }

    #[must_use]
    pub fn deployment_topology_spread_constraints(&self) -> Option<Vec<TopologySpreadConstraint>> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.topology_spread_constraints.as_ref())
            .cloned()
    }

    #[must_use]
    pub fn deployment_volumes(&self) -> Vec<Volume> {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.volumes.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn onion_balanced(&self) -> bool {
        self.spec.onion_balance.is_some()
    }

    #[must_use]
    pub fn onion_balance_onion_key_hostname(&self) -> Option<Hostname> {
        self.spec
            .onion_balance
            .as_ref()
            .map(|onion_balance| onion_balance.onion_key.hostname.clone())
            .map(Hostname::new)
    }

    #[must_use]
    pub fn onion_key_name(&self) -> ResourceName {
        ResourceName::from(&self.spec.onion_key.name)
    }

    #[must_use]
    pub fn ports(&self) -> &[OnionServiceSpecHiddenServicePort] {
        &self.spec.ports
    }

    #[must_use]
    pub fn torrc_template(&self) -> Option<&str> {
        self.spec
            .torrc
            .as_ref()
            .and_then(|f| f.template.as_ref())
            .map(String::as_str)
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

    fn status(&self) -> Option<&Self::Status> {
        self.status.as_ref()
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
    metrics.kubernetes_api_usage_count::<OnionService>("watch");
    metrics.kubernetes_api_usage_count::<ConfigMap>("watch");
    metrics.kubernetes_api_usage_count::<Deployment>("watch");
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
    Initialized(Box<OnionKey>),
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
#[tracing::instrument(skip_all)]
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(OnionService::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let ob_config = generate_ob_config(&object);
    let torrc = generate_torrc(&object);

    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    // OnionKey
    let state = reconcile_onion_key(
        &Api::new(
            kube::Api::namespaced(ctx.client.clone(), &namespace),
            ctx.metrics.clone(),
        ),
        &object,
    )
    .await?;

    if let State::Initialized(onion_key) = &state {
        let annotations = Annotations::new()
            .add_opt(onion_key.hostname().as_ref())
            .add_opt(ob_config.as_ref())
            .add(&torrc);

        // ConfigMap
        reconcile_config_map(
            &Api::new(
                kube::Api::namespaced(ctx.client.clone(), &namespace),
                ctx.metrics.clone(),
            ),
            &object,
            &annotations,
            &labels,
            &torrc,
            ob_config.as_ref(),
        )
        .await?;

        // Deployment
        reconcile_deployment(
            &Api::new(
                kube::Api::namespaced(ctx.client.clone(), &namespace),
                ctx.metrics.clone(),
            ),
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
        &Api::new(
            kube::Api::namespaced(ctx.client.clone(), &namespace),
            ctx.metrics.clone(),
        ),
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
    let Some(onion_key) = api.get_opt(&object.onion_key_name()).await? else {
        return Ok(State::OnionKeyNotFound);
    };

    if onion_key.hostname().is_none() {
        return Ok(State::OnionKeyHostnameNotFound);
    }

    Ok(State::Initialized(Box::new(onion_key)))
}

async fn reconcile_config_map(
    api: &Api<ConfigMap>,
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    ob_config: Option<&OBConfig>,
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
    let conditions = object
        .status_conditions()
        .unwrap_or(&Vec::new())
        .merge_from(&state.into());

    let summary = conditions
        .iter()
        .fold(BTreeMap::new(), |mut summary, condition| {
            summary.insert(condition.type_.clone(), condition.reason.clone());
            summary
        });

    api.update_status(
        object,
        OnionServiceStatus {
            conditions,
            hostname: if let State::Initialized(onion_key) = state {
                onion_key.hostname().as_ref().map(ToString::to_string)
            } else {
                None
            },
            summary,
        },
    )
    .await
}

fn generate_ob_config(object: &OnionService) -> Option<OBConfig> {
    object
        .onion_balance_onion_key_hostname()
        .map(|hostname| OBConfig::builder().master_onion_address(&hostname).build())
}

fn generate_torrc(object: &OnionService) -> Torrc {
    let mut torrc = Torrc::builder();
    if let Some(template) = object.torrc_template() {
        torrc = torrc.template(template);
    }
    torrc = torrc
        .data_dir("${TOR_TMP_DIR}/home/.tor")
        .hidden_service_dir("${TOR_TMP_DIR}/var/lib/tor/hidden_service");
    if object.onion_balanced() {
        torrc = torrc.hidden_service_onion_balance_instance(true);
    }
    torrc = object.ports().iter().fold(torrc, |torrc, port| {
        torrc.hidden_service_port(port.virtport, &port.target)
    });
    torrc.build()
}

fn generate_config_map(
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    ob_config: Option<&OBConfig>,
    torrc: &Torrc,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(object.config_map_name().into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.config_map_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.config_map_labels())
                    .into(),
            ),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some({
            let mut data = BTreeMap::from([("torrc".into(), torrc.to_string())]);
            if let Some(ob_config) = ob_config {
                data.insert("ob_config".into(), ob_config.to_string());
            }
            data
        }),
        ..Default::default()
    }
}

#[allow(clippy::too_many_lines)]
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
            name: Some(object.deployment_name().into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.deployment_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.deployment_labels())
                    .into(),
            ),
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
                    annotations: Some(
                        annotations
                            .clone()
                            .append_reverse(object.deployment_annotations())
                            .into(),
                    ),
                    labels: Some(
                        labels
                            .clone()
                            .append_reverse(object.deployment_labels())
                            .into(),
                    ),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    affinity: object.deployment_affinity(),
                    containers: generate_deployment_containers(object, config),
                    image_pull_secrets: object.deployment_image_pull_secrets(),
                    init_containers: Some(generate_deployment_init_containers(object)),
                    node_selector: object.deployment_node_selector(),
                    security_context: Some(object.deployment_security_context()),
                    tolerations: object.deployment_tolerations(),
                    topology_spread_constraints: object.deployment_topology_spread_constraints(),
                    volumes: Some(generate_deployment_volumes(object, onion_key)),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn generate_deployment_containers(object: &OnionService, config: &Config) -> Vec<Container> {
    let mut containers = object.deployment_containers();

    {
        let container = vec_get_or_insert(&mut containers, |f| f.name == "tor");
        container.name = "tor".to_string();
        container.args = Some(vec![
            "-c".into(),
            {
                let mut commands = vec!["export TOR_TMP_DIR=${TOR_TMP_DIR:-$(mktemp -d --suffix=.tor -p /tmp)}"];

                // hidden_service
                commands.push("mkdir -p $TOR_TMP_DIR/var/lib/tor/hidden_service");
                commands.push("chmod 700 $TOR_TMP_DIR/var/lib/tor/hidden_service");
                commands.push("cp -L /etc/secrets/* $TOR_TMP_DIR/var/lib/tor/hidden_service");

                // ob_config
                if object.onion_balanced() {
                    commands.push("cp -L /etc/configs/ob_config $TOR_TMP_DIR/var/lib/tor/hidden_service/ob_config");
                }

                // torrc
                commands.push("mkdir -p $TOR_TMP_DIR/usr/local/etc/tor");
                commands.push("envsubst < /etc/configs/torrc > $TOR_TMP_DIR/usr/local/etc/tor/torrc");

                // data directory
                commands.push("mkdir -p $TOR_TMP_DIR/home/.tor");
                commands.push("chmod 700 $TOR_TMP_DIR/home/.tor");

                // executable
                commands.push("tor -f $TOR_TMP_DIR/usr/local/etc/tor/torrc");
                commands
            }
            .join(" && "),
        ]);
        container.command = Some(vec!["/bin/bash".into()]);
        container.image = Some(config.tor_image.uri.clone());
        container.image_pull_policy = Some(config.tor_image.pull_policy.clone());
        container.liveness_probe = Some(Probe {
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
        });
        container.readiness_probe = Some(Probe {
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
        });

        let volume_mounts = container.volume_mounts.get_or_insert_with(Default::default);

        {
            let volume_mount = vec_get_or_insert(volume_mounts, |f| f.name == "etc-secrets");
            volume_mount.name = "etc-secrets".to_string();
            volume_mount.mount_path = "/etc/secrets".into();
            volume_mount.read_only = Some(true);
        }

        {
            let volume_mount = vec_get_or_insert(volume_mounts, |f| f.name == "etc-configs");
            volume_mount.name = "etc-configs".to_string();
            volume_mount.mount_path = "/etc/configs".into();
            volume_mount.read_only = Some(true);
        }
    }

    for container in &mut containers {
        container.security_context = Some(SecurityContext {
            capabilities: Some(Capabilities {
                drop: Some(vec!["ALL".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    containers
}

fn generate_deployment_init_containers(object: &OnionService) -> Vec<Container> {
    let mut containers = object.deployment_init_containers();

    for container in &mut containers {
        container.security_context = Some(SecurityContext {
            capabilities: Some(Capabilities {
                drop: Some(vec!["ALL".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        });
    }

    containers
}

fn generate_deployment_volumes(object: &OnionService, onion_key: &OnionKey) -> Vec<Volume> {
    let mut volumes = object.deployment_volumes();

    {
        let volume = vec_get_or_insert(&mut volumes, |f| f.name == "etc-secrets");
        volume.name = "etc-secrets".to_string();
        volume.secret = Some(SecretVolumeSource {
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
        });
    }

    {
        let volume = vec_get_or_insert(&mut volumes, |f| f.name == "etc-configs");
        volume.name = "etc-configs".to_string();
        volume.config_map = Some(ConfigMapVolumeSource {
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
            name: object.config_map_name().into(),
            optional: Some(false),
        });
    }

    volumes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config() {
        let object = &OnionService {
            spec: OnionServiceSpec {
                ports: vec![
                    OnionServiceSpecHiddenServicePort {
                        target: "example:80".into(),
                        virtport: 80,
                    },
                    OnionServiceSpecHiddenServicePort {
                        target: "example:443".into(),
                        virtport: 443,
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let ob_config = generate_ob_config(object);

        assert!(ob_config.is_none());

        let torrc = generate_torrc(object);

        assert_eq!(
            r"DataDirectory ${TOR_TMP_DIR}/home/.tor
HiddenServiceDir ${TOR_TMP_DIR}/var/lib/tor/hidden_service
HiddenServicePort 80 example:80
HiddenServicePort 443 example:443",
            torrc.to_string()
        );
    }

    #[test]
    fn config_onion_balance() {
        let object = &OnionService {
            spec: OnionServiceSpec {
                onion_balance: Some(OnionServiceSpecOnionBalance {
                    onion_key: OnionServiceSpecOnionBalanceOnionKey {
                        hostname: "hostname.onion".into(),
                    },
                }),
                ports: vec![
                    OnionServiceSpecHiddenServicePort {
                        target: "example:80".into(),
                        virtport: 80,
                    },
                    OnionServiceSpecHiddenServicePort {
                        target: "example:443".into(),
                        virtport: 443,
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        let ob_config = generate_ob_config(object).unwrap();

        assert_eq!("MasterOnionAddress hostname.onion", ob_config.to_string());

        let torrc = generate_torrc(object);

        assert_eq!(
            r"DataDirectory ${TOR_TMP_DIR}/home/.tor
HiddenServiceDir ${TOR_TMP_DIR}/var/lib/tor/hidden_service
HiddenServiceOnionbalanceInstance 1
HiddenServicePort 80 example:80
HiddenServicePort 443 example:443",
            torrc.to_string()
        );
    }
}
