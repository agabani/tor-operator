use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{CustomResource, CustomResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    status = "OnionServiceStatus",
    version = "v1"
)]
pub struct OnionServiceSpec {}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceStatus {}

#[must_use]
pub fn generate_onion_service() -> CustomResourceDefinition {
    OnionService::crd()
}
