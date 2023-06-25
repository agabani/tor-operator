use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, ExecAction, KeyToPath, PodSpec,
            PodTemplateSpec, Probe, SecretVolumeSource, Volume, VolumeMount,
        },
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::Patch,
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    kubernetes::{
        Annotations, KubeCrdResourceExt, KubeResourceExt, Labels, OBConfig, SelectorLabels, Torrc,
    },
    onion_key::OnionKey,
    Error, Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionService",
    namespaced,
    status = "OnionServiceStatus",
    version = "v1"
)]
/// # Onion Service
///
/// An Onion Service is an abstraction of a Tor Onion Service.
///
/// A Tor Onion Service is a service that can only be accessed over Tor.
/// Running a Tor Onion Service gives your users all teh security of HTTPS with
/// the added privacy benefits of Tor.
pub struct OnionServiceSpec {
    /// Config Map settings.
    pub config_map: Option<OnionServiceSpecConfigMap>,

    /// Deployment settings.
    pub deployment: Option<OnionServiceSpecDeployment>,

    /// Onion Balance part the Onion Service belongs to.
    ///
    /// Default: nil / none / null / undefined.
    pub onion_balance: Option<OnionServiceSpecOnionBalance>,

    /// Onion Key settings.
    pub onion_key: OnionServiceSpecOnionKey,

