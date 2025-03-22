use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        autoscaling::v2::{
            CrossVersionObjectReference, HorizontalPodAutoscaler, HorizontalPodAutoscalerBehavior,
            HorizontalPodAutoscalerSpec, MetricSpec,
        },
        core::v1::{
            Affinity, Capabilities, ConfigMap, ConfigMapVolumeSource, Container, ExecAction,
            KeyToPath, LocalObjectReference, PodSecurityContext, PodSpec, PodTemplateSpec, Probe,
            SecurityContext, Service, ServicePort, ServiceSpec, Toleration,
            TopologySpreadConstraint, Volume,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::{
        apis::meta::v1::{Condition, LabelSelector, Time},
        util::intstr::IntOrString,
    },
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
        ResourceName, SelectorLabels, Torrc as KubernetesTorrc, error_policy, pod_security_context,
    },
    metrics::Metrics,
    tor::Torrc,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # `TorProxy`
///
/// A `TorProxy` is collection of `Tor` clients load balanced by a `Service`.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[kube(
    derive = "Default",
    derive = "PartialEq",
    group = "tor.agabani.co.uk",
    kind = "TorProxy",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the tor proxy", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"Replicas", "type":"number", "description":"Number of replicas", "jsonPath":".status.replicas"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.summary.Initialized"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    scale = r#"{"specReplicasPath":".spec.deployment.replicas", "statusReplicasPath":".status.replicas", "labelSelectorPath":".status.labelSelector"}"#,
    status = "TorProxyStatus",
    version = "v1"
)]
#[serde(rename_all = "camelCase")]
pub struct TorProxySpec {
    /// Config Map settings.
    pub config_map: Option<TorProxySpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<TorProxySpecDeployment>,

    /// `HorizontalPodAutoscaler` settings.
    pub horizontal_pod_autoscaler: Option<TorProxyHorizontalPodAutoscaler>,

    /// Service settings.
    pub service: TorProxySpecService,

