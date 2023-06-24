use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::core::v1::Secret,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition, ByteString,
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource, ResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{self, Hostname},
    utils::{btree_maps_are_equal, KubeCrdResourceExt, KubeResourceExt},
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
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionKeyStatus {
    /// Onion key hostname.
    ///
    /// The hostname is only populated once `validation` is "valid".
    pub hostname: Option<String>,

    /// Human readable description of onion key validation.
    ///
    /// Possible Values:
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
    pub validation: String,
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
    pub fn secret_name(&self) -> &str {
        &self.spec.secret.name
    }
}

impl KubeResourceExt for OnionKey {}

impl KubeCrdResourceExt for OnionKey {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str = "onion-key";
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
pub async fn run_controller(client: Client, config: Config) {
    let onion_keys = Api::<OnionKey>::all(client.clone());
    let secrets = Api::<Secret>::all(client.clone());

    let context = Arc::new(Context {
        client,
        _config: config,
    });

    Controller::new(onion_keys, WatcherConfig::default())
        .owns(secrets, WatcherConfig::default())
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
const APP_KUBERNETES_IO_COMPONENT_VALUE: &str = "onion-key";

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
async fn reconciler(object: Arc<OnionKey>, ctx: Arc<Context>) -> Result<Action> {
    tracing::info!("reconciling");

    let object_name = get_object_name(&object)?;
    let object_namespace = get_object_namespace(&object)?;

    let annotations = generate_annotations();
    let labels = generate_labels(&object, &object_name);

    let secrets = Api::<Secret>::namespaced(ctx.client.clone(), object_namespace.0);

    let secret = secrets
        .get_opt(object.secret_name())
        .await
        .map_err(Error::Kube)?;

    let (result, secret) = generate_secret(&object, &secret, &annotations, &labels);

    if let Some(secret) = secret {
        secrets
            .patch(
                object.secret_name(),
                &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force(),
                &Patch::Apply(&secret),
            )
            .await
            .map_err(Error::Kube)?;
    }

    let hostname = match &result {
        GenerateSecretResult::Valid(hostname) => Some(hostname.to_string()),
        _ => None,
    };

    let validation = result.to_string();

    let changed = object.status.as_ref().map_or(true, |status| {
        status.validation != validation || status.hostname != hostname
    });

    if changed {
        Api::<OnionKey>::namespaced(ctx.client.clone(), object_namespace.0)
            .patch_status(
                object_name.0,
                &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE),
                &Patch::Merge(serde_json::json!({
                    "status": OnionKeyStatus {
                        hostname,
                        validation,
                    }
                })),
            )
            .await
            .map_err(Error::Kube)?;
    }

    let owned_secrets = secrets
        .list(&ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            object.metadata.uid.as_ref().unwrap()
        )))
        .await
        .map_err(Error::Kube)?;

    for owned_secret in owned_secrets {
        let name = owned_secret.metadata.name.unwrap();
        if name != object.secret_name() {
            secrets
                .delete(&name, &DeleteParams::default())
                .await
                .map_err(Error::Kube)?;
        }
    }

    tracing::info!("reconciled");

    match result {
        GenerateSecretResult::Valid(_) => Ok(Action::requeue(Duration::from_secs(3600))),
        _ => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

fn get_object_name(object: &OnionKey) -> Result<ObjectName> {
    Ok(ObjectName(
        object
            .metadata
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
            .as_str(),
    ))
}

fn get_object_namespace(object: &OnionKey) -> Result<ObjectNamespace> {
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

fn generate_labels(object: &OnionKey, object_name: &ObjectName) -> Labels {
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

enum GenerateSecretResult {
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

impl std::fmt::Display for GenerateSecretResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateSecretResult::SecretNotFound => write!(f, "secret not found"),
            GenerateSecretResult::SecretKeyNotFound => write!(f, "secret key not found"),
            GenerateSecretResult::SecretKeyMalformed(e) => write!(f, "secret key malformed: {e}"),
            GenerateSecretResult::PublicKeyNotFound => write!(f, "public key not found"),
            GenerateSecretResult::PublicKeyMalformed(e) => write!(f, "public key malformed: {e}"),
            GenerateSecretResult::PublicKeyMismatch => write!(f, "public key mismatch"),
            GenerateSecretResult::HostnameNotFound => write!(f, "hostname not found"),
            GenerateSecretResult::HostnameMalformed(e) => write!(f, "hostname malformed: {e}"),
            GenerateSecretResult::HostnameMismatch => write!(f, "hostname mismatch"),
            GenerateSecretResult::Valid(_) => write!(f, "valid"),
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
) -> (GenerateSecretResult, Option<Secret>) {
    let auto_generate = object.auto_generate();

    let Some(secret) = secret else {
        if !auto_generate {
            return (GenerateSecretResult::SecretNotFound, None);
        }

        tracing::info!("generating secret key");
        let secret_key = crypto::ExpandedSecretKey::generate();

        tracing::info!("generating public key");
        let public_key = secret_key.public_key();

        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        let secret = generate_owned_secret(
            object,
            annotations,
            labels,
            &public_key,
            &secret_key,
            &hostname
        );

        return (GenerateSecretResult::Valid(hostname), Some(secret));
    };

    let secret_key = secret
        .data
        .as_ref()
        .ok_or(GenerateSecretResult::SecretKeyNotFound)
        .and_then(|f| {
            f.get("hs_ed25519_secret_key")
                .ok_or(GenerateSecretResult::SecretKeyNotFound)
        })
        .and_then(|f| {
            crypto::HiddenServiceSecretKey::try_from_bytes(&f.0)
                .map_err(GenerateSecretResult::SecretKeyMalformed)
        })
        .and_then(|f| {
            crypto::ExpandedSecretKey::try_from_hidden_service_secret_key(&f)
                .map_err(GenerateSecretResult::SecretKeyMalformed)
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

            let secret = generate_owned_secret(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (GenerateSecretResult::Valid(hostname), Some(secret));
        }
    };

    let public_key = secret
        .data
        .as_ref()
        .ok_or(GenerateSecretResult::PublicKeyNotFound)
        .and_then(|f| {
            f.get("hs_ed25519_public_key")
                .ok_or(GenerateSecretResult::PublicKeyNotFound)
        })
        .and_then(|f| {
            crypto::HiddenServicePublicKey::try_from_bytes(&f.0)
                .map_err(GenerateSecretResult::PublicKeyMalformed)
        })
        .and_then(|f| {
            crypto::PublicKey::try_from_hidden_service_public_key(&f)
                .map_err(GenerateSecretResult::PublicKeyMalformed)
        })
        .and_then(|f| {
            if f == secret_key.public_key() {
                Ok(f)
            } else {
                Err(GenerateSecretResult::PublicKeyMismatch)
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

            let secret = generate_owned_secret(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (GenerateSecretResult::Valid(hostname), Some(secret));
        }
    };

    let hostname = secret
        .data
        .as_ref()
        .ok_or(GenerateSecretResult::HostnameNotFound)
        .and_then(|f| {
            f.get("hostname")
                .ok_or(GenerateSecretResult::HostnameNotFound)
        })
        .and_then(|f| {
            crypto::Hostname::try_from_bytes(&f.0).map_err(GenerateSecretResult::HostnameMalformed)
        })
        .and_then(|f| {
            if f == public_key.hostname() {
                Ok(f)
            } else {
                Err(GenerateSecretResult::HostnameMismatch)
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

            let secret = generate_owned_secret(
                object,
                annotations,
                labels,
                &public_key,
                &secret_key,
                &hostname,
            );

            return (GenerateSecretResult::Valid(hostname), Some(secret));
        }
    };

    if auto_generate
        && !(btree_maps_are_equal(object.annotations(), &annotations.0)
            && btree_maps_are_equal(object.labels(), &labels.0))
    {
        let secret = generate_owned_secret(
            object,
            annotations,
            labels,
            &public_key,
            &secret_key,
            &hostname,
        );

        return (GenerateSecretResult::Valid(hostname), Some(secret));
    }

    (GenerateSecretResult::Valid(hostname), None)
}

fn generate_owned_secret(
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
            annotations: Some(annotations.0.clone()),
            labels: Some(labels.0.clone()),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(BTreeMap::from([
            ("hostname".into(), ByteString(hostname.as_bytes().to_vec())),
            (
                "hs_ed25519_public_key".into(),
                ByteString(crypto::HiddenServicePublicKey::from_public_key(public_key).to_bytes()),
            ),
            (
                "hs_ed25519_secret_key".into(),
                ByteString(
                    crypto::HiddenServiceSecretKey::from_expanded_secret_key(secret_key).to_bytes(),
                ),
            ),
        ])),
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
fn error_policy(object: Arc<OnionKey>, error: &Error, ctx: Arc<Context>) -> Action {
    tracing::error!("failed to reconcile");
    Action::requeue(Duration::from_secs(5))
}