    /// Onion Service Hidden Service ports.
    pub ports: Vec<OnionServiceSpecHiddenServicePort>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecConfigMap {
    /// Name of the Config Map.
    ///
    /// Default: name of the Onion Service
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecDeployment {
    /// Name of the Deployment.
    ///
    /// Default: name of the Onion Service
    pub name: Option<String>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalance {
    /// Onion Key reference of the Onion Balance.
    pub onion_key: OnionServiceSpecOnionBalanceOnionKey,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionBalanceOnionKey {
    /// Hostname value of the Onion Key.
    ///
    /// Example: "abcdefg.onion"
    pub hostname: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecOnionKey {
    /// Name of the Onion Key.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionServiceSpecHiddenServicePort {
    /// The target any incoming traffic will be redirect to.
    ///
    /// Example: example.default.svc.cluster.local:80
    pub target: String,

    /// The virtual port that the Onion Service will be using.
    ///
    /// Example: 80
    pub virtport: i32,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionServiceStatus {
    /// Human readable description of state.
    ///
    /// Possible values:
    ///
    /// - onion key not found
    /// - onion key hostname not found
    /// - running
    pub state: String,
}

impl OnionService {
    #[must_use]
    fn default_name(&self) -> &str {
        self.try_name().unwrap().into()
    }

    #[must_use]
    pub fn config_map_name(&self) -> &str {
        self.spec
            .config_map
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn deployment_name(&self) -> &str {
        self.spec
            .deployment
            .as_ref()
            .and_then(|f| f.name.as_ref())
            .map_or_else(|| self.default_name(), String::as_str)
    }

    #[must_use]
    pub fn onion_balanced(&self) -> bool {
        self.spec.onion_balance.is_some()
    }

    #[must_use]
    pub fn onion_balance_onion_key_hostname(&self) -> Option<&str> {
        self.spec
            .onion_balance
            .as_ref()
            .map(|onion_balance| onion_balance.onion_key.hostname.as_str())
    }

    #[must_use]
    pub fn onion_key_name(&self) -> &str {
        &self.spec.onion_key.name
    }

    #[must_use]
    pub fn ports(&self) -> &[OnionServiceSpecHiddenServicePort] {
        &self.spec.ports
    }
}

impl KubeResourceExt for OnionService {}

impl KubeCrdResourceExt for OnionService {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-service";
}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    OnionService::crd()
}

/*
 * ============================================================================
 * Config
 * ============================================================================
 */
pub struct Config {
    pub tor_image: ImageConfig,
}

pub struct ImageConfig {
    pub pull_policy: String,
    pub uri: String,
}

/*
 * ============================================================================
 * Controller
 * ============================================================================
 */
pub async fn run_controller(client: Client, config: Config) {
    Controller::new(
        Api::<OnionService>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        Api::<ConfigMap>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        Api::<Deployment>::all(client.clone()),
        WatcherConfig::default(),
    )
    .shutdown_on_signal()
    .run(
        reconciler,
        error_policy,
        Arc::new(Context { client, config }),
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
    config: Config,
}

/*
 * ============================================================================
 * State
 * ============================================================================
 */
enum State {
    OnionKeyNotFound,
    OnionKeyHostnameNotFound,
    Running(OnionKey),
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::OnionKeyNotFound => write!(f, "onion key not found"),
            State::OnionKeyHostnameNotFound => write!(f, "onion key hostname not found"),
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
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let ob_config = generate_ob_config(&object);
    let torrc = generate_torrc(&object);

    let annotations = generate_annotations(&torrc);
    let labels = object.try_labels()?;
    let selector_labels = object.try_selector_labels()?;

    // onion key
    let state = reconcile_onion_key(
        //
        &Api::namespaced(ctx.client.clone(), &namespace),
        &object,
    )
    .await?;

    if let State::Running(onion_key) = &state {
        // config map
        reconcile_config_map(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &object,
            &annotations,
            &labels,
            &torrc,
            &ob_config,
        )
        .await?;

        // deployment
        reconcile_deployment(
            &Api::namespaced(ctx.client.clone(), &namespace),
            &ctx.config,
            &object,
            &annotations,
            &labels,
            &selector_labels,
            onion_key,
        )
        .await?;
    }

    // onion service
    reconcile_onion_service(
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

async fn reconcile_onion_key(api: &Api<OnionKey>, object: &OnionService) -> Result<State> {
    let Some(onion_key)  = api
        .get_opt(object.onion_key_name())
        .await
        .map_err(Error::Kube)? else {
            return Ok(State::OnionKeyNotFound)
        };

    if onion_key.hostname().is_none() {
        return Ok(State::OnionKeyHostnameNotFound);
    }

    Ok(State::Running(onion_key))
}

async fn reconcile_config_map(
    api: &Api<ConfigMap>,
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    torrc: &Torrc,
    ob_config: &Option<OBConfig>,
) -> Result<()> {
    // creation
    let config_map = generate_config_map(object, annotations, labels, ob_config, torrc);

    api.patch(
        &config_map.try_name()?,
        &object.patch_params(),
        &Patch::Apply(&config_map),
    )
    .await
    .map_err(Error::Kube)?;

    // deletion
    let config_maps = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    for config_map in &config_maps {
        let config_map_name = config_map.try_name()?;

        if config_map_name.as_str() != object.config_map_name() {
            api.delete(&config_map_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_deployment(
    api: &Api<Deployment>,
    config: &Config,
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Result<()> {
    let deployment = generate_deployment(
        object,
        config,
        annotations,
        labels,
        selector_labels,
        onion_key,
    );

    api.patch(
        &deployment.try_name()?,
        &object.patch_params(),
        &Patch::Apply(&deployment),
    )
    .await
    .map_err(Error::Kube)?;

    // deletion
    let deployments = api
        .list(&object.try_owned_list_params()?)
        .await
        .map_err(Error::Kube)?;

    for deployment in &deployments {
        let deployment_name = deployment.try_name()?;

        if deployment_name.as_str() != object.deployment_name() {
            api.delete(&deployment_name, &object.delete_params())
                .await
                .map_err(Error::Kube)?;
        }
    }

    Ok(())
}

async fn reconcile_onion_service(
    api: &Api<OnionService>,
    object: &OnionService,
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
            &object.patch_status(OnionServiceStatus { state }),
        )
        .await
        .map_err(Error::Kube)?;
    }

    Ok(())
}

fn generate_annotations(torrc: &Torrc) -> Annotations {
    Annotations::new(BTreeMap::from([(torrc.to_annotation_tuple())]))
}

fn generate_ob_config(object: &OnionService) -> Option<OBConfig> {
    object
        .spec
        .onion_balance
        .as_ref()
        .map(|f| OBConfig::new(format!("MasterOnionAddress {}", f.onion_key.hostname)))
}

fn generate_torrc(object: &OnionService) -> Torrc {
    let mut torrc = vec!["HiddenServiceDir /var/lib/tor/hidden_service".into()];
    if object.onion_balanced() {
        torrc.push("HiddenServiceOnionbalanceInstance 1".into());
    }
    for port in object.ports() {
        torrc.push(format!(
            "HiddenServicePort {} {}",
            port.virtport, port.target
        ));
    }
    Torrc::new(torrc.join("\n"))
}

fn generate_config_map(
    object: &OnionService,
    annotations: &Annotations,
    labels: &Labels,
    ob_config: &Option<OBConfig>,
    torrc: &Torrc,
) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(object.config_map_name().to_string()),
            annotations: Some(annotations.into()),
            labels: Some(labels.into()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some({
            let mut data = BTreeMap::from([("torrc".into(), torrc.into())]);
            if let Some(ob_config) = ob_config {
                data.insert("ob_config".into(), ob_config.into());
            }
            data
        }),
        ..Default::default()
    }
}

fn generate_deployment(
    object: &OnionService,
    config: &Config,
    annotations: &Annotations,
    labels: &Labels,
    selector_labels: &SelectorLabels,
    onion_key: &OnionKey,
) -> Deployment {
    Deployment {
        metadata: ObjectMeta {
            name: Some(object.deployment_name().to_string()),
            annotations: Some(annotations.into()),
            labels: Some(labels.into()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(selector_labels.into()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    annotations: Some(annotations.into()),
                    labels: Some(labels.into()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        args: Some(vec![
                            "-c".into(),
                            {
                                let mut commands = vec![
                                    "mkdir -p /var/lib/tor/hidden_service",
                                    "chmod 400 /var/lib/tor/hidden_service",
                                    "cp /etc/secrets/* /var/lib/tor/hidden_service",
                                ];

                                if object.onion_balanced() {
                                    commands.push("cp /etc/configs/ob_config /var/lib/tor/hidden_service/ob_config");
                                }

                                commands.push("mkdir -p /usr/local/etc/tor");
                                commands.push("cp /etc/configs/torrc /usr/local/etc/tor/torrc");
                                commands.push("tor -f /usr/local/etc/tor/torrc");
                                commands
                            }
                            .join(" && "),
                        ]),
                        command: Some(vec!["/bin/bash".into()]),
                        image: Some(config.tor_image.uri.clone()),
                        image_pull_policy: Some(config.tor_image.pull_policy.clone()),
                        liveness_probe: Some(Probe {
                            exec: Some(ExecAction {
                                command: Some(vec![
                                    "/bin/bash".to_string(),
                                    "-c".to_string(),
                                    "echo > /dev/tcp/127.0.0.1/9050".to_string(),
                                ]),
                            }),
                            failure_threshold: Some(3),
                            period_seconds: Some(10),
                            success_threshold: Some(1),
                            timeout_seconds: Some(1),
                            ..Default::default()
                        }),
                        name: "tor".into(),
                        readiness_probe: Some(Probe {
                            exec: Some(ExecAction {
                                command: Some(vec![
                                    "/bin/bash".to_string(),
                                    "-c".to_string(),
                                    "echo > /dev/tcp/127.0.0.1/9050".to_string(),
                                ]),
                            }),
                            failure_threshold: Some(3),
                            period_seconds: Some(10),
                            success_threshold: Some(1),
                            timeout_seconds: Some(1),
                            ..Default::default()
                        }),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                mount_path: "/etc/secrets".into(),
                                name: "etc-secrets".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                            VolumeMount {
                                mount_path: "/etc/configs".into(),
                                name: "etc-configs".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }],
                    volumes: Some(vec![
                        Volume {
                            name: "etc-secrets".into(),
                            secret: Some(SecretVolumeSource {
                                default_mode: Some(0o400),
                                items: Some(vec![
                                    KeyToPath {
                                        key: "hostname".into(),
                                        mode: Some(0o400),
                                        path: "hostname".into(),
                                    },
                                    KeyToPath {
                                        key: "hs_ed25519_public_key".into(),
                                        mode: Some(0o400),
                                        path: "hs_ed25519_public_key".into(),
                                    },
                                    KeyToPath {
                                        key: "hs_ed25519_secret_key".into(),
                                        mode: Some(0o400),
                                        path: "hs_ed25519_secret_key".into(),
                                    },
                                ]),
                                optional: Some(false),
                                secret_name: Some(onion_key.secret_name().to_string()),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "etc-configs".into(),
                            config_map: Some(ConfigMapVolumeSource {
                                default_mode: Some(0o400),
                                items: Some({
                                    let mut items = vec![
                                        KeyToPath {
                                            key: "torrc".into(),
                                            mode: Some(0o400),
                                            path: "torrc".into(),
                                        },
                                    ];
                                    if object.onion_balanced() {
                                        items.push( KeyToPath {
                                            key: "ob_config".into(),
                                            mode: Some(0o400),
                                            path: "ob_config".into(),
                                        });
                                    }
                                   items
                                }),
                                name: Some(object.config_map_name().to_string()),
                                optional: Some(false),
                            }),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

/*
 * ============================================================================
 * Error Policy
 * ============================================================================
 */
#[allow(clippy::needless_pass_by_value, unused_variables)]
#[tracing::instrument(skip(object, ctx))]
fn error_policy(object: Arc<OnionService>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
