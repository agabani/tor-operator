use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// VolumeMount describes a mounting of a Volume within a container.
#[derive(JsonSchema, Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMount {
    /// Path within the container at which the volume should be mounted.  Must not contain ':'.
    pub mount_path: String,

    /// mountPropagation determines how mounts are propagated from the host to container and the other way around. When not set, MountPropagationNone is used. This field is beta in 1.10.
    pub mount_propagation: Option<String>,

    /// Mounted read-only if true, read-write otherwise (false or unspecified). Defaults to false.
    pub read_only: Option<bool>,

    /// Path within the volume from which the container's volume should be mounted. Defaults to "" (volume's root).
    pub sub_path: Option<String>,

    /// Expanded path within the volume from which the container's volume should be mounted. Behaves similarly to SubPath but environment variable references $(VAR_NAME) are expanded using the container's environment. Defaults to "" (volume's root). SubPathExpr and SubPath are mutually exclusive.
    pub sub_path_expr: Option<String>,
}

impl VolumeMount {
    pub fn to_volume_mount(self, name: String) -> k8s_openapi::api::core::v1::VolumeMount {
        k8s_openapi::api::core::v1::VolumeMount {
            mount_path: self.mount_path,
            mount_propagation: self.mount_propagation,
            name,
            read_only: self.read_only,
            sub_path: self.sub_path,
            sub_path_expr: self.sub_path_expr,
        }
    }
}
