use crate::crypto::Hostname;

use super::annotation::Annotation;

impl Annotation for Hostname {
    const NAME: &'static str = "hostname";
}
