use std::ops::Deref;

pub struct ObjectNamespace(String);

impl Deref for ObjectNamespace {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ObjectNamespace {
    fn from(value: String) -> Self {
        Self(value)
    }
}
