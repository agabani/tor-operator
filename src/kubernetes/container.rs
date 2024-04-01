use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{Probe, ResourceRequirements};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ContainerPort, EnvVar, VolumeMount};

/// A single application container that you want to run within a pod.
#[derive(JsonSchema, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    /// Arguments to the entrypoint. The container image's CMD is used if this is not provided. Variable references $(VAR_NAME) are expanded using the container's environment. If a variable cannot be resolved, the reference in the input string will be unchanged. Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME) syntax: i.e. "$$(VAR_NAME)" will produce the string literal "$(VAR_NAME)". Escaped references will never be expanded, regardless of whether the variable exists or not. Cannot be updated. More info: https://kubernetes.io/docs/tasks/inject-data-application/define-command-argument-container/#running-a-command-in-a-shell
    pub args: Option<Vec<String>>,

    /// Entrypoint array. Not executed within a shell. The container image's ENTRYPOINT is used if this is not provided. Variable references $(VAR_NAME) are expanded using the container's environment. If a variable cannot be resolved, the reference in the input string will be unchanged. Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME) syntax: i.e. "$$(VAR_NAME)" will produce the string literal "$(VAR_NAME)". Escaped references will never be expanded, regardless of whether the variable exists or not. Cannot be updated. More info: https://kubernetes.io/docs/tasks/inject-data-application/define-command-argument-container/#running-a-command-in-a-shell
    pub command: Option<Vec<String>>,

    /// List of environment variables to set in the container. Cannot be updated.
    pub env: Option<BTreeMap<String, EnvVar>>,

    /// Container image name. More info: https://kubernetes.io/docs/concepts/containers/images This field is optional to allow higher level config management to default or override container images in workload controllers like Deployments and StatefulSets.
    pub image: Option<String>,

    /// Image pull policy. One of Always, Never, IfNotPresent. Defaults to Always if :latest tag is specified, or IfNotPresent otherwise. Cannot be updated. More info: https://kubernetes.io/docs/concepts/containers/images#updating-images
    pub image_pull_policy: Option<String>,

    /// Periodic probe of container liveness. Container will be restarted if the probe fails. Cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes
    pub liveness_probe: Option<Probe>,

    /// List of ports to expose from the container. Not specifying a port here DOES NOT prevent that port from being exposed. Any port which is listening on the default "0.0.0.0" address inside a container will be accessible from the network. Modifying this array with strategic merge patch may corrupt the data. For more information See https://github.com/kubernetes/kubernetes/issues/108255. Cannot be updated.
    pub ports: Option<BTreeMap<String, ContainerPort>>,

    /// Periodic probe of container service readiness. Container will be removed from service endpoints if the probe fails. Cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes
    pub readiness_probe: Option<Probe>,

    /// Compute Resources required by this container. Cannot be updated. More info: https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/
    pub resources: Option<ResourceRequirements>,

    /// StartupProbe indicates that the Pod has successfully initialized. If specified, no other probes are executed until this completes successfully. If this probe fails, the Pod will be restarted, just as if the livenessProbe failed. This can be used to provide different probe parameters at the beginning of a Pod's lifecycle, when it might take a long time to load data or warm a cache, than during steady-state operation. This cannot be updated. More info: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle#container-probes
    pub startup_probe: Option<Probe>,

    /// Pod volumes to mount into the container's filesystem. Cannot be updated.
    pub volume_mounts: Option<BTreeMap<String, VolumeMount>>,
}

impl Container {
    pub fn into_container(self, name: String) -> k8s_openapi::api::core::v1::Container {
        k8s_openapi::api::core::v1::Container {
            args: self.args,
            command: self.command,
            env: self.env.map(|f| {
                f.into_iter()
                    .map(|(name, value)| value.into_env_var(name))
                    .collect()
            }),
            image: self.image,
            image_pull_policy: self.image_pull_policy,
            liveness_probe: self.liveness_probe,
            name,
            ports: self.ports.map(|f| {
                f.into_iter()
                    .map(|(name, value)| value.into_container_port(name))
                    .collect()
            }),
            readiness_probe: self.readiness_probe,
            resources: self.resources,
            startup_probe: self.startup_probe,
            volume_mounts: self.volume_mounts.map(|f| {
                f.into_iter()
                    .map(|(name, value)| value.into_volume_mount(name))
                    .collect()
            }),
            ..Default::default()
        }
    }
}
