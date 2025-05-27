use std::net::SocketAddr;

use axum::{Router, routing::get};
use tokio::{net::TcpListener, signal};

#[allow(clippy::missing_panics_doc)]
pub async fn run(addr: SocketAddr) {
    let app = Router::new()
        .route("/livez", get(handler))
        .route("/readyz", get(handler));

    let listener = TcpListener::bind(&addr).await.unwrap();

    tracing::info!(addr =% addr, "server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    tracing::info!("server stopped");
}

#[allow(clippy::unused_async)]
async fn handler() {}

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
