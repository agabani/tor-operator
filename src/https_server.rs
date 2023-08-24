use std::net::SocketAddr;

use axum::{extract::rejection::JsonRejection, http::StatusCode, routing::post, Json, Router};
use hyper::server::conn::AddrIncoming;
use hyper_rustls::TlsAcceptor;
use kube::core::{
    admission::{AdmissionRequest, AdmissionResponse, AdmissionReview},
    DynamicObject,
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
) -> (StatusCode, Json<AdmissionResponse>) {
    match review {
        Ok(Json(review)) => {
            tracing::info!(review =? review, "review");
            let request: AdmissionRequest<_> = review.try_into().unwrap();
            let response = AdmissionResponse::from(&request);
            tracing::info!(request =? request, response =? response, "request");
            (StatusCode::OK, Json(response))
        }
        Err(rejection) => {
            tracing::warn!(rejection =? rejection,  "rejection");
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(AdmissionResponse::invalid("reason")),
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
