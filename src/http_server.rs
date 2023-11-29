use std::{net::SocketAddr, pin::pin, sync::Arc, time::Duration};

use axum::{
    extract::{Request, State},
    routing::get,
    Router,
};
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use prometheus::Encoder;
use tokio::{net::TcpListener, signal, sync::watch, time::sleep};
use tower::Service;

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

    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::info!(addr =% addr, "server started");

    let (close_tx, close_rx) = watch::channel(());

    loop {
        let (tcp_stream, remote_addr) = tokio::select! {
            result = listener.accept() => {
                result.unwrap()
            }
            () = shutdown_signal() => {
                tracing::info!("shutdown signal received, not accepting new connections");
                break;
            }
        };

        let tower_service = app.clone();
        let close_rx = close_rx.clone();

        tokio::spawn(async move {
            let tcp_stream = TokioIo::new(tcp_stream);

            let hyper_service =
                service_fn(move |request: Request<Incoming>| tower_service.clone().call(request));

            let builder = Builder::new(TokioExecutor::new());

            let mut connection =
                pin!(builder.serve_connection_with_upgrades(tcp_stream, hyper_service));

            tokio::select! {
                result = connection.as_mut() => {
                    if let Err(error) = result {
                        tracing::warn!(error =% error, "failed to serve connection");
                    }
                }
                () = shutdown_signal() => {
                    tokio::select! {
                        result = connection.as_mut() => {
                            if let Err(error) = result {
                                tracing::warn!(error =% error, "failed to serve connection");
                            }
                        }
                        () = sleep(Duration::from_secs(30)) => {}
                    }
                }
            }

            tracing::debug!(remote_addr =% remote_addr, "connection closed");

            drop(close_rx);
        });
    }

    drop(close_rx);
    drop(listener);
    close_tx.closed().await;

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
}
