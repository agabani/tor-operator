use std::{
    collections::{BTreeMap, HashMap, HashSet},
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
    onion_balance::{
        OnionBalance, OnionBalanceSpec, OnionBalanceSpecConfigMap, OnionBalanceSpecDeployment,
        OnionBalanceSpecOnionKey, OnionBalanceSpecOnionService,
        OnionBalanceSpecOnionServiceOnionKey,
    },
    onion_key::{OnionKey, OnionKeySpec, OnionKeySpecSecret},
    onion_service::{
        OnionService, OnionServiceSpec, OnionServiceSpecConfigMap, OnionServiceSpecDeployment,
        OnionServiceSpecHiddenServicePort, OnionServiceSpecOnionBalance,
        OnionServiceSpecOnionBalanceOnionKey, OnionServiceSpecOnionKey,
    },
    Annotations, Error, Labels, ObjectName, ObjectNamespace, Result,
    APP_KUBERNETES_IO_COMPONENT_KEY, APP_KUBERNETES_IO_INSTANCE_KEY,
    APP_KUBERNETES_IO_MANAGED_BY_KEY, APP_KUBERNETES_IO_MANAGED_BY_VALUE,
    APP_KUBERNETES_IO_NAME_KEY, APP_KUBERNETES_IO_NAME_VALUE, TOR_AGABANI_CO_UK_OWNED_BY_KEY,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # Tor Ingress
///
/// A Tor Ingress is collection of Onion Services load balanced by a Onion Balance.
///
/// The user must provide the Onion Key for the Onion Balance.
///
/// The Tor Operator will auto generate random Onion Keys for the Onion Services.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "TorIngress",
    namespaced,
    status = "TorIngressStatus",
    version = "v1"
)]
pub struct TorIngressSpec {
    /// Onion Balance settings.
    pub onion_balance: TorIngressSpecOnionBalance,

