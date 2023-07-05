use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::core::v1::Secret,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition, ByteString,
};
use kube::{
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Client, CustomResource, CustomResourceExt, Resource, ResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{self, Hostname},
    kubernetes::{
        self, error_policy, Annotations, Api, Labels, Object, Resource as KubernetesResource,
        ResourceName, Subset,
    },
    metrics::Metrics,
    Result,
};

/*
 * ============================================================================
 * Custom Resource Definition
 * ============================================================================
 */
/// # Onion Key
///
/// An Onion Key is an abstraction of a Tor Onion Key.
///
/// A Tor Onion Key consists of the following files:
///
/// - `hostname`
/// - `hs_ed25519_public_key`
/// - `hs_ed25519_public_key`
///
/// A user can import their existing Tor Onion keys by creating a secret.
///
/// ```ignore
/// kubectl create secret generic tor-ingress-example \
///   --from-file=hostname=./hostname \
///   --from-file=hs_ed25519_public_key=./hs_ed25519_public_key \
///   --from-file=hs_ed25519_secret_key=./hs_ed25519_secret_key
/// ```
///
/// A user can have the Tor Operator create a new random Onion Key by using the
/// auto generate feature controlled by `.auto_generate`.
#[allow(clippy::module_name_repetitions)]
#[derive(CustomResource, JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[kube(
    group = "tor.agabani.co.uk",
    kind = "OnionKey",
    namespaced,
    printcolumn = r#"{"name":"Hostname", "type":"string", "description":"The hostname of the onion key", "jsonPath":".status.hostname"}"#,
    printcolumn = r#"{"name":"Auto Generated", "type":"boolean", "description":"Auto generated onion key", "jsonPath":".spec.auto_generate"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"Human readable description of state", "jsonPath":".status.state"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#,
    status = "OnionKeyStatus",
    version = "v1"
)]
pub struct OnionKeySpec {
    /// # Auto Generate
    ///
    /// Auto generate a random onion key. default: false.
    ///
    /// ## Auto Generate: False
    ///
    /// Tor Operator will use an existing Onion Key from the Secret specified
    /// in `.secret.name`.
    ///
    /// ## Auto Generate: True
    ///
    /// The Tor Operator will generate a random Onion Key and save it in the
    /// secret specified in `.secret.name`.
    ///
    /// If the Onion Key's secret key is missing or malformed, the Tor Operator
    /// will recreate the secret key.
    ///
    /// If the Onion Key's public key is missing, malformed, or does not match
    /// the secret key, the Tor Operator will deterministically recreate the
    /// public key from the secret key.
    ///
    /// If the Onion Key's hostname is missing, malformed, or does not match
    /// the public key, the Tor Operator will deterministically recreate the
    /// hostname from the public key.
    pub auto_generate: Option<bool>,

