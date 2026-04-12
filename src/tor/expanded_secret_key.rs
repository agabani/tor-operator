use std::ops::Deref;

use rand::{Rng as _, SeedableRng as _};

use super::{Error, HiddenServiceSecretKey, Result, hidden_service_secret_key::Data};

pub struct ExpandedSecretKey(ed25519_dalek::hazmat::ExpandedSecretKey);

impl ExpandedSecretKey {
    /// # Panics
    ///
    /// Panics if the system entropy source cannot be used to seed the RNG.
    #[must_use]
    pub fn generate() -> Self {
        let mut csprng = rand::rngs::StdRng::try_from_rng(&mut rand::rngs::SysRng)
            .expect("failed to seed StdRng from system entropy source");

        // upstream reference: https://github.com/dalek-cryptography/curve25519-dalek/blob/ed25519-2.2.0/ed25519-dalek/src/signing.rs#L183-L206
        let mut secret = ed25519_dalek::SecretKey::default();
        csprng.fill_bytes(&mut secret);
        let secret_key = ed25519_dalek::SigningKey::from_bytes(&secret);

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
                        data[0..32].try_into().expect("incorrect scalar slice length"),
                    ),
                },
            )),
        }
    }
}
