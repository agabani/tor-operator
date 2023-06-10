#![allow(dead_code)]
#![allow(unused_variables)]

use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/*
 * ============================================================================
 * Custom Resource Definition
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
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    Onionbalance::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {}

/*
 * ============================================================================
 * Run
 * ============================================================================
 */
#[allow(clippy::missing_panics_doc)]
pub async fn run_controller(config: Config) {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<Onionbalance>::all(client.clone());

    let context = Arc::new(Context { client, config });

    Controller::new(onion_services, WatcherConfig::default())
        .shutdown_on_signal()
        .run(reconciler, error_policy, context)
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
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<Onionbalance>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
}

/*
 * ============================================================================
 * Error Policy
 * ============================================================================
 */
#[allow(clippy::needless_pass_by_value, unused_variables)]
#[tracing::instrument(skip(object, ctx))]
fn error_policy(object: Arc<Onionbalance>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
