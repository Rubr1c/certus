use std::sync::Arc;

use crate::{ReqResSchemaDTO, server::state::app_state::AppState};

#[inline(always)]
pub fn try_send(
    state: &AppState,
    schema: ReqResSchemaDTO,
    path: &Arc<str>,
    method: &hyper::Method,
    status_code: hyper::StatusCode,
) {
    if let Err(err) = state.schema_tx.try_send(schema) {
        tracing::warn!(
            path = %path,
            method = %method,
            status_code = status_code.as_u16(),
            err = ?err,
            "Dropping request schema"
        );
    }
}
