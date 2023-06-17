use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::StreamExt;
use k8s_openapi::{
    api::core::v1::Secret,
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition, ByteString,
};
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config as WatcherConfig, Controller},
    Api, Client, CustomResource, CustomResourceExt, Resource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{self, Hostname},
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
    kind = "OnionKey",
    namespaced,
    status = "OnionKeyStatus",
    version = "v1"
)]
pub struct OnionKeySpec {
    secret_name: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Clone)]
pub struct OnionKeyStatus {}

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
#[allow(clippy::missing_panics_doc)]
pub async fn run_controller(config: Config) {
    let client = Client::try_default().await.unwrap();

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
const APP_KUBERNETES_IO_COMPONENT: &str = "onion-key";
const APP_KUBERNETES_IO_NAME: &str = "tor";
const APP_KUBERNETES_IO_MANAGED_BY: &str = "tor-operator";

/*
 * ============================================================================
 * Types
 * ============================================================================
 */
struct Annotations(BTreeMap<String, String>);
struct Labels(BTreeMap<String, String>);
struct ObjectName<'a>(&'a str);
struct ObjectNamespace<'a>(&'a str);

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

    let secrets = Api::<Secret>::namespaced(ctx.client.clone(), object_namespace.0);

    let secret = secrets
        .get_opt(&object.spec.secret_name)
        .await
        .map_err(Error::Kube)?;

    let (secret_key, public_key, hostname) = generate_keys(&secret);

    let annotations = generate_annotations(&hostname);
    let labels = generate_labels(&object_name);

    secrets
        .patch(
            &object.spec.secret_name,
            &PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY).force(),
            &Patch::Apply(&generate_owned_secret(
                &object,
                &annotations,
                &labels,
                &public_key,
                &secret_key,
                &hostname,
            )),
        )
        .await
        .map_err(Error::Kube)?;

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
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

fn generate_annotations(hostname: &Hostname) -> Annotations {
    Annotations(BTreeMap::from([(
        "tor.agabani.co.uk/hostname".into(),
        hostname.to_string(),
    )]))
}

fn generate_labels(object_name: &ObjectName) -> Labels {
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
    ]))
}

#[allow(clippy::too_many_lines)]
fn generate_keys(
    secret: &Option<Secret>,
) -> (
    crypto::ExpandedSecretKey,
    crypto::PublicKey,
    crypto::Hostname,
) {
    let Some(secret) = secret else {
        tracing::info!("secret not found");

        tracing::info!("generating secret key");
        let secret_key = crypto::ExpandedSecretKey::generate();

        tracing::info!("generating public key");
        let public_key = secret_key.public_key();

        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        return (secret_key, public_key, hostname);
    };

    let secret_key = secret
        .data
        .as_ref()
        .and_then(|f| {
            let value = f.get("hs_ed25519_secret_key");
            if value.is_none() {
                tracing::warn!("secret key not found");
            }
            value
        })
        .and_then(|f| {
            let value = crypto::HiddenServiceSecretKey::try_from_bytes(&f.0);
            if let Err(error) = &value {
                tracing::warn!(error =? error, "secret key malformed");
            }
            value.ok()
        })
        .and_then(|f| {
            let value = crypto::ExpandedSecretKey::try_from_hidden_service_secret_key(&f);
            if let Err(error) = &value {
                tracing::warn!(error =? error, "secret key malformed");
            }
            value.ok()
        });

    let Some(secret_key) =  secret_key else {
        tracing::info!("generating secret key");
        let secret_key = crypto::ExpandedSecretKey::generate();

        tracing::info!("generating public key");
        let public_key = secret_key.public_key();

        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        return (secret_key, public_key, hostname);
    };

    let public_key = secret
        .data
        .as_ref()
        .and_then(|f| {
            let value = f.get("hs_ed25519_public_key");
            if value.is_none() {
                tracing::warn!("public key not found");
            }
            value
        })
        .and_then(|f| {
            let value = crypto::HiddenServicePublicKey::try_from_bytes(&f.0);
            if let Err(error) = &value {
                tracing::warn!(error =? error, "public key malformed");
            }
            value.ok()
        })
        .and_then(|f| {
            let value = crypto::PublicKey::try_from_hidden_service_public_key(&f);
            if let Err(error) = &value {
                tracing::warn!(error =? error, "public key malformed");
            }
            value.ok()
        })
        .and_then(|f| {
            if f == secret_key.public_key() {
                Some(f)
            } else {
                tracing::warn!("public key mismatch");
                None
            }
        });

    let Some(public_key) =  public_key else {
        tracing::info!("generating public key");
        let public_key = secret_key.public_key();

        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        return (secret_key, public_key, hostname);
    };

    let hostname = secret
        .data
        .as_ref()
        .and_then(|f| {
            let value = f.get("hostname");
            if value.is_none() {
                tracing::warn!("hostname not found");
            }
            value
        })
        .and_then(|f| {
            let value = crypto::Hostname::try_from_bytes(&f.0);
            if let Err(error) = &value {
                tracing::warn!(error =? error, "hostname malformed");
            }
            value.ok()
        })
        .and_then(|f| {
            if f == public_key.hostname() {
                Some(f)
            } else {
                tracing::warn!("hostname mismatch");
                None
            }
        });

    let Some(hostname) =  hostname else {
        tracing::info!("generating hostname");
        let hostname = public_key.hostname();

        return (secret_key, public_key, hostname);
    };

    (secret_key, public_key, hostname)
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
            name: Some(object.spec.secret_name.clone()),
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
