use sha3::{Digest, Sha3_256};

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    SignatureError(ed25519_dalek::SignatureError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

const ED25519_V1_PUBLIC_TYPE_0_KEY: &[u8] = b"== ed25519v1-public: type0 ==\0\0\0";
const ED25519_V1_PUBLIC_TYPE_0_LENGTH: usize = 32;
const ED25519_V1_SECRET_TYPE_0_KEY: &[u8] = b"== ed25519v1-secret: type0 ==\0\0\0";
const ED25519_V1_SECRET_TYPE_0_LENGTH: usize = 64;
const HOSTNAME_LENGTH: usize = 62;
const VERSION_LENGTH: usize = 32;

/*
 * ============================================================================
 * Expanded Secret Key
 * ============================================================================
 */
pub struct ExpandedSecretKey(ed25519_dalek::ExpandedSecretKey);

impl ExpandedSecretKey {
    #[must_use]
    pub fn generate() -> Self {
        let mut csprng = rand_07::rngs::OsRng {};
        let secret_key = ed25519_dalek::SecretKey::generate(&mut csprng);
        Self((&secret_key).into())
    }

    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey((&self.0).into())
    }

    /// # Errors
    ///
    /// Returns error if malformed.
    pub fn try_from_hidden_service_secret_key(value: &HiddenServiceSecretKey) -> Result<Self> {
        match value.0 {
            HiddenServiceSecretKeyData::Ed25519V1Type0(data) => {
                ed25519_dalek::ExpandedSecretKey::from_bytes(&data)
                    .map(Self)
                    .map_err(Error::SignatureError)
            }
        }
    }
}

impl TryFrom<&HiddenServiceSecretKey> for ExpandedSecretKey {
    type Error = Error;

    fn try_from(value: &HiddenServiceSecretKey) -> Result<Self, Self::Error> {
        Self::try_from_hidden_service_secret_key(value)
    }
}

/*
 * ============================================================================
 * Hidden Service Public Key
 * ============================================================================
 */
pub struct HiddenServicePublicKey(HiddenServicePublicKeyData);

enum HiddenServicePublicKeyData {
    Ed25519V1Type0([u8; ED25519_V1_PUBLIC_TYPE_0_LENGTH]),
}

impl HiddenServicePublicKey {
    /// # Errors
    ///
    /// Returns error if malformed.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
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
                        "expected {} byte secret, found {} bytes",
                        ED25519_V1_PUBLIC_TYPE_0_LENGTH,
                        public.len()
                    )));
                }
                HiddenServicePublicKeyData::Ed25519V1Type0(
                    public[0..ED25519_V1_PUBLIC_TYPE_0_LENGTH]
                        .try_into()
                        .expect("failed to convert"),
                )
            }
            _ => return Err(Error::ParseError("unrecognized version".to_string())),
        };

        Ok(Self(data))
    }

    #[must_use]
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        Self(HiddenServicePublicKeyData::Ed25519V1Type0(
            public_key.0.to_bytes(),
        ))
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.0 {
            HiddenServicePublicKeyData::Ed25519V1Type0(data) => {
                [ED25519_V1_PUBLIC_TYPE_0_KEY, &data].concat()
            }
        }
    }
}

impl TryFrom<&[u8]> for HiddenServicePublicKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(bytes)
    }
}

impl From<&PublicKey> for HiddenServicePublicKey {
    fn from(public_key: &PublicKey) -> Self {
        Self::from_public_key(public_key)
    }
}

/*
 * ============================================================================
 * Hidden Service Secret Key
 * ============================================================================
 */
pub struct HiddenServiceSecretKey(HiddenServiceSecretKeyData);

enum HiddenServiceSecretKeyData {
    Ed25519V1Type0([u8; ED25519_V1_SECRET_TYPE_0_LENGTH]),
}

impl HiddenServiceSecretKey {
    /// # Errors
    ///
    /// Returns error if malformed.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
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
                        "expected {} byte secret, found {} bytes",
                        ED25519_V1_SECRET_TYPE_0_LENGTH,
                        secret.len()
                    )));
                }
                HiddenServiceSecretKeyData::Ed25519V1Type0(
                    secret[0..ED25519_V1_SECRET_TYPE_0_LENGTH]
                        .try_into()
                        .expect("failed to convert"),
                )
            }
            _ => return Err(Error::ParseError("unrecognized version".to_string())),
        };

        Ok(Self(data))
    }

    #[must_use]
    pub fn from_expanded_secret_key(expanded_secret_key: &ExpandedSecretKey) -> Self {
        Self(HiddenServiceSecretKeyData::Ed25519V1Type0(
            expanded_secret_key.0.to_bytes(),
        ))
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.0 {
            HiddenServiceSecretKeyData::Ed25519V1Type0(data) => {
                [ED25519_V1_SECRET_TYPE_0_KEY, &data].concat()
            }
        }
    }
}

impl TryFrom<&[u8]> for HiddenServiceSecretKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(bytes)
    }
}

impl From<&ExpandedSecretKey> for HiddenServiceSecretKey {
    fn from(expanded_secret_key: &ExpandedSecretKey) -> Self {
        Self::from_expanded_secret_key(expanded_secret_key)
    }
}

/*
 * ============================================================================
 * Hostname
 * ============================================================================
 */
#[derive(Eq, PartialEq)]
pub struct Hostname(String);

