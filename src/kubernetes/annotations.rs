use std::{collections::BTreeMap, ops::Deref};

use super::Annotation;

#[derive(Default)]
pub struct Annotations(BTreeMap<String, String>);

impl Annotations {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add<A: Annotation>(mut self, annotation: &A) -> Self {
        let (key, value) = annotation.to_tuple();
        self.0.insert(key, value);
        self
    }

    pub fn add_opt<A: Annotation>(mut self, annotation: &Option<A>) -> Self {
        if let Some((key, value)) = annotation.as_ref().map(Annotation::to_tuple) {
            self.0.insert(key, value);
        }
        self
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
