use actix_web::{web, App, HttpResponse, HttpServer};

pub async fn run() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health/liveness", web::get().to(HttpResponse::Ok))
            .route("/health/readiness", web::get().to(HttpResponse::Ok))
    })
    .bind("0.0.0.0:8080")
    .expect("Failed to bind to 0.0.0.0:8080")
    .shutdown_timeout(0)
    .run();

    tokio::select! {
        result = server => {
            println!("server stopped: {:?}", result)
        },
    }
}
