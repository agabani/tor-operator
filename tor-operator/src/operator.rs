use futures::{future, FutureExt, StreamExt};
use kube::api::ListParams;
use kube::{Api, Client};
use kube_runtime::controller::{Context, ReconcilerAction};
use kube_runtime::Controller;
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;

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

async fn reconcile(
    tor_hidden_service: TorHiddenService,
    _ctx: Context<State>,
) -> Result<ReconcilerAction, OperatorError> {
    println!("Reconcile TorHiddenService {:?}", tor_hidden_service);

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