    /// Tor torrc settings.
    pub torrc: Option<KubernetesTorrc>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorProxySpecConfigMap {
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
pub struct TorProxySpecDeployment {
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

    /// Number of replicas.
    #[serde(default = "default_deployment_replicas")]
    pub replicas: i32,

    /// `SecurityContext` holds pod-level security attributes and common container settings. Optional: Defaults to empty.  See type description for default values of each field.
    pub security_context: Option<PodSecurityContext>,

    /// If specified, the pod's tolerations.
    pub tolerations: Option<Vec<Toleration>>,

    /// `TopologySpreadConstraints` describes how a group of pods ought to spread across topology domains. Scheduler will schedule pods in a way which abides by the constraints. All topologySpreadConstraints are `ANDed`.
    pub topology_spread_constraints: Option<Vec<TopologySpreadConstraint>>,

    /// List of volumes that can be mounted by containers belonging to the pod. More info: <https://kubernetes.io/docs/concepts/storage/volumes>
    pub volumes: Option<Vec<Volume>>,
}

fn default_deployment_replicas() -> i32 {
    3
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorProxyHorizontalPodAutoscaler {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: <http://kubernetes.io/docs/user-guide/annotations>
    pub annotations: Option<BTreeMap<String, String>>,

    /// behavior configures the scaling behavior of the target in both Up and Down directions (scaleUp and scaleDown fields respectively). If not set, the default `HPAScalingRules` for scale up and scale down are used.
    pub behavior: Option<HorizontalPodAutoscalerBehavior>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: <http://kubernetes.io/docs/user-guide/labels>
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the `HorizontalPodAutoscaler`.
    ///
    /// Default: name of the `TorIngress`
    pub name: Option<String>,

    /// maxReplicas is the upper limit for the number of replicas to which the autoscaler can scale up. It cannot be less that minReplicas.
    pub max_replicas: i32,

    /// metrics contains the specifications for which to use to calculate the desired replica count (the maximum replica count across all metrics will be used).  The desired replica count is calculated multiplying the ratio between the target value and the current value by the current number of pods.  Ergo, metrics used must decrease as the pod count is increased, and vice-versa.  See the individual metric source types for more information about how each type of metric must respond. If not set, the default metric will be set to 80% average CPU utilization.
    pub metrics: Option<Vec<MetricSpec>>,

    /// minReplicas is the lower limit for the number of replicas to which the autoscaler can scale down.  It defaults to 1 pod.  minReplicas is allowed to be 0 if the alpha feature gate `HPAScaleToZero` is enabled and at least one Object or External metric is configured.  Scaling is active as long as at least one metric value is available.
    pub min_replicas: Option<i32>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorProxySpecService {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: <http://kubernetes.io/docs/user-guide/annotations>
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: <http://kubernetes.io/docs/user-guide/labels>
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the Service.
    ///
    /// Default: name of the Service
    pub name: Option<String>,

    /// The list of ports that are exposed by this service. More info: <https://kubernetes.io/docs/concepts/services-networking/service/#virtual-ips-and-service-proxies>
    pub ports: Vec<TorProxySpecServicePort>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorProxySpecServicePort {
    /// The name of this port within the service. This must be a `DNS_LABEL`. All ports within a `ServiceSpec` must have unique names. When considering the endpoints for a Service, this must match the 'name' field in the `EndpointPort`.
    pub name: String,

    /// The port that will be exposed by this service.
    pub port: i32,

    /// The IP protocol for this port. Supports "`HTTP_TUNNEL`", "SOCKS".
    pub protocol: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorProxyStatus {
    /// Represents the latest available observations of a deployment's current state.
    ///
    /// ### Initialized
    ///
    /// `Initialized`
    ///
    /// ### Service
    /// `PortsNotFound`, `Ready`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,

    /// Hostname.
    pub hostname: Option<String>,

    /// Label selector the Horizontal Pod Autoscaler will use to collect metrics.
    #[serde(default)]
    pub label_selector: String,

    /// Number of replicas.
    pub replicas: i32,

    /// Represents the latest available observations of a deployment's current state.
    #[serde(default)]
    pub summary: BTreeMap<String, String>,
}

impl TorProxy {
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
    pub fn deployment_replicas(&self) -> i32 {
        self.spec
            .deployment
            .as_ref()
            .map_or_else(default_deployment_replicas, |f| f.replicas)
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
    pub fn horizontal_pod_autoscaler_annotations(&self) -> Option<Annotations> {
        self.spec
            .horizontal_pod_autoscaler
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .cloned()
            .map(Into::into)
    }

    #[must_use]
    pub fn horizontal_pod_autoscaler_labels(&self) -> Option<Labels> {
        self.spec
            .horizontal_pod_autoscaler
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .cloned()
            .map(Into::into)
    }

    #[must_use]
    pub fn horizontal_pod_autoscaler_name(&self) -> ResourceName {
        self.spec
            .horizontal_pod_autoscaler
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn service_annotations(&self) -> Option<Annotations> {
        self.spec.service.annotations.clone().map(Into::into)
    }

    #[must_use]
    pub fn service_labels(&self) -> Option<Labels> {
        self.spec.service.labels.clone().map(Into::into)
    }

    #[must_use]
    pub fn service_name(&self) -> ResourceName {
        self.spec
            .service
            .name
            .as_ref()
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn service_ports_http_tunnel(&self) -> Vec<&TorProxySpecServicePort> {
        self.spec
            .service
            .ports
            .iter()
            .filter(|port| port.protocol == "HTTP_TUNNEL")
            .collect()
    }

    #[must_use]
    pub fn service_ports_socks(&self) -> Vec<&TorProxySpecServicePort> {
        self.spec
            .service
            .ports
            .iter()
            .filter(|port| port.protocol == "SOCKS")
            .collect()
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

impl KubernetesResource for TorProxy {
    type Spec = TorProxySpec;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }
}

impl Object for TorProxy {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "tor-proxy";

    type Status = TorProxyStatus;

    fn status(&self) -> Option<&Self::Status> {
        self.status.as_ref()
    }
}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    TorProxy::crd()
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
    metrics.kubernetes_api_usage_count::<TorProxy>("watch");
    metrics.kubernetes_api_usage_count::<HorizontalPodAutoscaler>("watch");
    metrics.kubernetes_api_usage_count::<ConfigMap>("watch");
    metrics.kubernetes_api_usage_count::<Deployment>("watch");
    metrics.kubernetes_api_usage_count::<Service>("watch");
    Controller::new(
        kube::Api::<TorProxy>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<HorizontalPodAutoscaler>::all(client.clone()),
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
    .owns(
        kube::Api::<Service>::all(client.clone()),
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
    PortsNotFound,
    Initialized(String),
}

impl From<&State> for Vec<Condition> {
    fn from(value: &State) -> Self {
        match value {
            State::PortsNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The TorProxy service port was not found.".into(),
                observed_generation: None,
                reason: "PortsNotFound".into(),
                status: "False".into(),
                type_: "Service".into(),
            }],
            State::Initialized(_) => vec![
                Condition {
                    last_transition_time: Time(Utc::now()),
                    message: "The TorProxy service is ready.".into(),
                    observed_generation: None,
                    reason: "Ready".into(),
                    status: "True".into(),
                    type_: "Service".into(),
                },
                Condition {
                    last_transition_time: Time(Utc::now()),
                    message: "The TorProxy is initialized.".into(),
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
async fn reconciler(object: Arc<TorProxy>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(TorProxy::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let torrc = generate_torrc(&object);

    let annotations = Annotations::new().add(&torrc);
    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    let state = if object.service_ports_http_tunnel().is_empty()
        && object.service_ports_socks().is_empty()
    {
        State::PortsNotFound
    } else {
        State::Initialized(object.service_name().into())
    };

    if let State::Initialized(_) = state {
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
        )
        .await?;

        // HorizontalPodAutoscaler
        reconcile_horizontal_pod_autoscaler(
            &Api::new(
                kube::Api::namespaced(ctx.client.clone(), &namespace),
                ctx.metrics.clone(),
            ),
            &object,
            &annotations,
            &labels,
        )
        .await?;

        // Service
        reconcile_service(
            &Api::new(
                kube::Api::namespaced(ctx.client.clone(), &namespace),
                ctx.metrics.clone(),
            ),
            &object,
            &annotations,
            &labels,
            &selector_labels,
        )
        .await?;
    }

    // TorProxy
    reconcile_tor_proxy(
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
        State::Initialized(_) | State::PortsNotFound => {
            Ok(Action::requeue(Duration::from_secs(3600)))
        }
    }
}

async fn reconcile_config_map(
    api: &Api<ConfigMap>,
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
) -> Result<()> {
    api.sync(
        object,
        [((), generate_config_map(object, annotations, labels, torrc))].into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_deployment(
    api: &Api<Deployment>,
    config: &Config,
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
) -> Result<()> {
    api.sync(
        object,
        [(
            (),
            generate_deployment(object, config, annotations, labels, selector_labels),
        )]
        .into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_horizontal_pod_autoscaler(
    api: &Api<HorizontalPodAutoscaler>,
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<()> {
    let resources: HashMap<(), _> = if let Some(horizontal_pod_autoscaler) =
        generate_horizontal_pod_autoscaler(object, annotations, labels)
    {
        let mut map = HashMap::with_capacity(1);
        map.insert((), horizontal_pod_autoscaler);
        map
    } else {
        HashMap::new()
    };

    api.sync(object, resources).await.map(|_| ())
}

async fn reconcile_service(
    api: &Api<Service>,
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
) -> Result<()> {
    api.sync(
        object,
        [(
            (),
            generate_service(object, annotations, labels, selector_labels),
        )]
        .into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_tor_proxy(api: &Api<TorProxy>, object: &TorProxy, state: &State) -> Result<()> {
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
        TorProxyStatus {
            conditions,
            hostname: if let State::Initialized(hostname) = state {
                Some(hostname.clone())
            } else {
                None
            },
            label_selector: object.try_label_selector::<TorProxy>()?,
            replicas: object.deployment_replicas(),
            summary,
        },
    )
    .await
}

fn generate_torrc(object: &TorProxy) -> Torrc {
    let mut torrc = Torrc::builder();
    if let Some(template) = object.torrc_template() {
        torrc = torrc.template(template);
    }
    torrc = torrc.data_dir("${TOR_TMP_DIR}/home/.tor");
    if !object.service_ports_http_tunnel().is_empty() {
        torrc = torrc.http_tunnel_port("0.0.0.0:1080");
    }
    if !object.service_ports_socks().is_empty() {
        torrc = torrc.socks_port("0.0.0.0:9050");
    }
    torrc.build()
}

fn generate_config_map(
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
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
        data: Some(BTreeMap::from([("torrc".into(), torrc.to_string())])),
        ..Default::default()
    }
}

#[allow(clippy::too_many_lines)]
fn generate_deployment(
    object: &TorProxy,
    config: &Config,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
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
            replicas: Some(object.deployment_replicas()),
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
                    volumes: Some(generate_deployment_volumes(object)),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn generate_deployment_containers(object: &TorProxy, config: &Config) -> Vec<Container> {
    let mut containers = object.deployment_containers();

    {
        let container = vec_get_or_insert(&mut containers, |f| f.name == "tor");
        container.name = "tor".to_string();
        container.args = Some(vec![
            "-c".into(),
            [
                "export TOR_TMP_DIR=${TOR_TMP_DIR:-$(mktemp -d --suffix=.tor -p /tmp)}",
                // torrc
                "mkdir -p $TOR_TMP_DIR/usr/local/etc/tor",
                "envsubst < /etc/configs/torrc > $TOR_TMP_DIR/usr/local/etc/tor/torrc",
                // data directory
                "mkdir -p $TOR_TMP_DIR/home/.tor",
                "chmod 700 $TOR_TMP_DIR/home/.tor",
                // executable
                "tor -f $TOR_TMP_DIR/usr/local/etc/tor/torrc",
            ]
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

        let ports = container.ports.get_or_insert_with(Default::default);

        {
            let port = vec_get_or_insert(ports, |f| match &f.name {
                Some(name) => name == "http-tunnel",
                None => false,
            });
            port.name = Some("http-tunnel".to_string());
            port.container_port = 1080;
            port.protocol = Some("TCP".to_string());
        }

        {
            let port = vec_get_or_insert(ports, |f| match &f.name {
                Some(name) => name == "socks",
                None => false,
            });
            port.name = Some("socks".to_string());
            port.container_port = 9050;
            port.protocol = Some("TCP".to_string());
        }

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

fn generate_deployment_init_containers(object: &TorProxy) -> Vec<Container> {
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

fn generate_deployment_volumes(object: &TorProxy) -> Vec<Volume> {
    let mut volumes = object.deployment_volumes();

    {
        let volume = vec_get_or_insert(&mut volumes, |f| f.name == "etc-configs");
        volume.name = "etc-configs".to_string();
        volume.config_map = Some(ConfigMapVolumeSource {
            default_mode: Some(0o400),
            items: Some(vec![KeyToPath {
                key: "torrc".into(),
                mode: Some(0o400),
                path: "torrc".into(),
            }]),
            name: object.config_map_name().into(),
            optional: Some(false),
        });
    }

    volumes
}

fn generate_horizontal_pod_autoscaler(
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
) -> Option<HorizontalPodAutoscaler> {
    object
        .spec
        .horizontal_pod_autoscaler
        .as_ref()
        .map(|horizontal_pod_autoscaler| HorizontalPodAutoscaler {
            metadata: ObjectMeta {
                name: Some(object.horizontal_pod_autoscaler_name().into()),
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.horizontal_pod_autoscaler_annotations())
                        .into(),
                ),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.horizontal_pod_autoscaler_labels())
                        .into(),
                ),
                owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                behavior: horizontal_pod_autoscaler.behavior.clone(),
                max_replicas: horizontal_pod_autoscaler.max_replicas,
                metrics: horizontal_pod_autoscaler.metrics.clone(),
                min_replicas: horizontal_pod_autoscaler.min_replicas,
                scale_target_ref: CrossVersionObjectReference {
                    api_version: Some(TorProxy::api_version(&()).into()),
                    kind: TorProxy::kind(&()).into(),
                    name: object.try_name().unwrap().into(),
                },
            }),
            ..Default::default()
        })
}

fn generate_service(
    object: &TorProxy,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
) -> Service {
    Service {
        metadata: ObjectMeta {
            name: Some(object.service_name().into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.service_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.service_labels())
                    .into(),
            ),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            ports: Some({
                let mut vec = Vec::new();
                vec.append(
                    &mut object
                        .service_ports_http_tunnel()
                        .into_iter()
                        .map(|service_port| ServicePort {
                            name: Some(service_port.name.clone()),
                            port: service_port.port,
                            protocol: Some("TCP".to_string()),
                            target_port: Some(IntOrString::String("http-tunnel".to_string())),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>(),
                );
                vec.append(
                    &mut object
                        .service_ports_socks()
                        .into_iter()
                        .map(|service_port| ServicePort {
                            name: Some(service_port.name.clone()),
                            port: service_port.port,
                            protocol: Some("TCP".to_string()),
                            target_port: Some(IntOrString::String("socks".to_string())),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>(),
                );
                vec
            }),
            selector: Some(selector_labels.into()),
            type_: Some("ClusterIP".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config() {
        let object = &TorProxy {
            spec: TorProxySpec {
                service: TorProxySpecService {
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let torrc = generate_torrc(object);

        assert_eq!(r"DataDirectory ${TOR_TMP_DIR}/home/.tor", torrc.to_string());
    }

    #[test]
    fn config_http_tunnel() {
        let object = &TorProxy {
            spec: TorProxySpec {
                service: TorProxySpecService {
                    ports: vec![TorProxySpecServicePort {
                        name: "http-tunnel".to_string(),
                        port: 1080,
                        protocol: "HTTP_TUNNEL".to_string(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let torrc = generate_torrc(object);

        assert_eq!(
            r"DataDirectory ${TOR_TMP_DIR}/home/.tor
HTTPTunnelPort 0.0.0.0:1080",
            torrc.to_string()
        );
    }

    #[test]
    fn config_socks() {
        let object = &TorProxy {
            spec: TorProxySpec {
                service: TorProxySpecService {
                    ports: vec![TorProxySpecServicePort {
                        name: "socks".to_string(),
                        port: 9050,
                        protocol: "SOCKS".to_string(),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let torrc = generate_torrc(object);

        assert_eq!(
            r"DataDirectory ${TOR_TMP_DIR}/home/.tor
SocksPort 0.0.0.0:9050",
            torrc.to_string()
        );
    }

    #[test]
    fn config_http_tunnel_socks() {
        let object = &TorProxy {
            spec: TorProxySpec {
                service: TorProxySpecService {
                    ports: vec![
                        TorProxySpecServicePort {
                            name: "http-tunnel".to_string(),
                            port: 1080,
                            protocol: "HTTP_TUNNEL".to_string(),
                        },
                        TorProxySpecServicePort {
                            name: "socks".to_string(),
                            port: 9050,
                            protocol: "SOCKS".to_string(),
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let torrc = generate_torrc(object);

        assert_eq!(
            r"DataDirectory ${TOR_TMP_DIR}/home/.tor
HTTPTunnelPort 0.0.0.0:1080
SocksPort 0.0.0.0:9050",
            torrc.to_string()
        );
    }
}
