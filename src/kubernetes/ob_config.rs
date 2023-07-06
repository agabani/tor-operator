use super::Annotation;

pub struct OBConfig(String);

impl OBConfig {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for OBConfig {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&OBConfig> for String {
    fn from(value: &OBConfig) -> Self {
        value.0.clone()
    }
}

impl Annotation for OBConfig {
    const NAME: &'static str = "ob-config";
}
