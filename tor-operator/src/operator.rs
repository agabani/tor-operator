#[derive(
    Clone, Debug, kube::CustomResource, schemars::JsonSchema, serde::Deserialize, serde::Serialize,
)]
#[kube(
    kind = "TorHiddenService",
    group = "tor-operator.agabani",
    version = "v1",
    namespaced,
    status = "TorHiddenServiceStatus"
)]
pub struct TorHiddenServiceSpec {
    pub target_address: String,
    pub target_port: u16,
    pub virtual_port: u16,
}

#[derive(Clone, Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct TorHiddenServiceStatus {}
