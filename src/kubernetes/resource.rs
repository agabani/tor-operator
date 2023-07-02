use std::fmt::Debug;

use crate::{Error, Result};

use super::{Object, ObjectNamespace, ResourceName, ResourceUid};

pub trait Resource: kube::ResourceExt<DynamicType = ()> {
    type Spec: PartialEq + Debug;

    fn spec(&self) -> &Self::Spec;

    fn try_name(&self) -> Result<ResourceName> {
        self.meta()
            .name
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.name"))
            .map(String::to_string)
            .map(ResourceName::new)
    }

    fn try_namespace(&self) -> Result<ObjectNamespace> {
        self.meta()
            .namespace
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.namespace"))
            .map(String::to_string)
            .map(Into::into)
    }

    fn try_uid(&self) -> Result<ResourceUid> {
        self.meta()
            .uid
            .as_ref()
            .ok_or_else(|| Error::MissingObjectKey(".metadata.uid"))
            .map(String::to_string)
            .map(ResourceUid::new)
    }

    fn try_with_owner(mut self, object: &(impl Object + Resource)) -> Result<Self>
    where
        Self: Sized,
    {
        let (key, value, reference) = object.try_owner_reference()?;
        *self.owner_references_mut() = vec![reference];
        self.labels_mut().insert(key, value);
        Ok(self)
    }
}

impl Resource for k8s_openapi::api::core::v1::ConfigMap {
    type Spec = std::collections::BTreeMap<String, String>;

    fn spec(&self) -> &Self::Spec {
        self.data.as_ref().unwrap()
    }
}

impl Resource for k8s_openapi::api::apps::v1::Deployment {
    type Spec = k8s_openapi::api::apps::v1::DeploymentSpec;

    fn spec(&self) -> &Self::Spec {
        self.spec.as_ref().unwrap()
    }
}

impl Resource for k8s_openapi::api::core::v1::Secret {
    type Spec = std::collections::BTreeMap<String, k8s_openapi::ByteString>;

    fn spec(&self) -> &Self::Spec {
        self.data.as_ref().unwrap()
    }
}
