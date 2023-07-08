use std::ops::Deref;

use super::{
    constants::{ED25519_V1_SECRET_TYPE_0_KEY, ED25519_V1_SECRET_TYPE_0_LENGTH, VERSION_LENGTH},
    Error, ExpandedSecretKey, Result,
};

pub struct HiddenServiceSecretKey(Data);

impl Deref for HiddenServiceSecretKey {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub enum Data {
    Ed25519V1Type0([u8; ED25519_V1_SECRET_TYPE_0_LENGTH]),
}

impl TryFrom<&[u8]> for HiddenServiceSecretKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < VERSION_LENGTH {
            return Err(Error::ParseError(format!(
                "expected {} byte version, found {} bytes",
                VERSION_LENGTH,
                bytes.len()
            )));
        }

        let (version, secret) = bytes.split_at(VERSION_LENGTH);

        let data = match version {
            ED25519_V1_SECRET_TYPE_0_KEY => {
                if secret.len() != ED25519_V1_SECRET_TYPE_0_LENGTH {
                    return Err(Error::ParseError(format!(
                        "expected {} byte secret key, found {} bytes",
                        ED25519_V1_SECRET_TYPE_0_LENGTH,
                        secret.len()
                    )));
                }
                Data::Ed25519V1Type0(
                    secret[0..ED25519_V1_SECRET_TYPE_0_LENGTH]
                        .try_into()
                        .expect("failed to convert"),
                )
            }
            _ => return Err(Error::ParseError("unrecognized version".to_string())),
        };

        Ok(Self(data))
    }
}

impl TryFrom<&Vec<u8>> for HiddenServiceSecretKey {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> std::result::Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

impl From<&ExpandedSecretKey> for HiddenServiceSecretKey {
    fn from(expanded_secret_key: &ExpandedSecretKey) -> Self {
        Self(Data::Ed25519V1Type0(expanded_secret_key.to_bytes()))
    }
}

/*
 * ============================================================================
 *
 * ============================================================================
 */
impl From<&HiddenServiceSecretKey> for Vec<u8> {
    fn from(value: &HiddenServiceSecretKey) -> Self {
        match value.0 {
            Data::Ed25519V1Type0(data) => [ED25519_V1_SECRET_TYPE_0_KEY, &data].concat(),
        }
    }
}