    /// Onion Service settings.
    pub onion_service: TorIngressSpecOnionService,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionBalance {
    /// Config Map settings.
    pub config_map: Option<TorIngressSpecOnionBalanceConfigMap>,

    /// Deployment settings.
    pub deployment: Option<TorIngressSpecOnionBalanceDeployment>,

    /// Name of the Onion Balance.
    ///
    /// Default: name of the Tor Ingress
    pub name: Option<String>,

    /// Onion Key settings.
    pub onion_key: TorIngressSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionBalanceConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the Tor Ingress
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionBalanceDeployment {
    /// Name of the Deployment.
    ///
    /// Default: name of the Tor Ingress
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionBalanceOnionKey {
    /// Name of the Onion Key.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionService {
    /// Config Map settings.
    pub config_map: Option<TorIngressSpecOnionServiceConfigMap>,

    /// Deployment settings.
    pub deployment: Option<TorIngressSpecOnionServiceDeployment>,

    /// Name prefix of the Onion Service.
    ///
    /// Default: name of the Tor Ingress
    pub name_prefix: Option<String>,

    /// Onion Key settings.
    pub onion_key: Option<TorIngressSpecOnionServiceOnionKey>,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<TorIngressSpecOnionServicePort>,

    /// Number of replicas.
    ///
    /// Default: 3
    pub replicas: Option<i32>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionServiceConfigMap {
    /// Name prefix of the Config Map.
    ///
    /// Default: name of the Tor Ingress
    pub name_prefix: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionServiceDeployment {
    /// Name prefix of the Deployment.
    ///
    /// Default: name of the Tor Ingress
    pub name_prefix: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionServiceOnionKey {
    /// Name prefix of the Onion Key.
    ///
    /// Default: name of the Tor Ingress
    pub name_prefix: Option<String>,

    /// Secret settings.
    pub secret: Option<TorIngressSpecOnionServiceOnionKeySecret>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionServiceOnionKeySecret {
    /// Name prefix of the Secret.
    ///
    /// Default: name of the Tor Ingress
    pub name_prefix: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorIngressSpecOnionServicePort {
    /// The target any incoming traffic will be redirect to.
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct TorIngressStatus {}

impl TorIngress {
    #[must_use]
    fn default_name(&self) -> &str {
        self.metadata.name.as_ref().unwrap()
    }

    #[must_use]
    pub fn onion_balance_config_map_name(&self) -> &str {
        self.spec
            .onion_balance
            .config_map
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_balance_deployment_name(&self) -> &str {
        self.spec
            .onion_balance
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_balance_name(&self) -> &str {
        self.spec
            .onion_balance
            .name
            .as_ref()
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_balance_onion_key_name(&self) -> &str {
        &self.spec.onion_balance.onion_key.name
    }

    #[must_use]
    pub fn onion_service_config_map_name_prefix(&self) -> &str {
        self.spec
            .onion_service
            .config_map
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_service_config_map_name(&self, instance: i32) -> String {
        format!("{}-{instance}", self.onion_service_config_map_name_prefix())
    }

    #[must_use]
    pub fn onion_service_deployment_name_prefix(&self) -> &str {
        self.spec
            .onion_service
            .deployment
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_service_deployment_name(&self, instance: i32) -> String {
        format!("{}-{instance}", self.onion_service_deployment_name_prefix())
    }

    #[must_use]
    pub fn onion_service_name_prefix(&self) -> &str {
        self.spec
            .onion_service
            .name_prefix
            .as_ref()
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_service_name(&self, instance: i32) -> String {
        format!("{}-{instance}", self.onion_service_name_prefix())
    }

    #[must_use]
    pub fn onion_service_onion_key_name_prefix(&self) -> &str {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_service_onion_key_name(&self, instance: i32) -> String {
        format!("{}-{instance}", self.onion_service_onion_key_name_prefix())
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_name_prefix(&self) -> &str {
        self.spec
            .onion_service
            .onion_key
            .as_ref()
            .and_then(|f| f.secret.as_ref())
            .and_then(|f| f.name_prefix.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_service_onion_key_secret_name(&self, instance: i32) -> String {
        format!(
            "{}-{instance}",
            self.onion_service_onion_key_secret_name_prefix()
        )
    }

    #[must_use]
    pub fn onion_service_replicas(&self) -> i32 {
        self.spec.onion_service.replicas.unwrap_or(3)
    }
}

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
pub async fn run_controller(client: Client, config: Config) {
    let onion_balances = Api::<OnionBalance>::all(client.clone());
    let onion_keys = Api::<OnionKey>::all(client.clone());
    let onion_services = Api::<OnionService>::all(client.clone());
    let tor_ingresses = Api::<TorIngress>::all(client.clone());

    let context = Arc::new(Context {
        client,
        _config: config,
    });

    Controller::new(tor_ingresses, WatcherConfig::default())
        .owns(onion_balances, WatcherConfig::default())
        .owns(onion_keys, WatcherConfig::default())
        .owns(onion_services, WatcherConfig::default())
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
const APP_KUBERNETES_IO_COMPONENT_VALUE: &str = "tor-ingress";

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

    // onion balance onion key
    let onion_balance_onion_key =
        reconcile_onion_balance_onion_key(&object, &ctx, &object_namespace).await?;

    let Some(onion_balance_onion_key) = onion_balance_onion_key else {
        // TODO: status: waiting for onion balance onion key hostname.
        tracing::info!("status: waiting for onion balance onion key hostname.");
        return  Ok(Action::requeue(Duration::from_secs(5)));
    };

    // onion service onion keys
    let onion_service_onion_keys =
        reconcile_onion_service_onion_keys(&object, &ctx, &object_namespace, &annotations, &labels)
            .await?;

    let Some(onion_service_onion_keys) = onion_service_onion_keys else {
        // TODO: status: waiting for onion service onion key hostnames.
        tracing::info!("status: waiting for onion service onion key hostnames.");
        return  Ok(Action::requeue(Duration::from_secs(5)));
    };

    // onion services
    reconcile_onion_services(
        &object,
        &ctx,
        &object_namespace,
        &annotations,
        &labels,
        &onion_balance_onion_key,
    )
    .await?;

    // onion balance
    reconcile_onion_balance(
        &object,
        &ctx,
        &object_namespace,
        &annotations,
        &labels,
        &onion_service_onion_keys,
    )
    .await?;

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
}

async fn reconcile_onion_balance_onion_key(
    object: &TorIngress,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
) -> Result<Option<OnionKey>> {
    let onion_keys = Api::<OnionKey>::namespaced(ctx.client.clone(), object_namespace.0);

    let onion_key = onion_keys
        .get(object.onion_balance_onion_key_name())
        .await
        .map_err(Error::Kube)?;

    if onion_key.hostname().is_some() {
        Ok(Some(onion_key))
    } else {
        Ok(None)
    }
}

async fn reconcile_onion_service_onion_keys(
    object: &TorIngress,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<Option<HashMap<i32, OnionKey>>> {
    let onion_keys = Api::<OnionKey>::namespaced(ctx.client.clone(), object_namespace.0);

    // creation
    for instance in 0..object.onion_service_replicas() {
        let onion_key = onion_keys
            .get_opt(&object.onion_service_onion_key_name(instance))
            .await
            .map_err(Error::Kube)?;

        let onion_key =
            generate_onion_service_onion_key(object, &onion_key, annotations, labels, instance);

        if let Some(onion_key) = onion_key {
            onion_keys
                .patch(
                    onion_key
                        .metadata
                        .name
                        .as_ref()
                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
                    &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE),
                    &Patch::Apply(&onion_key),
                )
                .await
                .map_err(Error::Kube)?;
        }
    }

    // ready and deletion
    let mut owned_onion_keys = onion_keys
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?
        .into_iter()
        .map(|onion_key| {
            let name = onion_key.metadata.name.clone().unwrap();
            (name, onion_key)
        })
        .collect::<HashMap<_, _>>();

    let manifest = (0..object.onion_service_replicas())
        .map(|instance| (object.onion_service_onion_key_name(instance), instance))
        .collect::<HashMap<_, _>>();

    // ready
    let ready = manifest.iter().all(|(onion_key_name, _)| {
        let Some(onion_key) = owned_onion_keys.get(onion_key_name) else {
            return false;
        };
        onion_key.hostname().is_some()
    });

    if !ready {
        return Ok(None);
    }

    // deletion
    for onion_key_name in owned_onion_keys.keys() {
        let keep = manifest.get(onion_key_name).is_some();

        if !keep {
            onion_keys
                .delete(onion_key_name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(Some(
        manifest
            .iter()
            .map(|(name, instance)| {
                let onion_key = owned_onion_keys.remove(name).unwrap();
                (*instance, onion_key)
            })
            .collect(),
    ))
}

async fn reconcile_onion_services(
    object: &TorIngress,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
    onion_balance_onion_key: &OnionKey,
) -> Result<()> {
    let onion_services = Api::<OnionService>::namespaced(ctx.client.clone(), object_namespace.0);

    // creation
    for instance in 0..object.onion_service_replicas() {
        let onion_service = onion_services
            .get_opt(&object.onion_service_name(instance))
            .await
            .map_err(Error::Kube)?;

        let onion_service = generate_onion_service(
            object,
            &onion_service,
            annotations,
            labels,
            onion_balance_onion_key,
            instance,
        );

        if let Some(onion_service) = onion_service {
            onion_services
                .patch(
                    onion_service
                        .metadata
                        .name
                        .as_ref()
                        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
                    &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE),
                    &Patch::Apply(&onion_service),
                )
                .await
                .map_err(Error::Kube)?;
        }
    }

    // deletion
    let owned_onion_services = onion_services
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    let manifest: HashSet<String> = (0..object.onion_service_replicas())
        .map(|instance| object.onion_service_name(instance))
        .collect();

    for onion_service in &owned_onion_services {
        let onion_service_name = onion_service
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?;

        if !manifest.contains(onion_service_name) {
            onion_services
                .delete(onion_service_name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_onion_balance(
    object: &TorIngress,
    ctx: &Context,
    object_namespace: &ObjectNamespace<'_>,
    annotations: &Annotations,
    labels: &Labels,
    onion_service_onion_keys: &HashMap<i32, OnionKey>,
) -> Result<()> {
    let onion_balances = Api::<OnionBalance>::namespaced(ctx.client.clone(), object_namespace.0);

    // creation
    let onion_balance = onion_balances
        .get_opt(object.onion_balance_name())
        .await
        .map_err(Error::Kube)?;

    let onion_balance = generate_onion_balance(
        object,
        &onion_balance,
        annotations,
        labels,
        onion_service_onion_keys,
    );

    if let Some(onion_balance) = onion_balance {
        onion_balances
            .patch(
                onion_balance.metadata.name.as_ref().unwrap(),
                &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE),
                &Patch::Apply(&onion_balance),
            )
            .await
            .map_err(Error::Kube)?;
    }

    // deletion
    let owned_onion_balances = onion_balances
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    for onion_balance in &owned_onion_balances {
        let onion_balances_name = onion_balance
            .meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?;

        if onion_balances_name != object.onion_balance_name() {
            onion_balances
                .delete(onion_balances_name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
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
            APP_KUBERNETES_IO_COMPONENT_KEY.into(),
            APP_KUBERNETES_IO_COMPONENT_VALUE.into(),
        ),
        (APP_KUBERNETES_IO_INSTANCE_KEY.into(), object_name.0.into()),
        (
            APP_KUBERNETES_IO_MANAGED_BY_KEY.into(),
            APP_KUBERNETES_IO_MANAGED_BY_VALUE.into(),
        ),
        (
            APP_KUBERNETES_IO_NAME_KEY.into(),
            APP_KUBERNETES_IO_NAME_VALUE.into(),
        ),
        (
            TOR_AGABANI_CO_UK_OWNED_BY_KEY.into(),
            object.metadata.uid.clone().unwrap(),
        ),
    ]))
}

/// only returns an onion key if a change needs to be made...
fn generate_onion_balance(
    object: &TorIngress,
    onion_balance: &Option<OnionBalance>,
    annotations: &Annotations,
    labels: &Labels,
    onion_service_onion_keys: &HashMap<i32, OnionKey>,
) -> Option<OnionBalance> {
    fn generate(
        object: &TorIngress,
        spec: OnionBalanceSpec,
        annotations: &Annotations,
        labels: &Labels,
    ) -> OnionBalance {
        OnionBalance {
            metadata: ObjectMeta {
                name: Some(object.onion_balance_name().to_string()),
                annotations: Some(annotations.0.clone()),
                labels: Some(labels.0.clone()),
                owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
                ..Default::default()
            },
            spec,
            status: None,
        }
    }

    let spec = OnionBalanceSpec {
        config_map: Some(OnionBalanceSpecConfigMap {
            name: Some(object.onion_balance_config_map_name().to_string()),
        }),
        deployment: Some(OnionBalanceSpecDeployment {
            name: Some(object.onion_balance_deployment_name().to_string()),
        }),
        onion_key: OnionBalanceSpecOnionKey {
            name: object.onion_balance_onion_key_name().to_string(),
        },
        onion_services: (0..onion_service_onion_keys.len())
            .map(|instance| OnionBalanceSpecOnionService {
                onion_key: OnionBalanceSpecOnionServiceOnionKey {
                    hostname: onion_service_onion_keys
                        .get(&i32::try_from(instance).unwrap())
                        .and_then(OnionKey::hostname)
                        .unwrap()
                        .to_string(),
                },
            })
            .collect(),
    };

    let Some(onion_balance) = onion_balance else {
        return Some(generate(object, spec, annotations, labels));
    };

    if onion_balance.spec != spec {
        return Some(generate(object, spec, annotations, labels));
    }

    None
}

/// only returns an onion key if a change needs to be made...
fn generate_onion_service_onion_key(
    object: &TorIngress,
    onion_key: &Option<OnionKey>,
    annotations: &Annotations,
    labels: &Labels,
    instance: i32,
) -> Option<OnionKey> {
    fn generate(
        object: &TorIngress,
        spec: OnionKeySpec,
        annotations: &Annotations,
        labels: &Labels,
        instance: i32,
    ) -> OnionKey {
        OnionKey {
            metadata: ObjectMeta {
                name: Some(object.onion_service_onion_key_name(instance)),
                annotations: Some(annotations.0.clone()),
                labels: Some(labels.0.clone()),
                owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
                ..Default::default()
            },
            spec,
            status: None,
        }
    }

    let spec = OnionKeySpec {
        auto_generate: Some(true),
        secret: OnionKeySpecSecret {
            name: object.onion_service_onion_key_secret_name(instance),
        },
    };

    let Some(onion_key) = onion_key else {
        return Some(generate(object, spec, annotations, labels, instance));
    };

    if onion_key.spec != spec {
        return Some(generate(object, spec, annotations, labels, instance));
    }

    None
}

/// only returns an onion service if a change needs to be made...
fn generate_onion_service(
    object: &TorIngress,
    onion_service: &Option<OnionService>,
    annotations: &Annotations,
    labels: &Labels,
    onion_balance_onion_key: &OnionKey,
    instance: i32,
) -> Option<OnionService> {
    fn generate(
        object: &TorIngress,
        spec: OnionServiceSpec,
        annotations: &Annotations,
        labels: &Labels,
        instance: i32,
    ) -> OnionService {
        OnionService {
            metadata: ObjectMeta {
                name: Some(object.onion_service_name(instance)),
                annotations: Some(annotations.0.clone()),
                labels: Some(labels.0.clone()),
                owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
                ..Default::default()
            },
            spec,
            status: None,
        }
    }

    let spec = OnionServiceSpec {
        config_map: Some(OnionServiceSpecConfigMap {
            name: Some(object.onion_service_config_map_name(instance)),
        }),
        deployment: Some(OnionServiceSpecDeployment {
            name: Some(object.onion_service_deployment_name(instance)),
        }),
        onion_balance: Some(OnionServiceSpecOnionBalance {
            onion_key: OnionServiceSpecOnionBalanceOnionKey {
                hostname: onion_balance_onion_key.hostname().unwrap().to_string(),
            },
        }),
        onion_key: OnionServiceSpecOnionKey {
            name: object.onion_service_onion_key_name(instance),
        },
        ports: object
            .spec
            .onion_service
            .ports
            .iter()
            .map(|f| OnionServiceSpecHiddenServicePort {
                target: f.target.clone(),
                virtport: f.virtport,
            })
            .collect(),
    };

    let Some(onion_service) = onion_service else {
        return Some(generate(object, spec, annotations, labels, instance));
    };

    if onion_service.spec != spec {
        return Some(generate(object, spec, annotations, labels, instance));
    }

    None
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
