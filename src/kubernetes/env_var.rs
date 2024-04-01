use k8s_openapi::api::core::v1::EnvVarSource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// EnvVar represents an environment variable present in a Container.
#[allow(clippy::doc_markdown)]
#[derive(JsonSchema, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    /// Variable references $(VAR_NAME) are expanded using the previously defined environment variables in the container and any service environment variables. If a variable cannot be resolved, the reference in the input string will be unchanged. Double $$ are reduced to a single $, which allows for escaping the $(VAR_NAME) syntax: i.e. "$$(VAR_NAME)" will produce the string literal "$(VAR_NAME)". Escaped references will never be expanded, regardless of whether the variable exists or not. Defaults to "".
    pub value: Option<String>,

    /// Source for the environment variable's value. Cannot be used if value is not empty.
    pub value_from: Option<EnvVarSource>,
}

impl EnvVar {
    pub fn into_env_var(self, name: String) -> k8s_openapi::api::core::v1::EnvVar {
        k8s_openapi::api::core::v1::EnvVar {
            name,
            value: self.value,
            value_from: self.value_from,
        }
    }
}
