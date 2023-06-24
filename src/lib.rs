#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, ops::Deref};

use sha2::{Digest, Sha256};

pub mod cli;
pub mod crypto;
pub mod http_server;
pub mod onion_balance;
pub mod onion_key;
pub mod onion_service;
pub mod tor_ingress;
mod utils;

/*
 * ============================================================================
 * Error
 * ============================================================================
 */
#[derive(Debug)]
pub enum Error {
    Kube(kube::Error),
    MissingObjectKey(&'static str),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/*
 * ============================================================================
 * Result
 * ============================================================================
 */
pub type Result<T, E = Error> = std::result::Result<T, E>;

/*
 * ============================================================================
 * Kubernetes Constants
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
 * Kubernetes Types
 * ============================================================================
 */
pub struct Annotations(BTreeMap<String, String>);

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

struct OBConfig(String);

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
