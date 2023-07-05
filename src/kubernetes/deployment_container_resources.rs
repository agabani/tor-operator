use std::collections::BTreeMap;

use k8s_openapi::{
    api::core::v1::ResourceRequirements, apimachinery::pkg::api::resource::Quantity,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DeploymentContainerResources {
    /// Limits of the Resources.
    pub limits: Option<DeploymentContainerResourcesLimits>,

    /// Requests of the Resources.
    pub requests: Option<DeploymentContainerResourcesRequests>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DeploymentContainerResourcesLimits {
    /// CPU quantity of the Limits.
    pub cpu: Option<String>,

    /// Memory quantity of the Limits.
    pub memory: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DeploymentContainerResourcesRequests {
    /// CPU quantity of the Requests.
    pub cpu: Option<String>,

    /// Memory quantity of the Requests.
    pub memory: Option<String>,
}

impl DeploymentContainerResources {
    pub fn limits_cpu(&self) -> Option<&str> {
        self.limits
            .as_ref()
            .and_then(|f| f.cpu.as_ref())
            .map(String::as_str)
    }

    pub fn limits_memory(&self) -> Option<&str> {
        self.limits
            .as_ref()
            .and_then(|f| f.memory.as_ref())
            .map(String::as_str)
    }

    pub fn requests_cpu(&self) -> Option<&str> {
        self.requests
            .as_ref()
            .and_then(|f| f.cpu.as_ref())
            .map(String::as_str)
    }

    pub fn requests_memory(&self) -> Option<&str> {
        self.requests
            .as_ref()
            .and_then(|f| f.memory.as_ref())
            .map(String::as_str)
    }
}

impl From<&DeploymentContainerResources> for ResourceRequirements {
    fn from(value: &DeploymentContainerResources) -> Self {
        ResourceRequirements {
            limits: Some({
                let mut map = BTreeMap::new();
                if let Some(quantity) = value.limits_cpu() {
                    map.insert("cpu".into(), Quantity(quantity.into()));
                }
                if let Some(quantity) = value.limits_memory() {
                    map.insert("memory".into(), Quantity(quantity.into()));
                }
                map
            }),
            requests: Some({
                let mut map = BTreeMap::new();
                if let Some(quantity) = value.requests_cpu() {
                    map.insert("cpu".into(), Quantity(quantity.into()));
                }
                if let Some(quantity) = value.requests_memory() {
                    map.insert("memory".into(), Quantity(quantity.into()));
                }
                map
            }),
            ..Default::default()
        }
    }
}
