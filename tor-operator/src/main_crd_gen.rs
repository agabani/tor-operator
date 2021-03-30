fn main() {
    println!(
        "{}",
        serde_yaml::to_string(&tor_operator::TorHiddenService::crd()).unwrap()
    );
}
