use libtor::{HiddenServiceVersion, Tor, TorAddress, TorFlag};

pub fn run() {
    Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
        .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
        .flag(TorFlag::HiddenServicePort(
            TorAddress::Port(80),
            Some(TorAddress::AddressPort("127.0.0.1".into(), 8080)).into(),
        ))
        .start()
        .unwrap();
}
