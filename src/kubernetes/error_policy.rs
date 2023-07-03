use std::{sync::Arc, time::Duration};

use kube::runtime::controller::Action;

use crate::Error;

use super::{Context, Object};

#[allow(clippy::needless_pass_by_value)]
pub fn error_policy<O, C>(_: Arc<O>, error: &Error, ctx: Arc<C>) -> Action
where
    O: Object,
    C: Context,
{
    tracing::warn!(error =% error, "failed to reconcile");
    ctx.metrics()
        .reconcile_failure(O::APP_KUBERNETES_IO_COMPONENT_VALUE, error);
    Action::requeue(Duration::from_secs(5))
}
