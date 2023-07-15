use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        autoscaling::v2::{
            CrossVersionObjectReference, HorizontalPodAutoscaler, HorizontalPodAutoscalerBehavior,
            HorizontalPodAutoscalerSpec, MetricSpec,
        },
        core::v1::{
            Affinity, LocalObjectReference, ResourceRequirements, Toleration,
            TopologySpreadConstraint,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::apis::meta::v1::{Condition, Time},
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
        self, error_policy, Annotations, Api, ConditionsExt, Labels, Object,
        Resource as KubernetesResource, ResourceName,
    },
    metrics::Metrics,
    onion_balance::{
        OnionBalance, OnionBalanceSpec, OnionBalanceSpecConfigMap, OnionBalanceSpecDeployment,
        OnionBalanceSpecDeploymentContainers, OnionBalanceSpecDeploymentContainersOnionBalance,
        OnionBalanceSpecDeploymentContainersTor, OnionBalanceSpecOnionKey,
        OnionBalanceSpecOnionService, OnionBalanceSpecOnionServiceOnionKey,
    },
    onion_key::{OnionKey, OnionKeySpec, OnionKeySpecSecret},
    onion_service::{
        OnionService, OnionServiceSpec, OnionServiceSpecConfigMap, OnionServiceSpecDeployment,
        OnionServiceSpecDeploymentContainers, OnionServiceSpecDeploymentContainersTor,
        OnionServiceSpecHiddenServicePort, OnionServiceSpecOnionBalance,
        OnionServiceSpecOnionBalanceOnionKey, OnionServiceSpecOnionKey,
    },
    Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # `TorIngress`
