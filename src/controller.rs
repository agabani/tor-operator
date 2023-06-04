use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::{future, StreamExt};
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{Container, ContainerPort, HTTPGetAction, PodSpec, PodTemplateSpec, Probe},
    },
    apimachinery::pkg::{apis::meta::v1::LabelSelector, util::intstr::IntOrString},
};
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Client, Resource,
};

use crate::crd::OnionService;

#[allow(clippy::missing_panics_doc)]
pub async fn run() {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<OnionService>::all(client.clone());
    let deployments = Api::<Deployment>::all(client.clone());

    let context = Arc::new(Context { client });

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
}

/*
 * ============================================================================
 * Reconciler
 * ============================================================================
 */
#[tracing::instrument(skip(object, ctx))]
async fn reconciler(object: Arc<OnionService>, ctx: Arc<Context>) -> Result<Action, Error> {
    tracing::info!("reconciling");

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: format!("onion-service-{}", object.metadata.name.as_ref().unwrap()).into(),
            labels: Some(BTreeMap::from([
                ("app.kubernetes.io/instance".into(), "tor".into()),
                ("app.kubernetes.io/managed-by".into(), "tor-operator".into()),
                ("app.kubernetes.io/name".into(), "tor".into()),
            ])),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([
                    ("app.kubernetes.io/instance".into(), "tor".into()),
                    ("app.kubernetes.io/name".into(), "tor".into()),
                ])),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(BTreeMap::from([
                        ("app.kubernetes.io/instance".into(), "tor".into()),
                        ("app.kubernetes.io/managed-by".into(), "tor-operator".into()),
                        ("app.kubernetes.io/name".into(), "tor".into()),
                    ])),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        image: Some("nginx:1.16.0".into()),
                        image_pull_policy: Some("IfNotPresent".into()),
                        liveness_probe: Some(Probe {
                            failure_threshold: Some(3),
                            http_get: Some(HTTPGetAction {
                                path: Some("/".into()),
                                port: IntOrString::String("http".into()),
                                scheme: Some("HTTP".into()),
                                ..Default::default()
                            }),
                            period_seconds: Some(10),
                            success_threshold: Some(1),
                            timeout_seconds: Some(1),
                            ..Default::default()
                        }),
                        name: "nginx".into(),
                        ports: Some(vec![ContainerPort {
                            container_port: 80,
                            name: Some("http".into()),
                            protocol: Some("TCP".into()),
                            ..Default::default()
                        }]),
                        readiness_probe: Some(Probe {
                            failure_threshold: Some(3),
                            http_get: Some(HTTPGetAction {
                                path: Some("/".into()),
                                port: IntOrString::String("http".into()),
                                scheme: Some("HTTP".into()),
                                ..Default::default()
                            }),
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
