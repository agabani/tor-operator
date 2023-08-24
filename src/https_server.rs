use std::net::SocketAddr;

use axum::{extract::rejection::JsonRejection, http::StatusCode, routing::post, Json, Router};
use hyper::server::conn::AddrIncoming;
use hyper_rustls::TlsAcceptor;
use k8s_openapi::api::core::v1::{Container, ContainerPort, ExecAction, Pod, Probe};
use kube::{
    core::{
        admission::{AdmissionRequest, AdmissionResponse, AdmissionReview},
        DynamicObject,
    },
    Resource, ResourceExt,
};
use tokio::signal;

#[allow(clippy::missing_panics_doc)]
pub async fn run(addr: SocketAddr, certs: Vec<rustls::Certificate>, key: rustls::PrivateKey) {
    let incoming = AddrIncoming::bind(&addr).unwrap();
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(certs, key)
        .unwrap()
        .with_all_versions_alpn()
        .with_incoming(incoming);

    let app = Router::new().route("/mutate", post(handler));

    let server = hyper::Server::builder(acceptor)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    tracing::info!(addr =% addr, "server started");

    server.await.unwrap();

    tracing::info!("server stopped");
}

#[allow(clippy::unused_async)]
async fn handler(
    review: Result<Json<AdmissionReview<DynamicObject>>, JsonRejection>,
) -> (StatusCode, Json<AdmissionReview<DynamicObject>>) {
    match review {
        Ok(Json(review)) => {
            tracing::info!(review =? review, "review");

            let request: AdmissionRequest<DynamicObject> = review.try_into().unwrap();
            let mut response = AdmissionResponse::from(&request);

            if let Some(object) = &request.object {
                // If the resource doesn't contain "admission", we add it to the resource.
                if !object.labels().contains_key("admission") {
                    let mut patches = Vec::new();

                    // Ensure labels exists before adding a key to it
                    if object.meta().labels.is_none() {
                        patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
                            path: "/metadata/labels".into(),
                            value: serde_json::json!({}),
                        }));
                    }

                    // Add our label
                    patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
                        path: "/metadata/labels/admission".into(),
                        value: serde_json::Value::String("modified-by-admission-controller".into()),
                    }));

                    tracing::info!(patches =? patches, "patches");

                    response = response.with_patch(json_patch::Patch(patches)).unwrap();
                }

                // If the resource doesn't contain "container", we add it to the resource.
                let pod: Result<Pod, _> = DynamicObject::try_parse(object.clone());
                let pod = pod.unwrap();
                if !pod
                    .spec
                    .as_ref()
                    .unwrap()
                    .containers
                    .iter()
                    .any(|f| f.name == "tor")
                {
                    let mut patches = Vec::new();

                    let value = serde_json::to_value(Container {
                        args: Some(vec!["-c".into(), vec!["tor"].join(" && ")]),
                        command: Some(vec!["/bin/bash".into()]),
                        image: Some("ghcr.io/agabani/tor-operator:tor-0.4.7.14".to_string()),
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
                        name: "tor".to_string(),
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
                    })
                    .unwrap();

                    patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
                        path: "/spec/containers/-".into(),
                        value,
                    }));

                    response = response.with_patch(json_patch::Patch(patches)).unwrap();
                }
            }

            // let pod = DynamicObject::try_parse::<Pod>(request.object.clone().unwrap()).unwrap();
            // let x = request.object.unwrap();

            // if pod
            //     .spec
            //     .as_ref()
            //     .unwrap()
            //     .containers
            //     .iter()
            //     .any(|f| f.name == "tor")
            // {}

            // let response: AdmissionReview<DynamicObject> = AdmissionResponse::from(&request)
            //     // .with_patch(j)
            //     .into_review();

            // tracing::info!(pod =? pod, "request");
            // tracing::info!(pod =? pod, "request");
            tracing::info!(request =? request, response =? response, "request");
            (StatusCode::OK, Json(response.into_review()))
        }
        Err(rejection) => {
            tracing::warn!(rejection =? rejection,  "rejection");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(AdmissionResponse::invalid("reason").into_review()),
            )
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
