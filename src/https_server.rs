use std::net::SocketAddr;

use axum::Router;
use tokio::signal;

#[allow(clippy::missing_panics_doc)]
pub async fn run(addr: SocketAddr) {
    let app = Router::new();

    let server = hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal());

    tracing::info!(addr =% addr, "server started");

    server.await.unwrap();

    tracing::info!("server stopped");
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
