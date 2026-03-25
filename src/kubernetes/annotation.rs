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
    fn sha_256_returns_sha256_prefix() {
        // arrange
        let annotation = TestAnnotation("hello".to_string());

        // act
        let result = annotation.sha_256();

        // assert
        assert!(result.starts_with("sha256:"));
    }

    #[test]
    fn sha_256_produces_correct_hash() {
        // arrange
        // echo -n "hello" | sha256sum => 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
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
    fn sha_256_empty_string() {
        // arrange
        // echo -n "" | sha256sum => e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let annotation = TestAnnotation(String::new());

        // act
        let result = annotation.sha_256();

        // assert
        assert_eq!(
            result,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha_256_is_deterministic() {
        // arrange
        let annotation = TestAnnotation("deterministic".to_string());

        // act
        let result1 = annotation.sha_256();
        let result2 = annotation.sha_256();

        // assert
        assert_eq!(result1, result2);
    }

    #[test]
    fn sha_256_different_inputs_produce_different_hashes() {
        // arrange
        let a = TestAnnotation("foo".to_string());
        let b = TestAnnotation("bar".to_string());

        // act / assert
        assert_ne!(a.sha_256(), b.sha_256());
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
