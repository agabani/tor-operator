use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{DeleteParams, ListParams, Patch, PatchParams};

use super::{
    Labels, SelectorLabels,
    constants::{
        APP_KUBERNETES_IO_COMPONENT_KEY, APP_KUBERNETES_IO_INSTANCE_KEY,
        APP_KUBERNETES_IO_MANAGED_BY_KEY, APP_KUBERNETES_IO_MANAGED_BY_VALUE,
        APP_KUBERNETES_IO_NAME_KEY, APP_KUBERNETES_IO_NAME_VALUE, TOR_AGABANI_CO_UK_OWNED_BY_KEY,
        TOR_AGABANI_CO_UK_PART_OF_KEY,
    },
    resource::Resource,
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

    fn status(&self) -> Option<&Self::Status>;

    fn try_owner_reference(&self) -> Result<(String, String, OwnerReference)>
    where
        Self: Resource,
    {
        self.try_uid().map(|uid| {
            (
                TOR_AGABANI_CO_UK_OWNED_BY_KEY.into(),
                uid.into(),
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
                self.try_name()?.into(),
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
                self.try_uid()?.into(),
            ),
            (
                TOR_AGABANI_CO_UK_PART_OF_KEY.into(),
                if let Some(part_of) = self.labels().get(TOR_AGABANI_CO_UK_PART_OF_KEY) {
                    part_of.to_string()
                } else {
                    self.try_uid()?.into()
                },
            ),
        ])
        .into())
    }

    fn try_label_selector<O: Object>(&self) -> Result<String>
    where
        Self: Resource,
    {
        Ok(format!(
            "{APP_KUBERNETES_IO_COMPONENT_KEY}={},{TOR_AGABANI_CO_UK_PART_OF_KEY}={}",
            O::APP_KUBERNETES_IO_COMPONENT_VALUE,
            self.try_uid()?
        ))
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
                self.try_name()?.into(),
            ),
            (
                APP_KUBERNETES_IO_NAME_KEY.into(),
                APP_KUBERNETES_IO_NAME_VALUE.into(),
            ),
        ])
        .into())
    }
}
