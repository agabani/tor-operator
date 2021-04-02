use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, HTTPGetAction, PodSpec, PodTemplateSpec, Probe,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{DeleteParams, Meta, ObjectMeta, PostParams};
use kube::Api;
use std::collections::BTreeMap;

pub struct DaemonDeployment {
    pub name: String,
    pub virtual_port: u16,
    pub target_address: String,
    pub target_port: u16,
}

pub async fn create_or_update(api: Api<Deployment>, deployment: &DaemonDeployment) {
    let name = name(deployment);
    let mut new_document = document(deployment);
    let param = PostParams::default();

    match api.get(&name).await {
        Ok(existing_document) => {
            new_document.metadata.resource_version = existing_document.resource_ver();
            api.replace(&name, &param, &new_document).await.unwrap();
        }
        Err(kube::Error::Api(error_response)) => {
            if let 404 = error_response.code {
                api.create(&param, &new_document).await.unwrap();
            }
        }
        _ => {}
    }
}

pub async fn destroy(api: Api<Deployment>, deployment: &DaemonDeployment) {
    let params = DeleteParams::default();
    api.delete(&name(deployment), &params).await.unwrap();
}

fn name(deployment: &DaemonDeployment) -> String {
    format!("{}-tor-daemon", deployment.name)
}

fn document(deployment: &DaemonDeployment) -> Deployment {
    let name = name(deployment);

    let mut labels = BTreeMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), name.clone());
    labels.insert("app.kubernetes.io/instance".to_string(), name.clone());

    Deployment {
        metadata: ObjectMeta {
            labels: Some(labels.clone()),
            name: Some(name),
            ..ObjectMeta::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: LabelSelector {
                match_expressions: None,
                match_labels: Some(labels.clone()),
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..ObjectMeta::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        env: Some(vec![
                            EnvVar {
                                name: "app_virtual_port".to_string(),
                                value: Some(deployment.virtual_port.to_string()),
                                value_from: None,
                            },
                            EnvVar {
                                name: "app_target_address".to_string(),
                                value: Some(deployment.target_address.to_string()),
                                value_from: None,
                            },
                            EnvVar {
                                name: "app_target_port".to_string(),
                                value: Some(deployment.target_port.to_string()),
                                value_from: None,
                            },
                        ]),
                        image: Some("agabani/tor-operator:daemon-latest".to_string()),
                        image_pull_policy: Some("Always".to_string()),
                        liveness_probe: Some(Probe {
                            http_get: Some(HTTPGetAction {
                                path: Some("/health/liveness".to_string()),
                                port: IntOrString::String("http".to_string()),
                                ..HTTPGetAction::default()
                            }),
                            ..Probe::default()
                        }),
                        name: "daemon".to_string(),
                        ports: Some(vec![ContainerPort {
                            container_port: 8080,
                            name: Some("http".to_string()),
                            protocol: Some("TCP".to_string()),
                            ..ContainerPort::default()
                        }]),
                        readiness_probe: Some(Probe {
                            http_get: Some(HTTPGetAction {
                                path: Some("/health/readiness".to_string()),
                                port: IntOrString::String("http".to_string()),
                                ..HTTPGetAction::default()
                            }),
                            ..Probe::default()
                        }),
                        ..Container::default()
                    }],
                    ..PodSpec::default()
                }),
            },
            ..DeploymentSpec::default()
        }),
        ..Deployment::default()
    }
}
