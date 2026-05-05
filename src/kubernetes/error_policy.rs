use std::sync::Arc;

use kube::runtime::controller::Action;

use crate::Error;

use super::{Context, Object, Resource};

#[allow(clippy::needless_pass_by_value)]
pub fn error_policy<O, C>(object: Arc<O>, error: &Error, ctx: Arc<C>) -> Action
where
    O: Object + Resource,
    C: Context,
{
    tracing::warn!(error =% error, "failed to reconcile");
    ctx.metrics()
        .reconcile_failure(O::APP_KUBERNETES_IO_COMPONENT_VALUE, error);
    Action::requeue(ctx.error_backoff().next_delay(object.as_ref()))
}
