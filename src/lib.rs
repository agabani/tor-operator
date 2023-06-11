#![warn(clippy::pedantic)]

pub mod cli;
pub mod http_server;
pub mod onion_address;
pub mod onionbalance;
pub mod onionservice;

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
