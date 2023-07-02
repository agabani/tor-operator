use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceName(String);

impl ResourceName {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for ResourceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for ResourceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ResourceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<&String> for ResourceName {
    fn from(value: &String) -> Self {
        ResourceName(value.clone())
    }
}
