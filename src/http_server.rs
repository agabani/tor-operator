use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, routing::get, Router, Server};
use prometheus::Encoder;
use tokio::signal;

use crate::metrics::Metrics;

struct AppState {
    metrics: Metrics,
}

#[allow(clippy::missing_panics_doc)]
pub async fn run(addr: SocketAddr, metrics: Metrics) {
    let app = Router::new()
        .route("/livez", get(handler))
        .route("/metrics", get(metrics_handler))
        .route("/readyz", get(handler))
        .with_state(Arc::new(AppState { metrics }));

    let server = Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    tracing::info!(addr =% addr, "server started");

    server.await.unwrap();

    tracing::info!("server stopped");
}

#[allow(clippy::unused_async)]
async fn handler() {}

#[allow(clippy::unused_async)]
async fn metrics_handler(State(state): State<Arc<AppState>>) -> String {
    let mut buffer = vec![];
    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics.registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8_lossy(&buffer).into()
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
