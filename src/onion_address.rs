use base32::encode;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sha3::{Digest, Sha3_256};

/// Generates Onion Address
///
/// ---
///
/// Encoding onion addresses [ONIONADDRESS]
///
/// The onion address of a hidden service includes its identity public key, a
/// version field and a basic checksum. All this information is then base32
/// encoded as shown below:
///
/// ```
/// onion_address = base32(PUBKEY | CHECKSUM | VERSION) + ".onion"
/// CHECKSUM = H(".onion checksum" | PUBKEY | VERSION)[:2]
/// ```
///
/// where:
/// - PUBKEY is the 32 bytes ed25519 master pubkey of the hidden service.
/// - VERSION is a one byte version field (default value '\x03')
/// - ".onion checksum" is a constant string
/// - CHECKSUM is truncated to two bytes before inserting it in `onion_address`
#[must_use]
pub fn generate() -> OnionAddress {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);

    let public = signing_key.verifying_key().to_bytes();
    let secret = signing_key.to_bytes();

    let mut sha = Sha3_256::new();
    sha.update(".onion checksum");
    sha.update(public);
    sha.update([3]);
    let checksum = sha.finalize();

    let hostname = format!(
        "{:}.onion",
        encode(
            base32::Alphabet::RFC4648 { padding: false },
            &[&public[..], &checksum[..2], &[3]].concat(),
        )
        .to_lowercase()
    );

    let public = &[b"== ed25519v1-public: type0 ==\0\0\0", &public[..]].concat();

    let secret = &[b"== ed25519v1-secret: type0 ==\0\0\0", &secret[..]].concat();

    OnionAddress {
        hostname,
        public: public.clone(),
        secret: secret.clone(),
    }
}

pub struct OnionAddress {
    pub hostname: String,

    pub public: Vec<u8>,

    pub secret: Vec<u8>,
}
