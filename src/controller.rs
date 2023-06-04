use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::{future, StreamExt};
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec},
        core::v1::{
            ConfigMap, ConfigMapVolumeSource, Container, EmptyDirVolumeSource, ExecAction,
            KeyToPath, PodSpec, PodTemplateSpec, Probe, SecretVolumeSource, Volume, VolumeMount,
        },
    },
    apimachinery::pkg::apis::meta::v1::LabelSelector,
};
use kube::{
    api::{Patch, PatchParams},
    core::ObjectMeta,
    runtime::{controller::Action, watcher::Config, Controller},
    Api, Client, Resource,
};
use sha2::{Digest, Sha256};

use crate::crd::OnionService;

#[allow(clippy::missing_panics_doc)]
pub async fn run(
    busybox_image_pull_policy: &str,
    busybox_image_uri: &str,
    tor_image_pull_policy: &str,
    tor_image_uri: &str,
) {
    let client = Client::try_default().await.unwrap();

    let onion_services = Api::<OnionService>::all(client.clone());
    let config_maps = Api::<ConfigMap>::all(client.clone());
    let deployments = Api::<Deployment>::all(client.clone());

    let context = Arc::new(Context {
        client,
        busybox_image_pull_policy: busybox_image_pull_policy.into(),
        busybox_image_uri: busybox_image_uri.into(),
        tor_image_pull_policy: tor_image_pull_policy.into(),
        tor_image_uri: tor_image_uri.into(),
    });

    Controller::new(onion_services, Config::default())
        .owns(config_maps, Config::default())
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
    busybox_image_pull_policy: String,
    busybox_image_uri: String,
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

    let mut torrc = vec!["HiddenServiceDir /var/lib/tor/hidden_service".into()];
    for port in &object.spec.hidden_service_ports {
        torrc.push(format!(
            "HiddenServicePort {} {}",
            port.virtport, port.target
        ));
    }
    let torrc_content = torrc.join("\n");
    let torrc_content_hash = {
        let mut sha = Sha256::new();
        sha.update(&torrc_content);
        format!("sha256:{:x}", sha.finalize())
    };

    let config_map = ConfigMap {
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
            annotations: Some(BTreeMap::from([(
                "tor.agabani.co.uk/torrc-hash".into(),
                torrc_content_hash.to_string(),
            )])),
            owner_references: Some(vec![object.controller_owner_ref(&()).unwrap()]),
            ..Default::default()
        },
        data: Some(BTreeMap::from([("torrc".into(), torrc_content)])),
        ..Default::default()
    };

    let config_maps: Api<ConfigMap> = Api::namespaced(
        ctx.client.clone(),
        object.metadata.namespace.as_ref().unwrap(),
    );

    config_maps
        .patch(
            config_map
                .metadata
                .name
                .as_ref()
                .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?,
            &PatchParams::apply("onionservices.tor.agabani.co.uk").force(),
            &Patch::Apply(&config_map),
        )
        .await
        .map_err(Error::Kube)?;

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
            annotations: Some(BTreeMap::from([(
                "tor.agabani.co.uk/torrc-hash".into(),
                torrc_content_hash.to_string(),
            )])),
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
                    annotations: Some(BTreeMap::from([(
                        "tor.agabani.co.uk/torrc-hash".into(),
                        torrc_content_hash.to_string(),
                    )])),
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
                        volume_mounts: Some(vec![
                            VolumeMount {
                                mount_path: "/usr/local/etc/tor".into(),
                                name: "usr-local-etc-tor".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                            VolumeMount {
                                mount_path: "/var/lib/tor".into(),
                                name: "var-lib-tor".into(),
                                read_only: Some(false),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }],
                    init_containers: Some(vec![Container {
                        image: Some(ctx.busybox_image_uri.clone()),
                        image_pull_policy: Some(ctx.busybox_image_pull_policy.clone()),
                        name: "busybox".into(),
                        command: Some(vec!["/bin/sh".into()]),
                        args: Some(vec![
                            "-c".into(),
                            "mkdir -p /var/lib/tor/hidden_service && chmod 400 /var/lib/tor/hidden_service && cp /etc/secrets/* /var/lib/tor/hidden_service".into(),
                        ]),
                        volume_mounts: Some(vec![
                            VolumeMount {
                                mount_path: "/etc/secrets".into(),
                                name: "etc-secrets".into(),
                                read_only: Some(true),
                                ..Default::default()
                            },
                            VolumeMount {
                                mount_path: "/var/lib/tor".into(),
                                name: "var-lib-tor".into(),
                                read_only: Some(false),
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }]),
                    volumes: Some(vec![
                        Volume {
                            name: "usr-local-etc-tor".into(),
                            config_map: Some(ConfigMapVolumeSource {
                                default_mode: Some(0o400),
                                items: Some(vec![KeyToPath {
                                    key: "torrc".into(),
                                    mode: Some(0o400),
                                    path: "torrc".into(),
                                }]),
                                name: Some(config_map
                                    .metadata
                                    .name
                                    .as_ref()
                                    .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))?
                                    .clone()
                                ),
                                optional: Some(false),
                            }),
                            ..Default::default()
                        },
                        Volume {
                            name: "var-lib-tor".into(),
                            empty_dir: Some(EmptyDirVolumeSource {
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
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
                                secret_name: Some(object.spec.secret_name.clone()),
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
