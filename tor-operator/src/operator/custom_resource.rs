use kube::api::{Meta, Patch, PatchParams};
use kube::Api;
use serde_json::{json, Value};

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

pub async fn add_finalizer(api: Api<TorHiddenService>, tor_hidden_service: &TorHiddenService) {
    let value: Value = json!({
        "metadata": {
            "finalizers": ["tor.daemon.deployment"]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&value);
    api.patch(&tor_hidden_service.name(), &PatchParams::default(), &patch)
        .await
        .unwrap();
}

pub async fn remove_finalizer(api: Api<TorHiddenService>, tor_hidden_service: &TorHiddenService) {
    let value: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&value);
    api.patch(&tor_hidden_service.name(), &PatchParams::default(), &patch)
        .await
        .unwrap();
}
