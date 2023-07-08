mod config_yaml;
mod constants;
mod expanded_secret_key;
mod hidden_service_public_key;
mod hidden_service_secret_key;
mod hostname;
mod ob_config;
mod public_key;
mod torrc;

pub use config_yaml::{ConfigYaml, ConfigYamlService, ConfigYamlServiceInstance};
pub use expanded_secret_key::ExpandedSecretKey;
pub use hidden_service_public_key::HiddenServicePublicKey;
pub use hidden_service_secret_key::HiddenServiceSecretKey;
pub use hostname::Hostname;
pub use ob_config::{OBConfig, OBConfigBuilder};
pub use public_key::PublicKey;
pub use torrc::{Torrc, TorrcBuilder};

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    SignatureError(ed25519_dalek::SignatureError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "parse error: {e}"),
            Error::SignatureError(e) => write!(f, "signature error: {e}"),
        }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use crate::tor::{HiddenServicePublicKey, Hostname};

    use super::{ExpandedSecretKey, HiddenServiceSecretKey, PublicKey};

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
            secret.to_bytes(),
            "unable to create secret key from hs_ed25519_secret_key"
        );

        /*
         * ====================================================================
         * Hidden Service Secret Key (Round Trip)
         * ====================================================================
         */
        let hidden_service_secret_key_round_trip = HiddenServiceSecretKey::from(&secret);

        assert_eq!(
            data,
            Vec::<u8>::from(&hidden_service_secret_key_round_trip),
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

        let hidden_service_public_key = HiddenServicePublicKey::try_from(&data).unwrap();

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
            public.to_bytes(),
            "unable to create public key from hs_ed25519_public_key"
        );

        assert_eq!(
            public.to_bytes(),
            PublicKey::from(&secret).to_bytes(),
            "keys are different"
        );

        /*
         * ====================================================================
         * Hidden Service Secret Key (Round Trip)
         * ====================================================================
         */
        let hidden_service_public_key_round_trip = HiddenServicePublicKey::from(&public);

        assert_eq!(
            data,
            Vec::<u8>::from(&hidden_service_public_key_round_trip),
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

        let hostname: Hostname = data.as_slice().try_into().unwrap();

        assert_eq!(
            hostname,
            Hostname::from(&public),
            "host names are different"
        );
    }

    #[test]
    fn generate() {
        let _ = ExpandedSecretKey::generate();
    }
}
