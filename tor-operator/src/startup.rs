use crate::operator::Operator;
use actix_web::{web, App, HttpResponse, HttpServer};
use kube::Client;

#[tracing::instrument]
pub async fn run() {
    let client = Client::try_default()
        .await
        .expect("Failed to create client");
    let (operator, drainer) = Operator::new(client).await;

    let server = HttpServer::new(move || {
        App::new()
            .route("/health/liveness", web::get().to(HttpResponse::Ok))
            .route("/health/readiness", web::get().to(HttpResponse::Ok))
            .data(operator.clone())
    })
    .bind("0.0.0.0:8080")
    .expect("Failed to bind to 0.0.0.0:8080")
    .shutdown_timeout(0)
    .run();

    tokio::select! {
        result = drainer => {
            tracing::info!(result = ?result, "Drainer terminated");
        },
        result = server => {
            tracing::info!(result = ?result, "Server terminated");
        },
    }
}
