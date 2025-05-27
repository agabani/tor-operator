#![warn(clippy::pedantic)]

pub mod cli;
mod collections;
pub mod http_server;
mod kubernetes;
pub mod metrics;
pub mod onion_balance;
pub mod onion_key;
pub mod onion_service;
pub mod otel;
pub mod tor;
pub mod tor_ingress;
pub mod tor_proxy;

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
