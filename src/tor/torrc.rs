use crate::kubernetes::Annotation;

pub struct Torrc(String);

impl Torrc {
    #[must_use]
    pub fn builder() -> TorrcBuilder {
        TorrcBuilder(Vec::new())
    }
}

impl Annotation<'_> for Torrc {
    const NAME: &'static str = "torrc";
}

impl<'a> From<&'a Torrc> for std::borrow::Cow<'a, str> {
    fn from(value: &'a Torrc) -> Self {
        std::borrow::Cow::Borrowed(&value.0)
    }
}

impl std::fmt::Display for Torrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct TorrcBuilder(Vec<String>);

impl TorrcBuilder {
    #[must_use]
    pub fn build(&self) -> Torrc {
        Torrc(self.0.join("\n"))
    }

    /// 127.0.0.1:6666
    #[must_use]
    pub fn control_port(mut self, port: &str) -> Self {
        self.0.push(format!("ControlPort {port}"));
        self
    }

    /// `~/.tor`
    #[must_use]
    pub fn data_dir(mut self, dir: &str) -> Self {
        self.0.push(format!("DataDirectory {dir}"));
        self
    }

    /// `/var/lib/tor/hidden_service`
    #[must_use]
    pub fn hidden_service_dir(mut self, dir: &str) -> Self {
        self.0.push(format!("HiddenServiceDir {dir}"));
        self
    }

    #[must_use]
    pub fn hidden_service_onion_balance_instance(mut self, enabled: bool) -> Self {
        self.0.push(format!(
            "HiddenServiceOnionbalanceInstance {}",
            i32::from(enabled)
        ));
        self
    }

    #[must_use]
    pub fn hidden_service_port(mut self, virtport: i32, target: &str) -> Self {
        self.0
            .push(format!("HiddenServicePort {virtport} {target}"));
        self
    }

    /// 1080
    /// 0.0.0.0:1080
    #[must_use]
    pub fn http_tunnel_port(mut self, addr: &str) -> Self {
        self.0.push(format!("HTTPTunnelPort {addr}"));
        self
    }

    /// 9050
    /// 0.0.0.0:9050
    #[must_use]
    pub fn socks_port(mut self, addr: &str) -> Self {
        self.0.push(format!("SocksPort {addr}"));
        self
    }

    #[must_use]
    pub fn template(mut self, template: &str) -> Self {
        self.0.push(template.to_string());
        self
    }
}
