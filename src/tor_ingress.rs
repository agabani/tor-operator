use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::Patch,
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    kubernetes::{self, error_policy, Annotations, KubeCrdResourceExt, KubeResourceExt, Labels},
    metrics::Metrics,
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
    Error, Result,
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
pub struct TorIngressStatus {
    /// Human readable description of state.
    ///
    /// Possible values:
    ///
    /// - onion balance onion key not found
    /// - onion balance onion key hostname not found
    /// - onion service onion key hostname not found
    /// - running
    pub state: String,
}

impl TorIngress {
    #[must_use]
    fn default_name(&self) -> &str {
        self.try_name().unwrap().into()
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

impl KubeResourceExt for TorIngress {}

impl KubeCrdResourceExt for TorIngress {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "tor-ingress";
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
pub async fn run_controller(client: Client, config: Config, metrics: Metrics) {
    Controller::new(
        Api::<TorIngress>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        Api::<OnionBalance>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        Api::<OnionKey>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        Api::<OnionService>::all(client.clone()),
        WatcherConfig::default(),
    )
    .shutdown_on_signal()
    .run(
        reconciler,
        error_policy,
        Arc::new(Context {
            client,
            _config: config,
            metrics,
        }),
    )
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
    _config: Config,
    metrics: Metrics,
}

impl kubernetes::Context for Context {
    fn metrics(&self) -> &Metrics {
        &self.metrics
    }
}

/*
 * ============================================================================
 * State
 * ============================================================================
 */
enum State {
    OnionBalanceOnionKeyNotFound,
    OnionBalanceOnionKeyHostnameNotFound,
    OnionServiceOnionKeyHostnameNotFound,
    Running((OnionKey, HashMap<i32, OnionKey>)),
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::OnionBalanceOnionKeyNotFound => write!(f, "onion balance onion key not found"),
            State::OnionBalanceOnionKeyHostnameNotFound => {
                write!(f, "onion balance onion key hostname not found")
            }
            State::OnionServiceOnionKeyHostnameNotFound => {
                write!(f, "onion service onion key hostname not found")
            }
            State::Running(_) => write!(f, "running"),
        }
    }
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<TorIngress>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(TorIngress::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let annotations = generate_annotations();
    let labels = object.try_labels()?;

    // onion key
    let state = reconcile_onion_key(
        &Api::namespaced(ctx.client.clone(), &namespace),
        &object,
        &annotations,
        &labels,
    )
    .await?;

    if let State::Running((onion_balance_onion_key, onion_service_onion_keys)) = &state {
        // onion services
        reconcile_onion_services(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &object,
            &annotations,
            &labels,
            onion_balance_onion_key,
        )
        .await?;

        // onion balance
        reconcile_onion_balance(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &object,
            &annotations,
            &labels,
            onion_service_onion_keys,
        )
        .await?;
    }

    // tor ingress
    reconcile_tor_ingress(
        &Api::namespaced(ctx.client.clone(), &namespace),
        &object,
        &state,
    )
    .await?;

    tracing::info!("reconciled");

    match state {
        State::Running(_) => Ok(Action::requeue(Duration::from_secs(3600))),
        _ => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

async fn reconcile_onion_key(
    api: &Api<OnionKey>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<State> {
    // onion balance
    let Some(onion_balance_onion_key) = api
        .get_opt(object.onion_balance_onion_key_name())
        .await
        .map_err(Error::Kube)? else {
            return Ok(State::OnionBalanceOnionKeyNotFound)
        };

    if onion_balance_onion_key.hostname().is_none() {
        return Ok(State::OnionBalanceOnionKeyHostnameNotFound);
    }

    // onion service
    for instance in 0..object.onion_service_replicas() {
        let onion_key = api
            .get_opt(&object.onion_service_onion_key_name(instance))
            .await
            .map_err(Error::Kube)?;

        let onion_key =
            generate_onion_service_onion_key(object, &onion_key, annotations, labels, instance);

        if let Some(onion_key) = onion_key {
            api.patch(
                &onion_key.try_name()?,
                &object.patch_params(),
                &Patch::Apply(&onion_key),
            )
            .await
            .map_err(Error::Kube)?;
        }
    }

    // onion service: ready and deletion
    let mut onion_service_onion_keys = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?
        .into_iter()
        .map(|onion_key| {
            let name = onion_key.try_name().unwrap().as_str().to_string();
            (name, onion_key)
        })
        .collect::<HashMap<_, _>>();

    let manifest = (0..object.onion_service_replicas())
        .map(|instance| (object.onion_service_onion_key_name(instance), instance))
        .collect::<HashMap<_, _>>();

    // onion service: ready
    let ready = manifest.iter().all(|(onion_key_name, _)| {
        let Some(onion_key) = onion_service_onion_keys.get(onion_key_name) else {
            return false;
        };
        onion_key.hostname().is_some()
    });

    if !ready {
        return Ok(State::OnionServiceOnionKeyHostnameNotFound);
    }

    // onion service: deletion
    for onion_service_onion_key_name in onion_service_onion_keys.keys() {
        let keep = manifest.get(onion_service_onion_key_name).is_some();

        if !keep {
            api.delete(onion_service_onion_key_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    let onion_service_onion_keys = manifest
        .iter()
        .map(|(name, instance)| {
            let onion_key = onion_service_onion_keys.remove(name).unwrap();
            (*instance, onion_key)
        })
        .collect();

    Ok(State::Running((
        onion_balance_onion_key,
        onion_service_onion_keys,
    )))
}

async fn reconcile_onion_services(
    api: &Api<OnionService>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_balance_onion_key: &OnionKey,
) -> Result<()> {
    // creation
    for instance in 0..object.onion_service_replicas() {
        let onion_service = api
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
            api.patch(
                &onion_service.try_name()?,
                &object.patch_params(),
                &Patch::Apply(&onion_service),
            )
            .await
            .map_err(Error::Kube)?;
        }
    }

    // deletion
    let owned_onion_services = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    let manifest: HashSet<String> = (0..object.onion_service_replicas())
        .map(|instance| object.onion_service_name(instance))
        .collect();

    for onion_service in &owned_onion_services {
        let onion_service_name = onion_service.try_name()?;

        if !manifest.contains(onion_service_name.as_str()) {
            api.delete(&onion_service_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_onion_balance(
    api: &Api<OnionBalance>,
    object: &TorIngress,
    annotations: &Annotations,
    labels: &Labels,
    onion_service_onion_keys: &HashMap<i32, OnionKey>,
) -> Result<()> {
    // creation
    let onion_balance = api
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
        api.patch(
            &onion_balance.try_name()?,
            &object.patch_params(),
            &Patch::Apply(&onion_balance),
        )
        .await
        .map_err(Error::Kube)?;
    }

    // deletion
    let onion_balances = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    for onion_balance in &onion_balances {
        let onion_balances_name = onion_balance.try_name()?;

        if onion_balances_name.as_str() != object.onion_balance_name() {
            api.delete(&onion_balances_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_tor_ingress(
    api: &Api<TorIngress>,
    object: &TorIngress,
    state: &State,
) -> Result<()> {
    let state = state.to_string();

    let changed = object
        .status
        .as_ref()
        .map_or(true, |status| status.state != state);

    if changed {
        api.patch_status(
            &object.try_name()?,
            &object.patch_status_params(),
            &object.patch_status(TorIngressStatus { state }),
        )
        .await
        .map_err(Error::Kube)?;
    }

    Ok(())
}

fn generate_annotations() -> Annotations {
    Annotations::new(BTreeMap::from([]))
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
                annotations: Some(annotations.into()),
                labels: Some(labels.into()),
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
                annotations: Some(annotations.into()),
                labels: Some(labels.into()),
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
                annotations: Some(annotations.into()),
                labels: Some(labels.into()),
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
