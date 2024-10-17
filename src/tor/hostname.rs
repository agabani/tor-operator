use sha3::{Digest, Sha3_256};

use crate::kubernetes::Annotation;

use super::{constants::ONION_DOMAIN_LENGTH, Error, PublicKey};

#[derive(Debug, PartialEq)]
pub struct Hostname(String);

impl Hostname {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl Annotation<'_> for Hostname {
    const NAME: &'static str = "hostname";
}

impl std::fmt::Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&PublicKey> for Hostname {
    fn from(value: &PublicKey) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(b".onion checksum");
        hasher.update(value.as_ref());
        hasher.update([0x03]);
        let checksum: [u8; 2] = hasher.finalize()[..2]
            .try_into()
            .expect("slice of fixed size wasn't that size");

        let data = [value.as_ref(), &checksum, &[0x03]].concat();

        let address = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &data)
            .to_ascii_lowercase();

        Self::new(format!("{address}.onion"))
    }
}

impl TryFrom<&[u8]> for Hostname {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let hostname = String::from_utf8_lossy(value);
        let hostname = hostname.trim();

        match hostname.split_once('.') {
            Some((domain, tld)) => match tld {
                "onion" => {
                    if domain.len() == ONION_DOMAIN_LENGTH {
                        Ok(Self(hostname.into()))
                    } else {
                        Err(Error::ParseError(format!(
                            "expected {} byte domain, found {} bytes",
                            ONION_DOMAIN_LENGTH,
                            domain.len()
                        )))
                    }
                }
                _ => Err(Error::ParseError(format!("unsupported TLD: {tld}"))),
            },
            None => Err(Error::ParseError("missing TLD".to_string())),
        }
    }
}

impl TryFrom<&Vec<u8>> for Hostname {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        value.as_slice().try_into()
    }
}

/*
 * ============================================================================
 *
 * ============================================================================
 */
impl<'a> From<&'a Hostname> for std::borrow::Cow<'a, str> {
    fn from(value: &'a Hostname) -> Self {
        std::borrow::Cow::Borrowed(&value.0)
    }
}

impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.0
    }
}

impl From<&Hostname> for Vec<u8> {
    fn from(value: &Hostname) -> Self {
        value.0.clone().into_bytes()
    }
}
