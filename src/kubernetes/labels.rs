use std::{collections::BTreeMap, ops::Deref};

pub struct Labels(BTreeMap<String, String>);

impl Deref for Labels {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Labels> for BTreeMap<String, String> {
    fn from(value: Labels) -> Self {
        value.0
    }
}

impl From<BTreeMap<String, String>> for Labels {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(value)
    }
}

impl From<&Labels> for BTreeMap<String, String> {
    fn from(value: &Labels) -> Self {
        value.0.clone()
    }
}
