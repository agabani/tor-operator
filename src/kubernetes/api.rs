use std::collections::HashMap;

use kube::core::ObjectList;

use crate::{metrics::Metrics, Error, Result};

use super::{subset::Subset, Object, Resource, ResourceName};

pub struct Api<K>(kube::Api<K>);

impl<R> Api<R> {
    pub fn new(api: kube::Api<R>) -> Self {
        Self(api)
    }
}

impl<R> Api<R>
where
    R: Resource + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize,
{
    #[tracing::instrument(skip_all)]
    pub async fn delete_many<O>(&self, object: &O, resources: Vec<R>) -> Result<()>
    where
        O: Object,
    {
        futures::future::join_all(
            resources
                .into_iter()
                .map(|resource| self.delete(object, resource)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map(|_| ())
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_status<O>(&self, object: &O, status: O::Status) -> Result<()>
    where
        O: Object + Resource,
    {
        match object.status() {
            Some(api_status) if &status == api_status => {}
            _ => {
                self.patch_status(object, status).await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        skip_all,
        fields(
            resource.r#ref = %format!("{}.{}.{}/{}", R::kind(&()), R::version(&()), R::group(&()), resource.name_any())
        )
    )]
    async fn delete<O>(&self, object: &O, resource: R) -> Result<()>
    where
        O: Object,
    {
        Metrics::kubernetes_api_usage_count::<R>("delete");
        self.0
            .delete(&resource.try_name()?, &object.delete_params())
            .await
            .map(|_| ())
            .map_err(Error::Kube)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            resource.r#ref = %format!("{}.{}.{}/{}", R::kind(&()), R::version(&()), R::group(&()), name)
        )
    )]
    pub async fn get_opt(&self, name: &ResourceName) -> Result<Option<R>> {
        Metrics::kubernetes_api_usage_count::<R>("get");
        self.0.get_opt(name).await.map_err(Error::Kube)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            resource.r#ref = %format!("{}.{}.{}/{}", R::kind(&()), R::version(&()), R::group(&()), object.name_any())
        )
    )]
    async fn list<O>(&self, object: &O) -> Result<ObjectList<R>>
    where
        O: Object + Resource,
    {
        Metrics::kubernetes_api_usage_count::<R>("list");
        self.0
            .list(&object.try_owned_list_params()?)
            .await
            .map_err(Error::Kube)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            resource.r#ref = %format!("{}.{}.{}/{}", R::kind(&()), R::version(&()), R::group(&()), resource.name_any())
        )
    )]
    async fn patch<O>(&self, object: &O, resource: &R) -> Result<()>
    where
        O: Object + Resource,
    {
        Metrics::kubernetes_api_usage_count::<R>("patch");
        self.0
            .patch(
                &resource.try_name()?,
                &object.patch_params(),
                &kube::api::Patch::Apply(&resource),
            )
            .await
            .map(|_| ())
            .map_err(Error::Kube)
    }

    #[tracing::instrument(
        skip_all,
        fields(
            resource.r#ref = %format!("{}.{}.{}/{}", R::kind(&()), R::version(&()), R::group(&()), object.name_any())
        )
    )]
    async fn patch_status<O>(&self, object: &O, status: O::Status) -> Result<()>
    where
        O: Object + Resource,
    {
        Metrics::kubernetes_api_usage_count::<R>("patch");
        self.0
            .patch_status(
                &object.try_name()?,
                &object.patch_status_params(),
                &object.patch_status(status),
            )
            .await
            .map(|_| ())
            .map_err(Error::Kube)
    }
}

impl<R> Api<R>
where
    R: Resource + Clone + std::fmt::Debug + serde::de::DeserializeOwned + serde::Serialize,
    R::Spec: Subset,
{
    #[tracing::instrument(skip_all)]
    pub async fn sync<O, I>(&self, object: &O, resources: HashMap<I, R>) -> Result<HashMap<I, R>>
    where
        I: PartialEq + Eq + std::hash::Hash,
        O: Object + Resource,
    {
        let (results, delete) = self.update(object, resources).await?;
        self.delete_many(object, delete).await?;
        Ok(results)
    }

    #[tracing::instrument(skip_all)]
    pub async fn update<O, I>(
        &self,
        object: &O,
        resources: HashMap<I, R>,
    ) -> Result<(HashMap<I, R>, Vec<R>)>
    where
        I: PartialEq + Eq + std::hash::Hash,
        O: Object + Resource,
    {
        let mut resources = resources
            .into_iter()
            .map(|(identifier, resource)| {
                resource.try_name().and_then(|resource_name| {
                    resource
                        .try_with_owner(object)
                        .map(|resource| (resource_name, (identifier, resource)))
                })
            })
            .collect::<Result<HashMap<ResourceName, (I, R)>>>()?;

        for (resource_name, (_, resource)) in &resources {
            match self.get_opt(resource_name).await? {
                Some(api_resource)
                    if resource.spec().is_subset(api_resource.spec())
                        && resource.meta().is_subset(api_resource.meta()) => {}
                _ => {
                    self.patch(object, resource).await?;
                }
            }
        }

        let mut patched = HashMap::new();
        let mut deprecated = Vec::new();

        for api_resource in self.list(object).await? {
            if let Some((identifier, _)) = resources.remove(&api_resource.try_name()?) {
                patched.insert(identifier, api_resource);
            } else {
                deprecated.push(api_resource);
            }
        }

        assert!(
            resources.is_empty(),
            "{} resources were not patched",
            resources.len()
        );

        Ok((patched, deprecated))
    }
}
