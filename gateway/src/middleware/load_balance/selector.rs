use std::{collections::HashMap, sync::Arc};

use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use tracing::instrument;

use crate::{config::RouteConfig, upstream::server};

use super::p2c;

#[inline(always)]
#[instrument(name = "lb", skip_all)]
pub fn run<'a>(
    routes: &'a HashMap<String, Arc<server::UpstreamServer>>,
    target: (&'a String, &'a RouteConfig),
    default_server: &'a String,
    idle_queue: &'a DashMap<String, SegQueue<Arc<server::UpstreamServer>>>,
) -> &'a String {
    if idle_queue.is_empty() {
        tracing::debug!("Idle Queue is empty");
        return p2c::p2c_pick(routes, target.1, default_server);
    }

    match idle_queue.get(target.0) {
        Some(q) => match q.pop() {
            Some(server) => routes
                .get_key_value(server.pool.server_addr.as_ref())
                .map(|(key, _)| key)
                .unwrap_or_else(|| {
                    tracing::debug!("Idle Queue is empty");
                    p2c::p2c_pick(routes, target.1, default_server)
                }),
            _ => {
                tracing::debug!("Idle Queue is empty");
                p2c::p2c_pick(routes, target.1, default_server)
            }
        },
        _ => p2c::p2c_pick(routes, target.1, default_server),
    }
}
