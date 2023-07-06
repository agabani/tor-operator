use sha2::{Digest, Sha256};

use super::constants::TOR_AGABANI_CO_UK;

pub trait Annotation
where
    Self: AsRef<str>,
{
    const NAME: &'static str;

    #[must_use]
    fn sha_256(&self) -> String {
        let mut sha = Sha256::new();
        sha.update(self.as_ref());
        format!("sha256:{:x}", sha.finalize())
    }

    #[must_use]
    fn to_tuple(&self) -> (String, String) {
        (
            format!("{TOR_AGABANI_CO_UK}/{}-hash", Self::NAME),
            self.sha_256(),
        )
    }
}
