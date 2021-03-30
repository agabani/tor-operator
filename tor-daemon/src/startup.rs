use crate::configuration::Configuration;
use actix_web::{web, App, HttpResponse, HttpServer};
use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

pub async fn run() {
    let configuration = Configuration::load().expect("Failed to load configuration.");

    let server = HttpServer::new(|| {
        App::new()
            .route("/health/liveness", web::get().to(HttpResponse::Ok))
            .route("/health/readiness", web::get().to(HttpResponse::Ok))
    })
    .bind("0.0.0.0:8080")
    .expect("Failed to bind to 0.0.0.0:8080")
    .shutdown_timeout(0)
    .run();

    let tor = Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(configuration.virtual_port),
            Some(TorAddress::AddressPort(
                configuration.target_address,
                configuration.target_port,
            ))
            .into(),
        ))
        .start_background();

    tokio::select! {
        result = server => {
            println!("server stopped: {:?}", result)
        },
        result = async { tor.join() } => {
            println!("tor stopped: {:?}", result)
        },
    }
}
