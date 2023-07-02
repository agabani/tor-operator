use std::{
    fmt::{Display, Formatter, Result},
    ops::Deref,
};

pub struct ResourceUid(String);

impl ResourceUid {
    pub fn new(value: String) -> Self {
        Self(value)
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