impl Hostname {
    /// # Errors
    ///
    /// Returns error if malformed.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HOSTNAME_LENGTH {
            return Err(Error::ParseError(format!(
                "expected {} byte hostname, found {} bytes",
                ED25519_V1_SECRET_TYPE_0_LENGTH,
                bytes.len()
            )));
        }
        let (hostname, _) = bytes.split_at(HOSTNAME_LENGTH);
        let hostname = String::from_utf8_lossy(hostname);

        match hostname.split_once('.') {
            Some((_, tld)) => match tld {
                "onion" => Ok(Self(hostname.into())),
                _ => Err(Error::ParseError(format!("unsupported TLD: {tld}"))),
            },
            None => Err(Error::ParseError("missing TLD".to_string())),
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<PublicKey> for Hostname {
    fn from(public_key: PublicKey) -> Self {
        public_key.hostname()
    }
}

impl std::fmt::Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*
 * ============================================================================
 * Public Key
 * ============================================================================
 */
#[derive(Eq, PartialEq)]
pub struct PublicKey(ed25519_dalek::PublicKey);

impl PublicKey {
    #[must_use]
    pub fn hostname(&self) -> Hostname {
        let mut hasher = Sha3_256::new();
        hasher.update(b".onion checksum");
        hasher.update(self.0.as_ref());
        hasher.update([0x03]);
        let checksum: [u8; 2] = hasher.finalize()[..2]
            .try_into()
            .expect("slice of fixed size wasn't that size");

        let data = [self.0.as_ref(), &checksum, &[0x03]].concat();

        let address = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data)
            .to_ascii_lowercase();

        Hostname(format!("{address}.onion"))
    }

    /// # Errors
    ///
    /// Returns error if malformed.
    pub fn try_from_hidden_service_public_key(
        hidden_service_public_key: &HiddenServicePublicKey,
    ) -> Result<Self> {
        match hidden_service_public_key.0 {
            HiddenServicePublicKeyData::Ed25519V1Type0(data) => {
                ed25519_dalek::PublicKey::from_bytes(&data)
                    .map(Self)
                    .map_err(Error::SignatureError)
            }
        }
    }
}

impl TryFrom<&HiddenServicePublicKey> for PublicKey {
    type Error = Error;

    fn try_from(value: &HiddenServicePublicKey) -> Result<Self, Self::Error> {
        Self::try_from_hidden_service_public_key(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::{
        ExpandedSecretKey, HiddenServicePublicKey, HiddenServiceSecretKey, Hostname, PublicKey,
    };

    #[test]
    fn auto_generated() {
        /*
         * ====================================================================
         * Hidden Service Secret Key
         * ====================================================================
         */
        let data = std::fs::read(
            "./src/test/hidden_service_examples/auto_generated/hidden_service/hs_ed25519_secret_key",
        )
        .unwrap();

        let hidden_service_secret_key: HiddenServiceSecretKey = data.as_slice().try_into().unwrap();

        /*
         * ====================================================================
         * Expanded Secret Key
         * ====================================================================
         */
        let secret: ExpandedSecretKey = (&hidden_service_secret_key).try_into().unwrap();

        assert_eq!(
            vec![
                88, 236, 169, 104, 35, 16, 225, 104, 131, 154, 122, 30, 191, 39, 112, 17, 224, 172,
                15, 86, 126, 204, 212, 127, 113, 239, 122, 27, 190, 146, 5, 118, 177, 88, 175, 88,
                62, 23, 143, 214, 221, 112, 253, 240, 55, 105, 247, 18, 140, 111, 103, 97, 207,
                188, 174, 62, 122, 124, 51, 184, 166, 59, 218, 13
            ],
            secret.0.to_bytes(),
            "unable to create secret key from hs_ed25519_secret_key"
        );

        /*
         * ====================================================================
         * Hidden Service Secret Key (Round Trip)
         * ====================================================================
         */
        let hidden_service_secret_key_round_trip: HiddenServiceSecretKey = (&secret).into();

        assert_eq!(
            data,
            hidden_service_secret_key_round_trip.to_bytes(),
            "unable to create secret key from hs_ed25519_secret_key"
        );

        /*
         * ====================================================================
         * Hidden Service Public Key
         * ====================================================================
         */
        let data = std::fs::read(
            "./src/test/hidden_service_examples/auto_generated/hidden_service/hs_ed25519_public_key",
        )
        .unwrap();

        let hidden_service_public_key: HiddenServicePublicKey = data.as_slice().try_into().unwrap();

        /*
         * ====================================================================
         * Public Key
         * ====================================================================
         */
        let public: PublicKey = (&hidden_service_public_key).try_into().unwrap();

        assert_eq!(
            vec![
                243, 245, 51, 158, 27, 175, 158, 33, 137, 180, 184, 102, 68, 94, 90, 238, 168, 137,
                84, 120, 11, 125, 66, 179, 30, 37, 117, 186, 194, 111, 12, 255
            ],
            public.0.to_bytes(),
            "unable to create public key from hs_ed25519_public_key"
        );

        assert_eq!(
            public.0.to_bytes(),
            secret.public_key().0.to_bytes(),
            "keys are different"
        );

        /*
         * ====================================================================
         * Hidden Service Secret Key (Round Trip)
         * ====================================================================
         */
        let hidden_service_public_key_round_trip: HiddenServicePublicKey = (&public).into();

        assert_eq!(
            data,
            hidden_service_public_key_round_trip.to_bytes(),
            "unable to create secret key from hs_ed25519_secret_key"
        );

        /*
         * ====================================================================
         * Hostname
         * ====================================================================
         */
        let data = std::fs::read(
            "./src/test/hidden_service_examples/auto_generated/hidden_service/hostname",
        )
        .unwrap();

        let hostname = Hostname::try_from_bytes(&data).unwrap();

        assert_eq!(hostname.0, public.hostname().0, "host names are different");
    }

    #[test]
    fn generate() {
        let _ = ExpandedSecretKey::generate();
    }
}
