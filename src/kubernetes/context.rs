use crate::metrics::Metrics;

use super::ErrorBackoff;

pub trait Context {
    fn error_backoff(&self) -> &ErrorBackoff;
    fn metrics(&self) -> &Metrics;
}
