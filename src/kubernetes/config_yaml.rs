use sha2::{Digest, Sha256};

use super::constants::TOR_AGABANI_CO_UK_CONFIG_HASH_KEY;

pub struct ConfigYaml(String);

impl ConfigYaml {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl ConfigYaml {
    #[must_use]
    pub fn to_annotation_tuple(&self) -> (String, String) {
        (TOR_AGABANI_CO_UK_CONFIG_HASH_KEY.into(), self.sha_256())
    }

    #[must_use]
    pub fn sha_256(&self) -> String {
        let mut sha = Sha256::new();
        sha.update(&self.0);
        format!("sha256:{:x}", sha.finalize())
    }
}

impl From<ConfigYaml> for String {
    fn from(value: ConfigYaml) -> Self {
        value.0
    }
}

impl From<&ConfigYaml> for String {
    fn from(value: &ConfigYaml) -> Self {
        value.0.clone()
    }
}
