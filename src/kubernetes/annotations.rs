use std::{collections::BTreeMap, ops::Deref};

pub struct Annotations(BTreeMap<String, String>);

impl Annotations {
    pub fn new(value: BTreeMap<String, String>) -> Self {
        Self(value)
    }
}

impl Deref for Annotations {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Annotations> for BTreeMap<String, String> {
    fn from(value: Annotations) -> Self {
        value.0
    }
}

impl From<&Annotations> for BTreeMap<String, String> {
    fn from(value: &Annotations) -> Self {
        value.0.clone()
    }
}
