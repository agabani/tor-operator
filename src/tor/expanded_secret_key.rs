use std::ops::Deref;

use super::{hidden_service_secret_key::Data, Error, HiddenServiceSecretKey, Result};

pub struct ExpandedSecretKey(ed25519_dalek::hazmat::ExpandedSecretKey);

impl ExpandedSecretKey {
    #[must_use]
    pub fn generate() -> Self {
        let mut csprng = rand_08::rngs::OsRng;
        let secret_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self(secret_key.as_bytes().into())
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut output = [0_u8; 64];
        output[0..32].copy_from_slice(&self.scalar.to_bytes());
        output[32..64].copy_from_slice(&self.hash_prefix);
        output
    }
}

impl Deref for ExpandedSecretKey {
    type Target = ed25519_dalek::hazmat::ExpandedSecretKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&HiddenServiceSecretKey> for ExpandedSecretKey {
    type Error = Error;

    fn try_from(value: &HiddenServiceSecretKey) -> Result<Self, Self::Error> {
        match &**value {
            Data::Ed25519V1Type0(data) => Ok(ExpandedSecretKey(
                ed25519_dalek::hazmat::ExpandedSecretKey {
                    hash_prefix: data[32..64].try_into().expect("incorrect hash prefix slice length"),
                    #[allow(deprecated)] // bytes from hs_ed25519_secret_key must be loaded into expanded secret key without modification
                    scalar: curve25519_dalek::Scalar::from_bits(
                        data[0..32].try_into().expect("incorrect scaler slice length"),
                    ),
                },
            )),
        }
    }
}
