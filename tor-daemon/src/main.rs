use tor_daemon::run;
use tracing::subscriber;
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

#[actix_web::main]
#[tracing::instrument]
async fn main() -> Result<(), ()> {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(ChronoUtc::rfc3339())
        .json();

    let registry = Registry::default().with(filter_layer).with(fmt_layer);

    subscriber::set_global_default(registry).expect("setting tracing default failed.");

    tracing::info!("Application started");

    run().await;

    tracing::info!("Application terminated");

    Ok(())
}
