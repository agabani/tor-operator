use std::collections::BTreeMap;

pub struct SelectorLabels(BTreeMap<String, String>);

impl From<SelectorLabels> for BTreeMap<String, String> {
    fn from(value: SelectorLabels) -> Self {
        value.0
    }
}

impl From<BTreeMap<String, String>> for SelectorLabels {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(value)
    }
}

impl From<&SelectorLabels> for BTreeMap<String, String> {
    fn from(value: &SelectorLabels) -> Self {
        value.0.clone()
    }
}
