use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tor torrc settings.
#[allow(clippy::module_name_repetitions)]
#[derive(JsonSchema, Deserialize, Serialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Torrc {
    /// The template to be prepended to the torrc file.
    pub template: Option<String>,
}
