pub struct OBConfig(String);

impl OBConfig {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl From<OBConfig> for String {
    fn from(value: OBConfig) -> Self {
        value.0
    }
}

impl From<&OBConfig> for String {
    fn from(value: &OBConfig) -> Self {
        value.0.clone()
    }
}
