use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceUid(String);

impl ResourceUid {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ResourceUid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ResourceUid {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ResourceUid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<&String> for ResourceUid {
    fn from(value: &String) -> Self {
        ResourceUid(value.clone())
    }
}

impl From<String> for ResourceUid {
    fn from(value: String) -> Self {
        ResourceUid(value)
    }
}

impl From<ResourceUid> for String {
    fn from(value: ResourceUid) -> Self {
        value.0
    }
}
