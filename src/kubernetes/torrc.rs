use sha2::{Digest, Sha256};

use super::constants::TOR_AGABANI_CO_UK_TORRC_HASH_KEY;

pub struct Torrc(String);

impl Torrc {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl Torrc {
    #[must_use]
    pub fn to_annotation_tuple(&self) -> (String, String) {
        (TOR_AGABANI_CO_UK_TORRC_HASH_KEY.into(), self.sha_256())
    }

    #[must_use]
    pub fn sha_256(&self) -> String {
        let mut sha = Sha256::new();
        sha.update(&self.0);
        format!("sha256:{:x}", sha.finalize())
    }
}

impl From<Torrc> for String {
    fn from(value: Torrc) -> Self {
        value.0
    }
}

impl From<&Torrc> for String {
    fn from(value: &Torrc) -> Self {
        value.0.clone()
    }
}
