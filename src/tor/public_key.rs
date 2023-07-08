use std::ops::Deref;

use super::{
    hidden_service_public_key::Data, Error, ExpandedSecretKey, HiddenServicePublicKey, Result,
};

#[derive(PartialEq)]
pub struct PublicKey(ed25519_dalek::PublicKey);

impl Deref for PublicKey {
    type Target = ed25519_dalek::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&ExpandedSecretKey> for PublicKey {
    fn from(value: &ExpandedSecretKey) -> Self {
        Self(ed25519_dalek::PublicKey::from(&**value))
    }
}

impl TryFrom<&HiddenServicePublicKey> for PublicKey {
    type Error = Error;

    fn try_from(value: &HiddenServicePublicKey) -> Result<Self, Self::Error> {
        match &**value {
            Data::Ed25519V1Type0(data) => ed25519_dalek::PublicKey::from_bytes(data)
                .map(Self)
                .map_err(Error::SignatureError),
        }
    }
}
