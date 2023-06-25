use std::{collections::BTreeMap, ops::Deref};

use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{ConfigMap, Secret},
};
use kube::{
    api::{DeleteParams, ListParams, Patch, PatchParams},
    ResourceExt,
};
use sha2::{Digest, Sha256};

use crate::{Error, Result};

pub(crate) fn btree_maps_are_equal<K: Ord + Eq, V: Eq>(
    map1: &BTreeMap<K, V>,
    map2: &BTreeMap<K, V>,
) -> bool {
    if map1.len() != map2.len() {
        return false;
    }

    for (key, value) in map1 {
        if map2.get(key) != Some(value) {
            return false;
        }
    }

    true
}

/*
 * ============================================================================
 * Constants
 * ============================================================================
 */
const APP_KUBERNETES_IO_COMPONENT_KEY: &str = "app.kubernetes.io/component";

const APP_KUBERNETES_IO_INSTANCE_KEY: &str = "app.kubernetes.io/instance";

const APP_KUBERNETES_IO_MANAGED_BY_KEY: &str = "app.kubernetes.io/managed-by";
const APP_KUBERNETES_IO_MANAGED_BY_VALUE: &str = "tor-operator";

const APP_KUBERNETES_IO_NAME_KEY: &str = "app.kubernetes.io/name";
const APP_KUBERNETES_IO_NAME_VALUE: &str = "tor";

const TOR_AGABANI_CO_UK_CONFIG_HASH_KEY: &str = "tor.agabani.co.uk/config-hash";

const TOR_AGABANI_CO_UK_OWNED_BY_KEY: &str = "tor.agabani.co.uk/owned-by";

const TOR_AGABANI_CO_UK_TORRC_HASH_KEY: &str = "tor.agabani.co.uk/torrc-hash";

/*
 * ============================================================================
 * Types
 * ============================================================================
 */
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

pub struct ConfigYaml(String);

impl ConfigYaml {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl ConfigYaml {
    #[must_use]
    pub fn to_annotation_tuple(&self) -> (String, String) {
        (TOR_AGABANI_CO_UK_CONFIG_HASH_KEY.into(), self.sha_256())
    }

    #[must_use]
    pub fn sha_256(&self) -> String {
        let mut sha = Sha256::new();
        sha.update(&self.0);
        format!("sha256:{:x}", sha.finalize())
    }
}

impl From<ConfigYaml> for String {
    fn from(value: ConfigYaml) -> Self {
        value.0
    }
}

impl From<&ConfigYaml> for String {
    fn from(value: &ConfigYaml) -> Self {
        value.0.clone()
    }
}

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

impl From<&Labels> for BTreeMap<String, String> {
    fn from(value: &Labels) -> Self {
        value.0.clone()
    }
}

pub struct OBConfig(String);

impl OBConfig {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl From<OBConfig> for String {
    fn from(value: OBConfig) -> Self {
        value.0
    }
}

impl From<&OBConfig> for String {
    fn from(value: &OBConfig) -> Self {
        value.0.clone()
    }
}

pub struct ObjectName<'a>(&'a str);

impl ObjectName<'_> {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl Deref for ObjectName<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> From<ObjectName<'a>> for &'a str {
    fn from(value: ObjectName<'a>) -> Self {
        value.0
    }
}

pub struct ObjectNamespace<'a>(&'a str);

impl Deref for ObjectNamespace<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct SelectorLabels(BTreeMap<String, String>);

impl From<SelectorLabels> for BTreeMap<String, String> {
    fn from(value: SelectorLabels) -> Self {
        value.0
    }
}

impl From<&SelectorLabels> for BTreeMap<String, String> {
    fn from(value: &SelectorLabels) -> Self {
        value.0.clone()
    }
}

pub struct Torrc(String);

impl Torrc {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl Torrc {
    #[must_use]
    pub fn to_annotation_tuple(&self) -> (String, String) {
        (TOR_AGABANI_CO_UK_TORRC_HASH_KEY.into(), self.sha_256())
    }

    #[must_use]
    pub fn sha_256(&self) -> String {
        let mut sha = Sha256::new();
        sha.update(&self.0);
        format!("sha256:{:x}", sha.finalize())
    }
}

impl From<Torrc> for String {
    fn from(value: Torrc) -> Self {
        value.0
    }
}

impl From<&Torrc> for String {
    fn from(value: &Torrc) -> Self {
        value.0.clone()
    }
}

pub struct Uid<'a>(&'a str);

impl Deref for Uid<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::fmt::Display for Uid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*
 * ============================================================================
 * Traits
 * ============================================================================
 */
pub trait KubeResourceExt: ResourceExt {
    fn try_name(&self) -> Result<ObjectName> {
        self.meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))
            .map(String::as_str)
            .map(ObjectName)
    }

    fn try_namespace(&self) -> Result<ObjectNamespace> {
        self.meta()
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))
            .map(String::as_str)
            .map(ObjectNamespace)
    }

    fn try_uid(&self) -> Result<Uid> {
        self.meta()
            .uid
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.uid"))
            .map(String::as_str)
            .map(Uid)
    }
}

impl KubeResourceExt for ConfigMap {}

impl KubeResourceExt for Deployment {}

impl KubeResourceExt for Secret {}

pub trait KubeCrdResourceExt: KubeResourceExt {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str;

    fn delete_params(&self) -> DeleteParams {
        DeleteParams::default()
    }

    fn patch_params(&self) -> PatchParams {
        PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force()
    }

    fn patch_status<P: serde::Serialize>(&self, status: P) -> Patch<serde_json::Value> {
        Patch::Merge(serde_json::json!({ "status": status }))
    }

    fn patch_status_params(&self) -> PatchParams {
        PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE)
    }

    fn try_owned_list_params(&self) -> Result<ListParams> {
        Ok(ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            self.try_uid()?
        )))
    }

    fn try_labels(&self) -> Result<Labels> {
        Ok(Labels(BTreeMap::from([
            (
                APP_KUBERNETES_IO_COMPONENT_KEY.into(),
                Self::APP_KUBERNETES_IO_COMPONENT_VALUE.into(),
            ),
            (
                APP_KUBERNETES_IO_INSTANCE_KEY.into(),
                self.try_name()?.to_string(),
            ),
            (
                APP_KUBERNETES_IO_MANAGED_BY_KEY.into(),
                APP_KUBERNETES_IO_MANAGED_BY_VALUE.into(),
            ),
            (
                APP_KUBERNETES_IO_NAME_KEY.into(),
                APP_KUBERNETES_IO_NAME_VALUE.into(),
            ),
            (
                TOR_AGABANI_CO_UK_OWNED_BY_KEY.into(),
                self.try_uid()?.to_string(),
            ),
        ])))
    }

    fn try_selector_labels(&self) -> Result<SelectorLabels> {
        Ok(SelectorLabels(BTreeMap::from([
            (
                APP_KUBERNETES_IO_COMPONENT_KEY.into(),
                Self::APP_KUBERNETES_IO_COMPONENT_VALUE.into(),
            ),
            (
                APP_KUBERNETES_IO_INSTANCE_KEY.into(),
                self.try_name()?.to_string(),
            ),
            (
                APP_KUBERNETES_IO_NAME_KEY.into(),
                APP_KUBERNETES_IO_NAME_VALUE.into(),
            ),
        ])))
    }
}
