use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    onion_key::{OnionKey, OnionKeySpec, OnionKeySpecSecret},
    Error, Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "TorIngress",
    namespaced,
    status = "TorIngressStatus",
    version = "v1"
)]
pub struct TorIngressSpec {
    pub onion_balance: TorIngressSpecOnionBalance,

    pub onion_service: TorIngressSpecOnionService,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionBalance {
    pub onion_key: TorIngressSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionBalanceOnionKey {
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionService {
    pub onion_key: TorIngressSpecOnionServiceOnionKey,

    pub ports: Vec<TorIngressSpecOnionServicePort>,

    pub replicas: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionServiceOnionKey {
    pub name_prefix: String,

    pub secret: TorIngressSpecOnionServiceOnionKeySecret,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionServiceOnionKeySecret {
    pub name_prefix: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressSpecOnionServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressStatus {}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    TorIngress::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {}

/*
 * ============================================================================
 * Controller
 * ============================================================================
 */
#[allow(clippy::missing_panics_doc)]
pub async fn run_controller(config: Config) {
    let client = Client::try_default().await.unwrap();

    let onion_keys = Api::<OnionKey>::all(client.clone());
    let tor_ingresses = Api::<TorIngress>::all(client.clone());

    let context = Arc::new(Context {
        client,
        _config: config,
    });

    Controller::new(tor_ingresses, WatcherConfig::default())
        .owns(onion_keys, WatcherConfig::default())
        .shutdown_on_signal()
        .run(reconciler, error_policy, context)
        .for_each(|_| async {})
        .await;
}

/*
 * ============================================================================
 * Constants
 * ============================================================================
 */
const APP_KUBERNETES_IO_COMPONENT: &str = "tor-ingress";
const APP_KUBERNETES_IO_NAME: &str = "tor";
const APP_KUBERNETES_IO_MANAGED_BY: &str = "tor-operator";

/*
 * ============================================================================
 * Types
 * ============================================================================
 */
struct Annotations(BTreeMap<String, String>);
struct Hostname(String);
struct Labels(BTreeMap<String, String>);
struct ObjectName<'a>(&'a str);
struct ObjectNamespace<'a>(&'a str);
#[derive(PartialEq, Eq, Hash)]
struct OnionKeyName(String);
struct OnionKeySecretName(String);

/*
 * ============================================================================
 * Context
 * ============================================================================
 */
struct Context {
    client: Client,
    _config: Config,
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<TorIngress>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let object_name = get_object_name(&object)?;
    let object_namespace = get_object_namespace(&object)?;

    let annotations = generate_annotations();
    let labels = generate_labels(&object, &object_name);

    let results =
        reconcile_onion_keys(&object, &ctx, &object_namespace, &annotations, &labels).await?;

    let hostnames = results.iter().map(|(_, f)| &f.0).collect::<Vec<_>>();

    tracing::info!(hostnames =? hostnames, "hostnames");

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
}

async fn reconcile_onion_keys(
    object: &TorIngress,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<Vec<(i32, Hostname)>> {
    let onion_keys = Api::<OnionKey>::namespaced(ctx.client.clone(), object_namespace.0);

    let manifest = (0..object.spec.onion_service.replicas)
        .map(|f| {
            (
                generate_onion_key_name(object, f),
                (f, generate_onion_key_secret_name(object, f)),
            )
        })
        .collect::<HashMap<_, _>>();

    for (onion_key_name, (_, onion_key_secret_name)) in &manifest {
        let onion_key = onion_keys
            .get_opt(&onion_key_name.0)
            .await
            .map_err(Error::Kube)?;

        let onion_key = generate_onion_key(
            object,
            &onion_key,
            annotations,
            labels,
            onion_key_name,
            onion_key_secret_name,
        );

        if let Some(onion_key) = onion_key {
            onion_keys
                .patch(
                    &onion_key_name.0,
                    &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY),
                    &Patch::Apply(&onion_key),
                )
                .await
                .map_err(Error::Kube)?;
        }
    }

    let mut hostnames = Vec::new();

    let owned_onion_keys = onion_keys
        .list(&ListParams::default().labels(&format!(
            "tor.agabani.co.uk/owned-by={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    for onion_key in owned_onion_keys {
        let onion_key_name = OnionKeyName(onion_key.metadata.name.clone().unwrap());

        if let Some((i, _)) = manifest.get(&onion_key_name) {
            let hostname = onion_key.status.and_then(|f| f.hostname).map(Hostname);

            if let Some(hostname) = hostname {
                hostnames.push((*i, hostname));
            }
        } else {
            onion_keys
                .delete(&onion_key_name.0, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(hostnames)
}

fn get_object_name(object: &TorIngress) -> Result<ObjectName> {
    Ok(ObjectName(
        object
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
            .as_str(),
    ))
}

fn get_object_namespace(object: &TorIngress) -> Result<ObjectNamespace> {
    Ok(ObjectNamespace(
        object
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))?
            .as_str(),
    ))
}

fn generate_annotations() -> Annotations {
    Annotations(BTreeMap::from([]))
}

fn generate_labels(object: &TorIngress, object_name: &ObjectName) -> Labels {
    Labels(BTreeMap::from([
        (
            "app.kubernetes.io/component".into(),
            APP_KUBERNETES_IO_COMPONENT.into(),
        ),
        ("app.kubernetes.io/instance".into(), object_name.0.into()),
        (
            "app.kubernetes.io/managed-by".into(),
            APP_KUBERNETES_IO_MANAGED_BY.into(),
        ),
        (
            "app.kubernetes.io/name".into(),
            APP_KUBERNETES_IO_NAME.into(),
        ),
        (
            "tor.agabani.co.uk/owned-by".into(),
            object.metadata.uid.clone().unwrap(),
        ),
    ]))
}

fn generate_onion_key_name(object: &TorIngress, instance: i32) -> OnionKeyName {
    OnionKeyName(format!(
        "{}-{}",
        object.spec.onion_service.onion_key.name_prefix, instance
    ))
}

fn generate_onion_key_secret_name(object: &TorIngress, instance: i32) -> OnionKeySecretName {
    OnionKeySecretName(format!(
        "{}-{}",
        object.spec.onion_service.onion_key.secret.name_prefix, instance
    ))
}

// only returns an onion key if a change needs to be made...
fn generate_onion_key(
    object: &TorIngress,
    onion_key: &Option<OnionKey>,
    annotations: &Annotations,
    labels: &Labels,
    onion_key_name: &OnionKeyName,
    onion_key_secret_name: &OnionKeySecretName,
) -> Option<OnionKey> {
    let Some(onion_key) = onion_key else {
        return Some(generate_owned_onion_key(
            object,
            annotations,
            labels,
            onion_key_name,
            onion_key_secret_name
        ));
    };

    if !onion_key.spec.auto_generate()
        || onion_key.metadata.name.as_ref().unwrap() != &onion_key_name.0
        || onion_key.spec.secret.name != onion_key_secret_name.0
    {
        return Some(generate_owned_onion_key(
            object,
            annotations,
            labels,
            onion_key_name,
            onion_key_secret_name,
        ));
    }

    None
}

fn generate_owned_onion_key(
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_key_name: &OnionKeyName,
    onion_key_secret_name: &OnionKeySecretName,
) -> OnionKey {
    OnionKey {
        metadata: ObjectMeta {
            name: Some(onion_key_name.0.clone()),
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: OnionKeySpec {
            auto_generate: Some(true),
            secret: OnionKeySpecSecret {
                name: onion_key_secret_name.0.clone(),
            },
        },
        status: None,
    }
}

/*
 * ============================================================================
 * Error Policy
 * ============================================================================
 */
#[allow(clippy::needless_pass_by_value, unused_variables)]
#[tracing::instrument(skip(object, ctx))]
fn error_policy(object: Arc<TorIngress>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
