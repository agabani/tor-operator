use std::net::SocketAddr;

use axum::{Router, routing::get};
use tokio::{net::TcpListener, signal};

/// # Errors
///
/// Returns an error if the TCP listener fails to bind or the server encounters an I/O error.
pub async fn run(addr: SocketAddr) -> std::io::Result<()> {
    let app = Router::new()
        .route("/livez", get(handler))
        .route("/readyz", get(handler));

    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(addr =% addr, "server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server stopped");

    Ok(())
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
