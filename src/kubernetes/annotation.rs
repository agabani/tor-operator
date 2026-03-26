use std::{borrow::Cow, fmt::Write as _};

use sha2::{Digest, Sha256};

use super::constants::TOR_AGABANI_CO_UK;

pub trait Annotation<'a>
where
    &'a Self: 'a + Into<Cow<'a, str>>,
{
    const NAME: &'static str;

    #[must_use]
    fn sha_256(&'a self) -> String {
        let digest = Sha256::digest(&*self.into());
        let mut sha256 = String::with_capacity(7 + 64);
        sha256.push_str("sha256:");
        for byte in &digest {
            write!(sha256, "{byte:02x}").expect("writing to a String is infallible");
        }
        sha256
    }

    #[must_use]
    fn to_tuple(&'a self) -> (String, String) {
        (
            format!("{TOR_AGABANI_CO_UK}/{}-hash", Self::NAME),
            self.sha_256(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::Annotation;

    struct TestAnnotation(String);

    impl Annotation<'_> for TestAnnotation {
        const NAME: &'static str = "test";
    }

    impl<'a> From<&'a TestAnnotation> for Cow<'a, str> {
        fn from(value: &'a TestAnnotation) -> Self {
            Cow::Borrowed(&value.0)
        }
    }

    #[test]
    fn sha_256_produces_correct_hash() {
        // arrange
        let annotation = TestAnnotation("hello".to_string());

        // act
        let result = annotation.sha_256();

        // assert
        assert_eq!(
            result,
            "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha_256_does_not_reallocate() {
        // arrange
        let annotation = TestAnnotation("hello".to_string());

        // act
        let result = annotation.sha_256();

        // assert
        assert_eq!(result.len(), result.capacity());
    }

    #[test]
    fn to_tuple_key_uses_name_and_domain() {
        // arrange
        let annotation = TestAnnotation("value".to_string());

        // act
        let (key, _) = annotation.to_tuple();

        // assert
        assert_eq!(key, "tor.agabani.co.uk/test-hash");
    }

    #[test]
    fn to_tuple_value_matches_sha_256() {
        // arrange
        let annotation = TestAnnotation("value".to_string());

        // act
        let (_, value) = annotation.to_tuple();

        // assert
        assert_eq!(value, annotation.sha_256());
    }
}
