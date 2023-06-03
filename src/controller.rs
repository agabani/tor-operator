use std::{sync::Arc, time::Duration};

use futures::{future, StreamExt};
use kube::{
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Client,
};

use crate::crd::OnionService;

#[allow(clippy::missing_panics_doc)]
pub async fn run() {
    let client = Client::try_default().await.unwrap();
    let onion_services = Api::<OnionService>::all(client);
    let context = Arc::new(Context {});

    Controller::new(onion_services, Config::default())
        .run(reconciler, error_policy, context)
        .for_each(|_| future::ready(()))
        .await;
}

/*
 * ============================================================================
 * Error
 * ============================================================================
 */
#[derive(Debug)]
enum Error {}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/*
 * ============================================================================
 * Context
 * ============================================================================
 */
struct Context {}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(_object, _ctx))]
async fn reconciler(_object: Arc<OnionService>, _ctx: Arc<Context>) -> Result<Action, Error> {
    tracing::info!("reconcile request");
    Ok(Action::requeue(Duration::from_secs(3600)))
}

/*
 * ============================================================================
 * Error Policy
 * ============================================================================
 */
#[allow(clippy::needless_pass_by_value)]
fn error_policy(_object: Arc<OnionService>, _error: &Error, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
