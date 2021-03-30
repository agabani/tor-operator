use tor_daemon::run;

#[actix_web::main]
async fn main() -> Result<(), ()> {
    run().await;
    Ok(())
}
