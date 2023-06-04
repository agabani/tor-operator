use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::{future, StreamExt};
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{Container, ExecAction, PodSpec, PodTemplateSpec, Probe},
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Client, Resource,
};

use crate::crd::OnionService;

#[allow(clippy::missing_panics_doc)]
pub async fn run(tor_image_pull_policy: &str, tor_image_uri: &str) {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<OnionService>::all(client.clone());
    let deployments = Api::<Deployment>::all(client.clone());

    let context = Arc::new(Context {
        client,
        tor_image_pull_policy: tor_image_pull_policy.into(),
        tor_image_uri: tor_image_uri.into(),
    });

    Controller::new(onion_services, Config::default())
        .owns(deployments, Config::default())
        .shutdown_on_signal()
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
enum Error {
    Kube(kube::Error),
    MissingObjectKey(&'static str),
}

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
struct Context {
    client: Client,
    tor_image_pull_policy: String,
    tor_image_uri: String,
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action, Error> {
    const APP_KUBERNETES_IO_COMPONENT: &str = "onion-service";
    const APP_KUBERNETES_IO_NAME: &str = "tor";
    const APP_KUBERNETES_IO_MANAGED_BY: &str = "tor-operator";

    tracing::info!("reconciling");

    let object_name = object
        .metadata
        .name
        .as_ref()
        .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?;

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(format!(
                "{APP_KUBERNETES_IO_NAME}-{APP_KUBERNETES_IO_COMPONENT}-{object_name}"
            )),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/component".into(),
                    APP_KUBERNETES_IO_COMPONENT.into(),
                ),
                ("app.kubernetes.io/instance".into(), object_name.into()),
                (
                    "app.kubernetes.io/managed-by".into(),
                    APP_KUBERNETES_IO_MANAGED_BY.into(),
                ),
                (
                    "app.kubernetes.io/name".into(),
                    APP_KUBERNETES_IO_NAME.into(),
                ),
            ])),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([
                    (
                        "app.kubernetes.io/component".into(),
                        APP_KUBERNETES_IO_COMPONENT.into(),
                    ),
                    ("app.kubernetes.io/instance".into(), object_name.into()),
                    (
                        "app.kubernetes.io/name".into(),
                        APP_KUBERNETES_IO_NAME.into(),
                    ),
                ])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([
                        (
                            "app.kubernetes.io/component".into(),
                            APP_KUBERNETES_IO_COMPONENT.into(),
                        ),
                        ("app.kubernetes.io/instance".into(), object_name.into()),
                        (
                            "app.kubernetes.io/managed-by".into(),
                            APP_KUBERNETES_IO_MANAGED_BY.into(),
                        ),
                        (
                            "app.kubernetes.io/name".into(),
                            APP_KUBERNETES_IO_NAME.into(),
                        ),
                    ])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        image: Some(ctx.tor_image_uri.clone()),
                        image_pull_policy: Some(ctx.tor_image_pull_policy.clone()),
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
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let deployments: Api<Deployment> = Api::namespaced(
        ctx.client.clone(),
        object.metadata.namespace.as_ref().unwrap(),
    );

    deployments
        .patch(
            deployment
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
            &PatchParams::apply("onionservices.tor.agabani.co.uk").force(),
            &Patch::Apply(&deployment),
        )
        .await
        .map_err(Error::Kube)?;

    tracing::info!("reconciled");

    Ok(Action::requeue(Duration::from_secs(3600)))
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
