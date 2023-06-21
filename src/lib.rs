#![warn(clippy::pedantic)]

use std::collections::BTreeMap;

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

const TOR_AGABANI_CO_UK_OWNED_BY_KEY: &str = "tor.agabani.co.uk/owned-by";

/*
 * ============================================================================
 * Kubernetes Types
 * ============================================================================
 */
struct Annotations(BTreeMap<String, String>);
struct ConfigYaml(String);
struct Labels(BTreeMap<String, String>);
struct OBConfig(String);
struct ObjectName<'a>(&'a str);
struct ObjectNamespace<'a>(&'a str);
struct SelectorLabels(BTreeMap<String, String>);
struct Torrc(String);
