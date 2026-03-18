use std::sync::Arc;

use crate::{MetricEvent, server::state::app_state::AppState};

#[inline(always)]
pub fn try_send(
    state: &AppState,
    event: MetricEvent,
    route: &Arc<str>,
    path: &Arc<str>,
    method: &hyper::Method,
) {
    if let Err(err) = state.metrics_tx.try_send(event) {
        tracing::warn!(
            route = %route,
            path = %path,
            method = %method,
            err = ?err,
            "Dropping request metric"
        );
    }
}
