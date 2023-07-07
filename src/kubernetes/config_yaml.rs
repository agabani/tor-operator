use super::Annotation;

pub struct ConfigYaml(String);

impl ConfigYaml {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ConfigYaml {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&ConfigYaml> for String {
    fn from(value: &ConfigYaml) -> Self {
        value.0.clone()
    }
}

impl Annotation for ConfigYaml {
    const NAME: &'static str = "config-yaml";
}
