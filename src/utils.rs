use std::collections::BTreeMap;

use k8s_openapi::api::{apps::v1::Deployment, core::v1::ConfigMap};
use kube::{
    api::{DeleteParams, ListParams, PatchParams},
    ResourceExt,
};

use crate::{
    Error, Labels, ObjectName, ObjectNamespace, Result, SelectorLabels, Uid,
    APP_KUBERNETES_IO_COMPONENT_KEY, APP_KUBERNETES_IO_INSTANCE_KEY,
    APP_KUBERNETES_IO_MANAGED_BY_KEY, APP_KUBERNETES_IO_MANAGED_BY_VALUE,
    APP_KUBERNETES_IO_NAME_KEY, APP_KUBERNETES_IO_NAME_VALUE, TOR_AGABANI_CO_UK_OWNED_BY_KEY,
};

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

pub trait KubeCrdResourceExt: KubeResourceExt {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str;

    fn delete_params(&self) -> DeleteParams {
        DeleteParams::default()
    }

    fn patch_params(&self) -> PatchParams {
        PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force()
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
