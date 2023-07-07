use k8s_openapi::ByteString;

pub trait Subset {
    fn is_subset(&self, superset: &Self) -> bool;
}

impl Subset for std::collections::BTreeMap<String, ByteString> {
    fn is_subset(&self, superset: &Self) -> bool {
        self.iter()
            .all(|(key, value)| Some(value) == superset.get(key))
    }
}

impl Subset for std::collections::BTreeMap<String, String> {
    fn is_subset(&self, superset: &Self) -> bool {
        self.iter()
            .all(|(key, value)| Some(value) == superset.get(key))
    }
}

impl Subset for Option<std::collections::BTreeMap<String, String>> {
    fn is_subset(&self, superset: &Self) -> bool {
        match (self, superset) {
            (None, None) => true,
            (None, Some(data)) | (Some(data), None) => data.is_empty(),
            (Some(subset), Some(superset)) => subset.is_subset(superset),
        }
    }
}

impl Subset for kube::core::ObjectMeta {
    fn is_subset(&self, superset: &Self) -> bool {
        self.annotations.is_subset(&superset.annotations)
            && self.labels.is_subset(&superset.labels)
            && self.name == superset.name
            && self.owner_references == superset.owner_references
    }
}

impl Subset for k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscalerSpec {
    fn is_subset(&self, superset: &Self) -> bool {
        self == superset
    }
}

impl Subset for k8s_openapi::api::apps::v1::DeploymentSpec {
    fn is_subset(&self, superset: &Self) -> bool {
        if self.replicas != superset.replicas {
            return false;
        }

        if self.selector != superset.selector {
            return false;
        }

        if self.template.metadata != superset.template.metadata {
            return false;
        }

        if self.template.spec != superset.template.spec {
            /* This can be further optimized by comparing fields
             *
             * The deployments operator auto populates:
             *  - `resources`
             *  - `termination_message_path`
             *  - `termination_message_policy`
             */
            return false;
        }

        true
    }
}
