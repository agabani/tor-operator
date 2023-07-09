use std::{collections::BTreeMap, ops::Deref};

#[derive(Debug, Default, Clone)]
pub struct Labels(BTreeMap<String, String>);

impl Labels {
    pub fn append_reverse(mut self, other: Option<Self>) -> Self {
        if let Some(other) = other {
            let mut other = other.0;
            other.append(&mut self.0);
            Self(other)
        } else {
            self
        }
    }
}

impl Deref for Labels {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, String>> for Labels {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(value)
    }
}

impl From<Labels> for BTreeMap<String, String> {
    fn from(value: Labels) -> Self {
        value.0
    }
}

impl From<&Labels> for BTreeMap<String, String> {
    fn from(value: &Labels) -> Self {
        value.0.clone()
    }
}
