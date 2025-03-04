use std::{borrow::Cow, collections::BTreeMap, ops::Deref};

use super::Annotation;

#[derive(Default, Clone)]
pub struct Annotations(BTreeMap<String, String>);

impl Annotations {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add<'a, A>(mut self, annotation: &'a A) -> Self
    where
        A: Annotation<'a>,
        &'a A: 'a + Into<Cow<'a, str>>,
    {
        let (key, value) = annotation.to_tuple();
        self.0.insert(key, value);
        self
    }

    pub fn add_opt<'a, A>(mut self, annotation: Option<&'a A>) -> Self
    where
        A: Annotation<'a>,
        &'a A: 'a + Into<Cow<'a, str>>,
    {
        if let Some((key, value)) = annotation.map(Annotation::to_tuple) {
            self.0.insert(key, value);
        }
        self
    }

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

impl Deref for Annotations {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, String>> for Annotations {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(value)
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
