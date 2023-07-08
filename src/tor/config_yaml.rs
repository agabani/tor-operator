use crate::kubernetes::Annotation;

#[derive(serde::Deserialize, serde:: Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigYaml {
    pub services: Vec<ConfigYamlService>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigYamlService {
    pub instances: Vec<ConfigYamlServiceInstance>,

    pub key: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigYamlServiceInstance {
    pub address: String,

    pub name: String,
}

impl Annotation<'_> for ConfigYaml {
    const NAME: &'static str = "config-yaml";
}

impl<'a> From<&'a ConfigYaml> for std::borrow::Cow<'a, str> {
    fn from(value: &'a ConfigYaml) -> Self {
        std::borrow::Cow::Owned(value.to_string())
    }
}

impl std::fmt::Display for ConfigYaml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_yaml::to_string(self).unwrap())
    }
}
