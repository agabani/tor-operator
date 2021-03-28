use tor_operator::run;

#[actix_web::main]
async fn main() -> Result<(), ()> {
    run().await;
    Ok(())
}