///
/// A `TorIngress` is collection of `OnionServices` load balanced by a `OnionBalance`.
///
/// The user must provide the `OnionKey` for the `OnionBalance`.
///
/// The Tor Operator will auto generate random `OnionKeys` for the `OnionServices`.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[kube(
    derive = "Default",
    derive = "PartialEq",
    group = "tor.agabani.co.uk",
    kind = "TorIngress",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the tor ingress", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"Replicas", "type":"number", "description":"Number of replicas", "jsonPath":".status.replicas"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.state"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    scale = r#"{"specReplicasPath":".spec.onionService.replicas", "statusReplicasPath":".status.replicas", "labelSelectorPath":".status.labelSelector"}"#,
    status = "TorIngressStatus",
    version = "v1"
)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpec {
    /// HorizontalPodAutoscaler settings.
    pub horizontal_pod_autoscaler: Option<TorIngressHorizontalPodAutoscaler>,

    /// OnionBalance settings.
    pub onion_balance: TorIngressSpecOnionBalance,

    /// OnionService settings.
    pub onion_service: TorIngressSpecOnionService,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressHorizontalPodAutoscaler {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// behavior configures the scaling behavior of the target in both Up and Down directions (scaleUp and scaleDown fields respectively). If not set, the default HPAScalingRules for scale up and scale down are used.
    pub behavior: Option<HorizontalPodAutoscalerBehavior>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the HorizontalPodAutoscaler.
    ///
    /// Default: name of the TorIngress
    pub name: Option<String>,

    /// maxReplicas is the upper limit for the number of replicas to which the autoscaler can scale up. It cannot be less that minReplicas.
    pub max_replicas: i32,

    /// metrics contains the specifications for which to use to calculate the desired replica count (the maximum replica count across all metrics will be used).  The desired replica count is calculated multiplying the ratio between the target value and the current value by the current number of pods.  Ergo, metrics used must decrease as the pod count is increased, and vice-versa.  See the individual metric source types for more information about how each type of metric must respond. If not set, the default metric will be set to 80% average CPU utilization.
    pub metrics: Option<Vec<MetricSpec>>,

    /// minReplicas is the lower limit for the number of replicas to which the autoscaler can scale down.  It defaults to 1 pod.  minReplicas is allowed to be 0 if the alpha feature gate HPAScaleToZero is enabled and at least one Object or External metric is configured.  Scaling is active as long as at least one metric value is available.
    pub min_replicas: Option<i32>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalance {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Config Map settings.
    pub config_map: Option<TorIngressSpecOnionBalanceConfigMap>,

    /// Deployment settings.
    pub deployment: Option<TorIngressSpecOnionBalanceDeployment>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the OnionBalance.
    ///
    /// Default: name of the TorIngress
    pub name: Option<String>,

    /// OnionKey settings.
    pub onion_key: TorIngressSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceConfigMap {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the Config Map.
    ///
    /// Default: name of the TorIngress
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceDeployment {
    /// If specified, the pod's scheduling constraints
    pub affinity: Option<Affinity>,

    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Containers of the Deployment.
    pub containers: Option<TorIngressSpecOnionBalanceDeploymentContainers>,

    /// ImagePullSecrets is an optional list of references to secrets in the same namespace to use for pulling any of the images used by this PodSpec. If specified, these secrets will be passed to individual puller implementations for them to use. More info: https://kubernetes.io/docs/concepts/containers/images#specifying-imagepullsecrets-on-a-pod
    pub image_pull_secrets: Option<Vec<LocalObjectReference>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the Deployment.
    ///
    /// Default: name of the TorIngress
    pub name: Option<String>,

    /// NodeSelector is a selector which must be true for the pod to fit on a node. Selector which must match a node's labels for the pod to be scheduled on that node. More info: https://kubernetes.io/docs/concepts/configuration/assign-pod-node/
    pub node_selector: Option<std::collections::BTreeMap<String, String>>,

    /// If specified, the pod's tolerations.
    pub tolerations: Option<Vec<Toleration>>,

    /// TopologySpreadConstraints describes how a group of pods ought to spread across topology domains. Scheduler will schedule pods in a way which abides by the constraints. All topologySpreadConstraints are ANDed.
    pub topology_spread_constraints: Option<Vec<TopologySpreadConstraint>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceDeploymentContainers {
    /// Onion Balance container.
    pub onion_balance: Option<TorIngressSpecOnionBalanceDeploymentContainersOnionBalance>,

    /// Tor container.
    pub tor: Option<TorIngressSpecOnionBalanceDeploymentContainersTor>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceDeploymentContainersOnionBalance {
    /// Resources of the container.
    pub resources: Option<ResourceRequirements>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceDeploymentContainersTor {
    /// Resources of the container.
    pub resources: Option<ResourceRequirements>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionBalanceOnionKey {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name of the OnionKey.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionService {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Config Map settings.
    pub config_map: Option<TorIngressSpecOnionServiceConfigMap>,

    /// Deployment settings.
    pub deployment: Option<TorIngressSpecOnionServiceDeployment>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name prefix of the OnionService.
    ///
    /// Default: name of the TorIngress
    pub name_prefix: Option<String>,

    /// OnionKey settings.
    pub onion_key: Option<TorIngressSpecOnionServiceOnionKey>,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<TorIngressSpecOnionServicePort>,

    /// Number of replicas.
    #[serde(default = "default_onion_service_replicas")]
    pub replicas: i32,
}

fn default_onion_service_replicas() -> i32 {
    3
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceConfigMap {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name prefix of the Config Map.
    ///
    /// Default: name of the TorIngress
    pub name_prefix: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceDeployment {
    /// If specified, the pod's scheduling constraints
    pub affinity: Option<Affinity>,

    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Containers of the Deployment.
    pub containers: Option<TorIngressSpecOnionServiceDeploymentContainers>,

    /// ImagePullSecrets is an optional list of references to secrets in the same namespace to use for pulling any of the images used by this PodSpec. If specified, these secrets will be passed to individual puller implementations for them to use. More info: https://kubernetes.io/docs/concepts/containers/images#specifying-imagepullsecrets-on-a-pod
    pub image_pull_secrets: Option<Vec<LocalObjectReference>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name prefix of the Deployment.
    ///
    /// Default: name of the TorIngress
    pub name_prefix: Option<String>,

    /// NodeSelector is a selector which must be true for the pod to fit on a node. Selector which must match a node's labels for the pod to be scheduled on that node. More info: https://kubernetes.io/docs/concepts/configuration/assign-pod-node/
    pub node_selector: Option<std::collections::BTreeMap<String, String>>,

    /// If specified, the pod's tolerations.
    pub tolerations: Option<Vec<Toleration>>,

    /// TopologySpreadConstraints describes how a group of pods ought to spread across topology domains. Scheduler will schedule pods in a way which abides by the constraints. All topologySpreadConstraints are ANDed.
    pub topology_spread_constraints: Option<Vec<TopologySpreadConstraint>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceDeploymentContainers {
    /// Tor container.
    pub tor: Option<TorIngressSpecOnionServiceDeploymentContainersTor>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceDeploymentContainersTor {
    /// Resources of the container.
    pub resources: Option<ResourceRequirements>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceOnionKey {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name prefix of the OnionKey.
    ///
    /// Default: name of the TorIngress
    pub name_prefix: Option<String>,

    /// Secret settings.
    pub secret: Option<TorIngressSpecOnionServiceOnionKeySecret>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServiceOnionKeySecret {
    /// Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: http://kubernetes.io/docs/user-guide/annotations
    pub annotations: Option<BTreeMap<String, String>>,

    /// Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: http://kubernetes.io/docs/user-guide/labels
    pub labels: Option<BTreeMap<String, String>>,

    /// Name prefix of the Secret.
    ///
    /// Default: name of the TorIngress
    pub name_prefix: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressSpecOnionServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorIngressStatus {
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

    /// Label selector the Horizontal Pod Autoscaler will use to collect metrics.
    #[serde(default)]
    pub label_selector: String,

    /// Number of replicas.
    pub replicas: i32,
}

impl TorIngress {
    #[must_use]
    fn default_name(&self) -> ResourceName {
        self.try_name().unwrap()
    }

    #[must_use]
    pub fn horizontal_pod_autoscaler_annotations(&self) -> Option<Annotations> {
        self.spec
            .horizontal_pod_autoscaler
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn horizontal_pod_autoscaler_labels(&self) -> Option<Labels> {
        self.spec
            .horizontal_pod_autoscaler
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
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
    pub fn onion_balance_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_balance
            .annotations
            .as_ref()
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_config_map_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_balance
            .config_map
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_config_map_labels(&self) -> Option<Labels> {
        self.spec
            .onion_balance
            .config_map
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_config_map_name(&self) -> ResourceName {
        self.spec
            .onion_balance
            .config_map
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_balance_deployment_affinity(&self) -> Option<Affinity> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.affinity.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_balance_deployment_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_deployment_containers_onion_balance_resources(
        &self,
    ) -> Option<&ResourceRequirements> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .and_then(|f| f.onion_balance.as_ref())
            .and_then(|f| f.resources.as_ref())
    }

    #[must_use]
    pub fn onion_balance_deployment_containers_tor_resources(
        &self,
    ) -> Option<&ResourceRequirements> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .and_then(|f| f.tor.as_ref())
            .and_then(|f| f.resources.as_ref())
    }

    #[must_use]
    pub fn onion_balance_deployment_image_pull_secrets(&self) -> Option<Vec<LocalObjectReference>> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.image_pull_secrets.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_balance_deployment_labels(&self) -> Option<Labels> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_deployment_name(&self) -> ResourceName {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_balance_deployment_node_selector(&self) -> Option<BTreeMap<String, String>> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.node_selector.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_balance_deployment_tolerations(&self) -> Option<Vec<Toleration>> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.tolerations.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_balance_deployment_topology_spread_constraints(
        &self,
    ) -> Option<Vec<TopologySpreadConstraint>> {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.topology_spread_constraints.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_balance_labels(&self) -> Option<Labels> {
        self.spec
            .onion_balance
            .labels
            .as_ref()
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_balance_name(&self) -> ResourceName {
        self.spec
            .onion_balance
            .name
            .as_ref()
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_balance_onion_key_name(&self) -> ResourceName {
        ResourceName::from(&self.spec.onion_balance.onion_key.name)
    }

    #[must_use]
    pub fn onion_service_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_service
            .annotations
            .as_ref()
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_config_map_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_service
            .config_map
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_config_map_labels(&self) -> Option<Labels> {
        self.spec
            .onion_service
            .config_map
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_config_map_name(&self, instance: i32) -> ResourceName {
        format!("{}-{instance}", self.onion_service_config_map_name_prefix()).into()
    }

    #[must_use]
    pub fn onion_service_config_map_name_prefix(&self) -> ResourceName {
        self.spec
            .onion_service
            .config_map
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_service_deployment_affinity(&self) -> Option<Affinity> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.affinity.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_service_deployment_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_deployment_containers_tor_resources(
        &self,
    ) -> Option<&ResourceRequirements> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.containers.as_ref())
            .and_then(|f| f.tor.as_ref())
            .and_then(|f| f.resources.as_ref())
    }

    #[must_use]
    pub fn onion_service_deployment_image_pull_secrets(&self) -> Option<Vec<LocalObjectReference>> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.image_pull_secrets.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_service_deployment_labels(&self) -> Option<Labels> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_deployment_name(&self, instance: i32) -> ResourceName {
        format!("{}-{instance}", self.onion_service_deployment_name_prefix()).into()
    }

    #[must_use]
    pub fn onion_service_deployment_name_prefix(&self) -> ResourceName {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_service_deployment_node_selector(&self) -> Option<BTreeMap<String, String>> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.node_selector.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_service_deployment_tolerations(&self) -> Option<Vec<Toleration>> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.tolerations.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_service_deployment_topology_spread_constraints(
        &self,
    ) -> Option<Vec<TopologySpreadConstraint>> {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.topology_spread_constraints.as_ref())
            .map(Clone::clone)
    }

    #[must_use]
    pub fn onion_service_labels(&self) -> Option<Labels> {
        self.spec
            .onion_service
            .labels
            .as_ref()
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_name(&self, instance: i32) -> ResourceName {
        format!("{}-{instance}", self.onion_service_name_prefix()).into()
    }

    #[must_use]
    pub fn onion_service_name_prefix(&self) -> ResourceName {
        self.spec
            .onion_service
            .name_prefix
            .as_ref()
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_name(&self, instance: i32) -> ResourceName {
        format!("{}-{instance}", self.onion_service_onion_key_name_prefix()).into()
    }

    #[must_use]
    pub fn onion_service_onion_key_name_prefix(&self) -> ResourceName {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_name_prefix(&self) -> ResourceName {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.secret.as_ref())
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_labels(&self) -> Option<Labels> {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_annotations(&self) -> Option<Annotations> {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.secret.as_ref())
            .and_then(|f| f.annotations.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_labels(&self) -> Option<Labels> {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.secret.as_ref())
            .and_then(|f| f.labels.as_ref())
            .map(Clone::clone)
            .map(Into::into)
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_name(&self, instance: i32) -> ResourceName {
        format!(
            "{}-{instance}",
            self.onion_service_onion_key_secret_name_prefix()
        )
        .into()
    }

    #[must_use]
    pub fn onion_service_replicas(&self) -> i32 {
        self.spec.onion_service.replicas
    }

    #[must_use]
    pub fn status_conditions(&self) -> Option<&Vec<Condition>> {
        self.status.as_ref().map(|f| f.conditions.as_ref())
    }
}

impl KubernetesResource for TorIngress {
    type Spec = TorIngressSpec;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }
}

impl Object for TorIngress {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "tor-ingress";

    type Status = TorIngressStatus;

    fn status(&self) -> &Option<Self::Status> {
        &self.status
    }
}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    TorIngress::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {}

/*
 * ============================================================================
 * Controller
 * ============================================================================
 */
pub async fn run_controller(client: Client, config: Config, metrics: Metrics) {
    Metrics::kubernetes_api_usage_count::<TorIngress>("watch");
    Metrics::kubernetes_api_usage_count::<HorizontalPodAutoscaler>("watch");
    Metrics::kubernetes_api_usage_count::<OnionBalance>("watch");
    Metrics::kubernetes_api_usage_count::<OnionKey>("watch");
    Metrics::kubernetes_api_usage_count::<OnionService>("watch");
    Controller::new(
        kube::Api::<TorIngress>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<HorizontalPodAutoscaler>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<OnionBalance>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<OnionKey>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<OnionService>::all(client.clone()),
        WatcherConfig::default(),
    )
    .shutdown_on_signal()
    .run(
        reconciler,
        error_policy,
        Arc::new(Context {
            client,
            _config: config,
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
    _config: Config,
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
    OnionBalanceOnionKeyNotFound,
    OnionBalanceOnionKeyHostnameNotFound,
    OnionServiceOnionKeyHostnameNotFound,
    Initialized((OnionKey, HashMap<i32, OnionKey>)),
}

impl From<&State> for Vec<Condition> {
    fn from(value: &State) -> Self {
        match value {
            State::OnionBalanceOnionKeyNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The OnionBalance OnionKey was not found.".into(),
                observed_generation: None,
                reason: "NotFound".into(),
                status: "False".into(),
                type_: "OnionKey".into(),
            }],
            State::OnionBalanceOnionKeyHostnameNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The OnionBalance OnionKey does not have a hostname.".into(),
                observed_generation: None,
                reason: "HostnameNotFound".into(),
                status: "False".into(),
                type_: "OnionKey".into(),
            }],
            State::OnionServiceOnionKeyHostnameNotFound => vec![Condition {
                last_transition_time: Time(Utc::now()),
                message: "The OnionService OnionKey does not have a hostname.".into(),
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
                    message: "The TorIngress is initialized.".into(),
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
async fn reconciler(object: Arc<TorIngress>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(TorIngress::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let annotations = Annotations::new();
    let labels = object.try_labels()?;

    // OnionKey
    let state = reconcile_onion_key(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
        &annotations,
        &labels,
    )
    .await?;

    if let State::Initialized((onion_balance_onion_key, onion_service_onion_keys)) = &state {
        // OnionServices
        reconcile_onion_services(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
            onion_balance_onion_key,
        )
        .await?;

        // OnionBalance
        reconcile_onion_balance(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
            onion_service_onion_keys,
        )
        .await?;

        // HorizontalPodAutoscaler
        reconcile_horizontal_pod_autoscaler(
            &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
            &object,
            &annotations,
            &labels,
        )
        .await?;
    }

    // TorIngress
    reconcile_tor_ingress(
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

async fn reconcile_onion_key(
    api: &Api<OnionKey>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<State> {
    // OnionBalance
    let Some(onion_balance_onion_key) = api
        .get_opt(&object.onion_balance_onion_key_name())
        .await? else {
            return Ok(State::OnionBalanceOnionKeyNotFound)
        };

    if onion_balance_onion_key.hostname().is_none() {
        return Ok(State::OnionBalanceOnionKeyHostnameNotFound);
    }

    // OnionService: update
    let (onion_service_onion_keys, deprecated) = api
        .update(
            object,
            (0..object.onion_service_replicas())
                .map(|instance| {
                    (
                        instance,
                        generate_onion_service_onion_key(object, annotations, labels, instance),
                    )
                })
                .collect(),
        )
        .await?;

    // OnionService: ready
    let ready = onion_service_onion_keys
        .iter()
        .all(|(_, onion_key)| onion_key.hostname().is_some());

    if !ready {
        return Ok(State::OnionServiceOnionKeyHostnameNotFound);
    }

    // OnionService: deletion
    api.delete(object, deprecated).await?;

    Ok(State::Initialized((
        onion_balance_onion_key,
        onion_service_onion_keys,
    )))
}

async fn reconcile_onion_services(
    api: &Api<OnionService>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_balance_onion_key: &OnionKey,
) -> Result<()> {
    api.sync(
        object,
        (0..object.onion_service_replicas())
            .map(|instance| {
                (
                    instance,
                    generate_onion_service(
                        object,
                        annotations,
                        labels,
                        onion_balance_onion_key,
                        instance,
                    ),
                )
            })
            .collect(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_onion_balance(
    api: &Api<OnionBalance>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_service_onion_keys: &HashMap<i32, OnionKey>,
) -> Result<()> {
    api.sync(
        object,
        [(
            (),
            generate_onion_balance(object, annotations, labels, onion_service_onion_keys),
        )]
        .into(),
    )
    .await
    .map(|_| ())
}

async fn reconcile_horizontal_pod_autoscaler(
    api: &Api<HorizontalPodAutoscaler>,
    object: &TorIngress,
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

async fn reconcile_tor_ingress(
    api: &Api<TorIngress>,
    object: &TorIngress,
    state: &State,
) -> Result<()> {
    api.update_status(
        object,
        TorIngressStatus {
            conditions: object
                .status_conditions()
                .unwrap_or(&Vec::new())
                .merge_from(&state.into()),
            hostname: if let State::Initialized((onion_key, _)) = state {
                onion_key.hostname().as_ref().map(ToString::to_string)
            } else {
                None
            },
            label_selector: object.try_label_selector::<OnionService>()?,
            replicas: object.onion_service_replicas(),
        },
    )
    .await
}

fn generate_onion_balance(
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_service_onion_keys: &HashMap<i32, OnionKey>,
) -> OnionBalance {
    OnionBalance {
        metadata: ObjectMeta {
            name: Some(object.onion_balance_name().into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.onion_balance_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.onion_balance_labels())
                    .into(),
            ),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: OnionBalanceSpec {
            config_map: Some(OnionBalanceSpecConfigMap {
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.onion_balance_config_map_annotations())
                        .into(),
                ),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.onion_balance_config_map_labels())
                        .into(),
                ),
                name: Some(object.onion_balance_config_map_name().into()),
            }),
            deployment: Some(OnionBalanceSpecDeployment {
                affinity: object.onion_balance_deployment_affinity(),
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.onion_balance_deployment_annotations())
                        .into(),
                ),
                containers: Some(OnionBalanceSpecDeploymentContainers {
                    onion_balance: Some(OnionBalanceSpecDeploymentContainersOnionBalance {
                        resources: object
                            .onion_balance_deployment_containers_onion_balance_resources()
                            .cloned(),
                    }),
                    tor: Some(OnionBalanceSpecDeploymentContainersTor {
                        resources: object
                            .onion_balance_deployment_containers_tor_resources()
                            .cloned(),
                    }),
                }),
                image_pull_secrets: object.onion_balance_deployment_image_pull_secrets(),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.onion_balance_deployment_labels())
                        .into(),
                ),
                name: Some(object.onion_balance_deployment_name().into()),
                node_selector: object.onion_balance_deployment_node_selector(),
                tolerations: object.onion_balance_deployment_tolerations(),
                topology_spread_constraints: object
                    .onion_balance_deployment_topology_spread_constraints(),
            }),
            onion_key: OnionBalanceSpecOnionKey {
                name: object.onion_balance_onion_key_name().into(),
            },
            onion_services: (0..onion_service_onion_keys.len())
                .map(|instance| OnionBalanceSpecOnionService {
                    onion_key: OnionBalanceSpecOnionServiceOnionKey {
                        hostname: onion_service_onion_keys
                            .get(&i32::try_from(instance).unwrap())
                            .and_then(OnionKey::hostname)
                            .unwrap()
                            .into(),
                    },
                })
                .collect(),
        },
        status: None,
    }
}

fn generate_onion_service_onion_key(
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    instance: i32,
) -> OnionKey {
    OnionKey {
        metadata: ObjectMeta {
            name: Some(object.onion_service_onion_key_name(instance).into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.onion_service_onion_key_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.onion_service_onion_key_labels())
                    .into(),
            ),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: OnionKeySpec {
            auto_generate: true,
            secret: OnionKeySpecSecret {
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.onion_service_onion_key_secret_annotations())
                        .into(),
                ),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.onion_service_onion_key_secret_labels())
                        .into(),
                ),
                name: object.onion_service_onion_key_secret_name(instance).into(),
            },
        },
        status: None,
    }
}

fn generate_onion_service(
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_balance_onion_key: &OnionKey,
    instance: i32,
) -> OnionService {
    OnionService {
        metadata: ObjectMeta {
            name: Some(object.onion_service_name(instance).into()),
            annotations: Some(
                annotations
                    .clone()
                    .append_reverse(object.onion_service_annotations())
                    .into(),
            ),
            labels: Some(
                labels
                    .clone()
                    .append_reverse(object.onion_service_labels())
                    .into(),
            ),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: OnionServiceSpec {
            config_map: Some(OnionServiceSpecConfigMap {
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.onion_service_config_map_annotations())
                        .into(),
                ),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.onion_service_config_map_labels())
                        .into(),
                ),
                name: Some(object.onion_service_config_map_name(instance).into()),
            }),
            deployment: Some(OnionServiceSpecDeployment {
                affinity: object.onion_service_deployment_affinity(),
                annotations: Some(
                    annotations
                        .clone()
                        .append_reverse(object.onion_service_deployment_annotations())
                        .into(),
                ),
                containers: Some(OnionServiceSpecDeploymentContainers {
                    tor: Some(OnionServiceSpecDeploymentContainersTor {
                        resources: object
                            .onion_service_deployment_containers_tor_resources()
                            .cloned(),
                    }),
                }),
                image_pull_secrets: object.onion_service_deployment_image_pull_secrets(),
                labels: Some(
                    labels
                        .clone()
                        .append_reverse(object.onion_service_deployment_labels())
                        .into(),
                ),
                name: Some(object.onion_service_deployment_name(instance).into()),
                node_selector: object.onion_service_deployment_node_selector(),
                tolerations: object.onion_service_deployment_tolerations(),
                topology_spread_constraints: object
                    .onion_service_deployment_topology_spread_constraints(),
            }),
            onion_balance: Some(OnionServiceSpecOnionBalance {
                onion_key: OnionServiceSpecOnionBalanceOnionKey {
                    hostname: onion_balance_onion_key.hostname().unwrap().into(),
                },
            }),
            onion_key: OnionServiceSpecOnionKey {
                name: object.onion_service_onion_key_name(instance).into(),
            },
            ports: object
                .spec
                .onion_service
                .ports
                .iter()
                .map(|f| OnionServiceSpecHiddenServicePort {
                    target: f.target.clone(),
                    virtport: f.virtport,
                })
                .collect(),
        },
        status: None,
    }
}

fn generate_horizontal_pod_autoscaler(
    object: &TorIngress,
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
                    api_version: Some(TorIngress::api_version(&()).into()),
                    kind: TorIngress::kind(&()).into(),
                    name: object.try_name().unwrap().into(),
                },
            }),
            ..Default::default()
        })
}
