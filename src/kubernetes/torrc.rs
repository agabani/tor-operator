use super::annotation::Annotation;

pub struct Torrc(String);

impl Torrc {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for Torrc {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&Torrc> for String {
    fn from(value: &Torrc) -> Self {
        value.0.clone()
    }
}

impl Annotation for Torrc {
    const NAME: &'static str = "torrc";
}