    /// Secret settings.
    pub secret: OnionKeySpecSecret,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionKeySpecSecret {
    /// Name of the secret.
    ///
    /// Secret data must have keys `hostname`, `hs_ed25519_public_key` and
    /// `hs_ed25519_secret_key`.
    pub name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct OnionKeyStatus {
    /// Onion key hostname.
    ///
    /// The hostname is only populated once `state` is "valid".
    pub hostname: Option<String>,

    /// Auto generated onion key.
    pub auto_generated: bool,

    /// Human readable description of state.
    ///
    /// Possible values:
    ///
    ///  - secret not found
    ///  - secret key not found
    ///  - secret key malformed: (reason)
    ///  - public key not found
    ///  - public key malformed: (reason)
    ///  - public key mismatch
    ///  - hostname not found
    ///  - hostname malformed: (reason)
    ///  - hostname mismatch
    ///  - valid
    pub state: String,
}

impl OnionKey {
    #[must_use]
    pub fn auto_generate(&self) -> bool {
        self.spec.auto_generate.unwrap_or(false)
    }

    #[must_use]
    pub fn hostname(&self) -> Option<&str> {
        self.status
            .as_ref()
            .and_then(|status| status.hostname.as_ref())
            .map(String::as_str)
    }

    #[must_use]
    pub fn secret_name(&self) -> ResourceName {
        (&self.spec.secret.name).into()
    }
}

impl KubernetesResource for OnionKey {
    type Spec = OnionKeySpec;

    fn spec(&self) -> &Self::Spec {
        &self.spec
    }
}

impl Object for OnionKey {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-key";

    type Status = OnionKeyStatus;

    fn status(&self) -> &Option<Self::Status> {
        &self.status
    }
}

impl Subset for OnionKeySpec {
    fn is_subset(&self, superset: &Self) -> bool {
        self == superset
    }
}

#[must_use]
pub fn generate_custom_resource_definition() -> CustomResourceDefinition {
    OnionKey::crd()
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
    Metrics::kubernetes_api_usage_count::<OnionKey>("watch");
    Metrics::kubernetes_api_usage_count::<Secret>("watch");
    Controller::new(
        kube::Api::<OnionKey>::all(client.clone()),
        WatcherConfig::default(),
    )
    .owns(
        kube::Api::<Secret>::all(client.clone()),
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
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<OnionKey>, ctx: Arc<Context>) -> Result<Action> {
    let _timer = ctx
        .metrics
        .count_and_measure(OnionKey::APP_KUBERNETES_IO_COMPONENT_VALUE);
    tracing::info!("reconciling");

    let namespace = object.try_namespace()?;

    let annotations = generate_annotations();
    let labels = object.try_labels()?;

    // secret
    let state = reconcile_secret(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
        &annotations,
        &labels,
    )
    .await?;

    // onion key
    reconcile_onion_key(
        &Api::new(kube::Api::namespaced(ctx.client.clone(), &namespace)),
        &object,
        &state,
    )
    .await?;

    tracing::info!("reconciled");

    match state {
        SecretState::Valid(_) => Ok(Action::requeue(Duration::from_secs(3600))),
        _ => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

async fn reconcile_secret(
    api: &Api<Secret>,
    object: &OnionKey,
    annotations: &Annotations,
    labels: &Labels,
) -> Result<SecretState> {
    let secret = api.get_opt(&object.secret_name()).await?;

    let (state, secret) = generate_secret(object, &secret, annotations, labels);

    if let Some(secret) = secret {
        if let SecretState::Valid(_) = state {
            api.sync(object, [((), secret)].into()).await?;
        }
    }

    Ok(state)
}

async fn reconcile_onion_key(
    api: &Api<OnionKey>,
    object: &OnionKey,
    state: &SecretState,
) -> Result<()> {
    api.update_status(
        object,
        OnionKeyStatus {
            hostname: match &state {
                SecretState::Valid(hostname) => Some(hostname.to_string()),
                _ => None,
            },
            auto_generated: object.auto_generate(),
            state: state.to_string(),
        },
    )
    .await
}

fn generate_annotations() -> Annotations {
    Annotations::new(BTreeMap::from([]))
}

enum SecretState {
    SecretNotFound,
    SecretKeyNotFound,
    SecretKeyMalformed(crypto::Error),
    PublicKeyNotFound,
    PublicKeyMalformed(crypto::Error),
    PublicKeyMismatch,
    HostnameNotFound,
    HostnameMalformed(crypto::Error),
    HostnameMismatch,
    Valid(Hostname),
}

impl std::fmt::Display for SecretState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretState::SecretNotFound => write!(f, "secret not found"),
            SecretState::SecretKeyNotFound => write!(f, "secret key not found"),
            SecretState::SecretKeyMalformed(e) => write!(f, "secret key malformed: {e}"),
            SecretState::PublicKeyNotFound => write!(f, "public key not found"),
            SecretState::PublicKeyMalformed(e) => write!(f, "public key malformed: {e}"),
            SecretState::PublicKeyMismatch => write!(f, "public key mismatch"),
            SecretState::HostnameNotFound => write!(f, "hostname not found"),
            SecretState::HostnameMalformed(e) => write!(f, "hostname malformed: {e}"),
            SecretState::HostnameMismatch => write!(f, "hostname mismatch"),
            SecretState::Valid(_) => write!(f, "valid"),
        }
    }
}

/// only returns a secret if a change needs to be made...
#[allow(clippy::too_many_lines)]
fn generate_secret(
    object: &OnionKey,
    secret: &Option<Secret>,
    annotations: &Annotations,
    labels: &Labels,
) -> (SecretState, Option<Secret>) {
    fn generate(
        object: &OnionKey,
        annotations: &Annotations,
        labels: &Labels,
        public_key: &crypto::PublicKey,
        secret_key: &crypto::ExpandedSecretKey,
        hostname: &crypto::Hostname,
    ) -> Secret {
        Secret {
            metadata: ObjectMeta {
                name: Some(object.secret_name().to_string()),
                annotations: Some(annotations.into()),
                labels: Some(labels.into()),
                owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
                ..Default::default()
            },
            data: Some(BTreeMap::from([
                ("hostname".into(), ByteString(hostname.as_bytes().to_vec())),
                (
                    "hs_ed25519_public_key".into(),
                    ByteString(
                        crypto::HiddenServicePublicKey::from_public_key(public_key).to_bytes(),
                    ),
                ),
                (
                    "hs_ed25519_secret_key".into(),
                    ByteString(
                        crypto::HiddenServiceSecretKey::from_expanded_secret_key(secret_key)
                            .to_bytes(),
                    ),
                ),
            ])),
            ..Default::default()
        }
    }

    let auto_generate = object.auto_generate();

    let Some(secret) = secret else {
        if !auto_generate {
            return (SecretState::SecretNotFound, None);
        }

        tracing::info!("generating secret key");
        let secret_key = crypto::ExpandedSecretKey::generate();

        tracing::info!("generating public key");
        let public_key = secret_key.public_key();

        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        let secret = generate(
            object,
            annotations,
            labels,
            &public_key,
            &secret_key,
            &hostname
        );

        return (SecretState::Valid(hostname), Some(secret));
    };

    let secret_key = secret
        .data
        .as_ref()
        .ok_or(SecretState::SecretKeyNotFound)
        .and_then(|f| {
            f.get("hs_ed25519_secret_key")
                .ok_or(SecretState::SecretKeyNotFound)
        })
        .and_then(|f| {
            crypto::HiddenServiceSecretKey::try_from_bytes(&f.0)
                .map_err(SecretState::SecretKeyMalformed)
        })
        .and_then(|f| {
            crypto::ExpandedSecretKey::try_from_hidden_service_secret_key(&f)
                .map_err(SecretState::SecretKeyMalformed)
        });

    let secret_key = match secret_key {
        Ok(secret_key) => secret_key,
        Err(validation) => {
            if !auto_generate {
                return (validation, None);
            }

            tracing::info!("generating secret key");
            let secret_key = crypto::ExpandedSecretKey::generate();

            tracing::info!("generating public key");
            let public_key = secret_key.public_key();

            tracing::info!("generating hostname");
            let hostname = public_key.hostname();

            let secret = generate(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (SecretState::Valid(hostname), Some(secret));
        }
    };

    let public_key = secret
        .data
        .as_ref()
        .ok_or(SecretState::PublicKeyNotFound)
        .and_then(|f| {
            f.get("hs_ed25519_public_key")
                .ok_or(SecretState::PublicKeyNotFound)
        })
        .and_then(|f| {
            crypto::HiddenServicePublicKey::try_from_bytes(&f.0)
                .map_err(SecretState::PublicKeyMalformed)
        })
        .and_then(|f| {
            crypto::PublicKey::try_from_hidden_service_public_key(&f)
                .map_err(SecretState::PublicKeyMalformed)
        })
        .and_then(|f| {
            if f == secret_key.public_key() {
                Ok(f)
            } else {
                Err(SecretState::PublicKeyMismatch)
            }
        });

    let public_key = match public_key {
        Ok(public_key) => public_key,
        Err(validation) => {
            if !auto_generate {
                return (validation, None);
            }

            tracing::info!("generating public key");
            let public_key = secret_key.public_key();

            tracing::info!("generating hostname");
            let hostname = public_key.hostname();

            let secret = generate(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (SecretState::Valid(hostname), Some(secret));
        }
    };

    let hostname = secret
        .data
        .as_ref()
        .ok_or(SecretState::HostnameNotFound)
        .and_then(|f| f.get("hostname").ok_or(SecretState::HostnameNotFound))
        .and_then(|f| {
            crypto::Hostname::try_from_bytes(&f.0).map_err(SecretState::HostnameMalformed)
        })
        .and_then(|f| {
            if f == public_key.hostname() {
                Ok(f)
            } else {
                Err(SecretState::HostnameMismatch)
            }
        });

    let hostname = match hostname {
        Ok(hostname) => hostname,
        Err(validation) => {
            if !auto_generate {
                return (validation, None);
            }

            tracing::info!("generating hostname");
            let hostname = public_key.hostname();

            let secret = generate(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (SecretState::Valid(hostname), Some(secret));
        }
    };

    if auto_generate
        && (!annotations.is_subset(object.annotations()) || !labels.is_subset(object.labels()))
    {
        let secret = generate(
            object,
            annotations,
            labels,
            &public_key,
            &secret_key,
            &hostname,
        );

        return (SecretState::Valid(hostname), Some(secret));
    }

    (SecretState::Valid(hostname), None)
}
