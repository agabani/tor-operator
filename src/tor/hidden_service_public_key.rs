use std::ops::Deref;

use super::{
    constants::{ED25519_V1_PUBLIC_TYPE_0_KEY, ED25519_V1_PUBLIC_TYPE_0_LENGTH, VERSION_LENGTH},
    Error, PublicKey, Result,
};

pub struct HiddenServicePublicKey(Data);

pub enum Data {
    Ed25519V1Type0([u8; ED25519_V1_PUBLIC_TYPE_0_LENGTH]),
}

impl Deref for HiddenServicePublicKey {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&PublicKey> for HiddenServicePublicKey {
    fn from(public_key: &PublicKey) -> Self {
        Self(Data::Ed25519V1Type0(public_key.to_bytes()))
    }
}

impl TryFrom<&[u8]> for HiddenServicePublicKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < VERSION_LENGTH {
            return Err(Error::ParseError(format!(
                "expected {} byte version, found {} bytes",
                VERSION_LENGTH,
                bytes.len()
            )));
        }

        let (version, public) = bytes.split_at(VERSION_LENGTH);

        let data = match version {
            ED25519_V1_PUBLIC_TYPE_0_KEY => {
                if public.len() != ED25519_V1_PUBLIC_TYPE_0_LENGTH {
                    return Err(Error::ParseError(format!(
                        "expected {} byte public key, found {} bytes",
                        ED25519_V1_PUBLIC_TYPE_0_LENGTH,
                        public.len()
                    )));
                }
                Data::Ed25519V1Type0(
                    public[0..ED25519_V1_PUBLIC_TYPE_0_LENGTH]
                        .try_into()
                        .expect("failed to convert"),
                )
            }
            _ => return Err(Error::ParseError("unrecognized version".to_string())),
        };

        Ok(Self(data))
    }
}

impl TryFrom<&Vec<u8>> for HiddenServicePublicKey {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> std::result::Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

/*
 * ============================================================================
 *
 * ============================================================================
 */
impl From<&HiddenServicePublicKey> for Vec<u8> {
    fn from(value: &HiddenServicePublicKey) -> Self {
        match value.0 {
            Data::Ed25519V1Type0(data) => [ED25519_V1_PUBLIC_TYPE_0_KEY, &data].concat(),
        }
    }
}
