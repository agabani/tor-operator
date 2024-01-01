use k8s_openapi::api::core::v1::PodSecurityContext;

use super::constants;

pub fn pod_security_context(mut value: PodSecurityContext) -> PodSecurityContext {
    if value.fs_group.is_none() {
        value.fs_group = Some(constants::POD_SECURITY_CONTEXT_FS_GROUP);
    }

    if value.run_as_non_root.is_none() {
        value.run_as_non_root = Some(constants::POD_SECURITY_CONTEXT_RUN_AS_NON_ROOT);
    }

    if value.run_as_user.is_none() {
        value.run_as_user = Some(constants::POD_SECURITY_CONTEXT_RUN_AS_USER);
    }

    value
}
