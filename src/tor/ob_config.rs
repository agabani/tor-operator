use crate::kubernetes::Annotation;

use super::Hostname;

pub struct OBConfig(String);

impl OBConfig {
    #[must_use]
    pub fn builder() -> OBConfigBuilder {
        OBConfigBuilder(Vec::new())
    }
}

impl Annotation<'_> for OBConfig {
    const NAME: &'static str = "ob-config";
}

impl<'a> From<&'a OBConfig> for std::borrow::Cow<'a, str> {
    fn from(value: &'a OBConfig) -> Self {
        std::borrow::Cow::Borrowed(&value.0)
    }
}

impl std::fmt::Display for OBConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct OBConfigBuilder(Vec<String>);

impl OBConfigBuilder {
    #[must_use]
    pub fn build(&self) -> OBConfig {
        OBConfig(self.0.join("\n"))
    }

    #[must_use]
    pub fn master_onion_address(mut self, hostname: &Hostname) -> Self {
        self.0.push(format!("MasterOnionAddress {hostname}"));
        self
    }
}
