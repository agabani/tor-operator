use crate::metrics::Metrics;

pub trait Context {
    fn metrics(&self) -> &Metrics;
}
