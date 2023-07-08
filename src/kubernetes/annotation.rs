use std::borrow::Cow;

use sha2::{Digest, Sha256};

use super::constants::TOR_AGABANI_CO_UK;

pub trait Annotation<'a>
where
    &'a Self: 'a + Into<Cow<'a, str>>,
{
    const NAME: &'static str;

    #[must_use]
    fn sha_256(&'a self) -> String {
        let mut sha = Sha256::new();
        sha.update(&*self.into());
        format!("sha256:{:x}", sha.finalize())
    }

    #[must_use]
    fn to_tuple(&'a self) -> (String, String) {
        (
            format!("{TOR_AGABANI_CO_UK}/{}-hash", Self::NAME),
            self.sha_256(),
        )
    }
}
