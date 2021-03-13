use crate::configuration::Configuration;
use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

pub fn run() {
    let configuration = Configuration::load().expect("Failed to load configuration.");

    Tor::new()
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
        .start()
        .unwrap();
}
