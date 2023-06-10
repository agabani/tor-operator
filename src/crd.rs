use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{CustomResource, CustomResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/*
 * ============================================================================
 * Onionbalance
 * ============================================================================
 */
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "Onionbalance",
    namespaced,
    status = "OnionbalanceStatus",
    version = "v1"
)]
pub struct OnionbalanceSpec {}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionbalanceStatus {}

#[must_use]
pub fn generate_onionbalance() -> CustomResourceDefinition {
    Onionbalance::crd()
}

/*
 * ============================================================================
 * Onion Service
 * ============================================================================
 */
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    status = "OnionServiceStatus",
    version = "v1"
)]
pub struct OnionServiceSpec {
    pub hidden_service_ports: Vec<OnionServiceSpecHiddenServicePort>,

    pub secret_name: String,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceSpecHiddenServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceStatus {}

#[must_use]
pub fn generate_onion_service() -> CustomResourceDefinition {
    OnionService::crd()
}
