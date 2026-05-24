use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceNamespace(String);

impl ResourceNamespace {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ResourceNamespace {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ResourceNamespace {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ResourceNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<&String> for ResourceNamespace {
    fn from(value: &String) -> Self {
        ResourceNamespace(value.clone())
    }
}

impl From<String> for ResourceNamespace {
    fn from(value: String) -> Self {
        ResourceNamespace(value)
    }
}

impl From<ResourceNamespace> for String {
    fn from(value: ResourceNamespace) -> Self {
        value.0
    }
}
