mod custom_resource;
mod daemon_deployment;

use crate::operator::custom_resource::{add_finalizer, remove_finalizer};
use crate::operator::daemon_deployment::{create_or_update, destroy, DaemonDeployment};
pub use custom_resource::TorHiddenService;
use custom_resource::TorHiddenServiceSpec;
use futures::{future, FutureExt, StreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use kube::api::{ListParams, Meta};
use kube::{Api, Client};
use kube_runtime::controller::{Context, ReconcilerAction};
use kube_runtime::Controller;
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct Operator {}

impl Operator {
    pub async fn new(client: Client) -> (Self, Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        let context = Context::new(State {
            client: client.clone(),
        });

        let api = Api::<TorHiddenService>::all(client);

        let drainer = Controller::new(api, ListParams::default())
            .run(reconcile, error_policy, context)
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|o| {
                println!("Reconciled {:?}", o);
                future::ready(())
            })
            .boxed();

        (Self {}, drainer)
    }
}

struct State {
    pub client: Client,
}

#[derive(Debug)]
struct OperatorError {}

impl std::error::Error for OperatorError {}

impl std::fmt::Display for OperatorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::Error> for OperatorError {
    fn from(e: serde_json::Error) -> Self {
        panic!("{:?}", e);
    }
}

impl From<kube::Error> for OperatorError {
    fn from(e: kube::Error) -> Self {
        panic!("{:?}", e);
    }
}

async fn reconcile(
    tor_hidden_service: TorHiddenService,
    ctx: Context<State>,
) -> Result<ReconcilerAction, OperatorError> {
    // logging.
    println!("Reconcile TorHiddenService {:?}", &tor_hidden_service);

    // metadata
    let name = tor_hidden_service.name();
    let namespace: &str = tor_hidden_service.metadata.namespace.as_ref().unwrap();
    let spec: &TorHiddenServiceSpec = &tor_hidden_service.spec;

    // prepare data
    let daemon_deployment = DaemonDeployment {
        name,
        virtual_port: spec.virtual_port,
        target_address: spec.target_address.to_owned(),
        target_port: spec.target_port,
    };

    // apis
    let deployments: Api<Deployment> = Api::namespaced(ctx.get_ref().client.clone(), namespace);
    let tor_hidden_services: Api<TorHiddenService> =
        Api::namespaced(ctx.get_ref().client.clone(), namespace);

    match &tor_hidden_service.metadata.deletion_timestamp {
        None => {
            add_finalizer(tor_hidden_services, &tor_hidden_service).await;
            create_or_update(deployments, &daemon_deployment).await;
        }
        Some(_) => {
            destroy(deployments, &daemon_deployment).await;
            remove_finalizer(tor_hidden_services, &tor_hidden_service).await;
        }
    }

    Ok(ReconcilerAction {
        requeue_after: Some(std::time::Duration::from_secs(1800)),
    })
}

fn error_policy(error: &OperatorError, _ctx: Context<State>) -> ReconcilerAction {
    println!("Reconcile failed: {}", error);

    ReconcilerAction {
        requeue_after: Some(std::time::Duration::from_secs(360)),
    }
}
