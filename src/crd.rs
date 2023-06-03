use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{CustomResource, CustomResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "agabani.co.uk",
    kind = "Tor",
    status = "TorStatus",
    version = "v1"
)]
pub struct TorSpec {}

#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorStatus {}

#[must_use]
pub fn generate() -> CustomResourceDefinition {
    Tor::crd()
}
