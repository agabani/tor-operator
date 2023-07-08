use std::ops::Deref;

use super::{hidden_service_secret_key::Data, Error, HiddenServiceSecretKey, Result};

pub struct ExpandedSecretKey(ed25519_dalek::ExpandedSecretKey);

impl ExpandedSecretKey {
    #[must_use]
    pub fn generate() -> Self {
        let mut csprng = rand_07::rngs::OsRng {};
        let secret_key = ed25519_dalek::SecretKey::generate(&mut csprng);
        Self(ed25519_dalek::ExpandedSecretKey::from(&secret_key))
    }
}

impl Deref for ExpandedSecretKey {
    type Target = ed25519_dalek::ExpandedSecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&HiddenServiceSecretKey> for ExpandedSecretKey {
    type Error = Error;

    fn try_from(value: &HiddenServiceSecretKey) -> Result<Self, Self::Error> {
        match &**value {
            Data::Ed25519V1Type0(data) => ed25519_dalek::ExpandedSecretKey::from_bytes(data)
                .map(Self)
                .map_err(Error::SignatureError),
        }
    }
}
