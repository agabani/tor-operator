use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{DeleteParams, ListParams, Patch, PatchParams};

use super::{
    constants::{
        APP_KUBERNETES_IO_COMPONENT_KEY, APP_KUBERNETES_IO_INSTANCE_KEY,
        APP_KUBERNETES_IO_MANAGED_BY_KEY, APP_KUBERNETES_IO_MANAGED_BY_VALUE,
        APP_KUBERNETES_IO_NAME_KEY, APP_KUBERNETES_IO_NAME_VALUE, TOR_AGABANI_CO_UK_OWNED_BY_KEY,
    },
    resource::Resource,
    Labels, SelectorLabels,
};

use crate::Result;

pub trait Object: kube::ResourceExt<DynamicType = ()> {
    const APP_KUBERNETES_IO_COMPONENT_VALUE: &'static str;

    type Status: PartialEq + serde::Serialize;

    fn delete_params(&self) -> DeleteParams {
        DeleteParams::default()
    }

    fn patch_params(&self) -> PatchParams {
        PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE).force()
    }

    fn patch_status(&self, status: Self::Status) -> Patch<serde_json::Value> {
        Patch::Merge(serde_json::json!({ "status": status }))
    }

    fn patch_status_params(&self) -> PatchParams {
        PatchParams::apply(APP_KUBERNETES_IO_MANAGED_BY_VALUE)
    }

    fn status(&self) -> &Option<Self::Status>;

    fn try_owner_reference(&self) -> Result<(String, String, OwnerReference)>
    where
        Self: Resource,
    {
        self.try_uid().map(|uid| {
            (
                TOR_AGABANI_CO_UK_OWNED_BY_KEY.into(),
                uid.to_string(),
                self.controller_owner_ref(&()).unwrap(),
            )
        })
    }

    fn try_owned_list_params(&self) -> Result<ListParams>
    where
        Self: Resource,
    {
        Ok(ListParams::default().labels(&format!(
            "{TOR_AGABANI_CO_UK_OWNED_BY_KEY}={}",
            self.try_uid()?
        )))
    }

    fn try_labels(&self) -> Result<Labels>
    where
        Self: Resource,
    {
        Ok(BTreeMap::from([
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
        ])
        .into())
    }

    fn try_selector_labels(&self) -> Result<SelectorLabels>
    where
        Self: Resource,
    {
        Ok(BTreeMap::from([
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
        ])
        .into())
    }
}
