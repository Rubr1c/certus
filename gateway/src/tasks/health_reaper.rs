use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use crate::{
    middleware::router,
    server::state::{app_state::AppState, routing_table::RoutingTable},
    upstream::{health, server::HealthState},
};

#[inline(always)]
pub async fn run(state: Arc<AppState>) {
    let state_clone = state.clone();

    tracing::info!("Starting upstream health monitor");

    // check and mark
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let table = state.routing_table.load();
            for (addr, upstream) in &table.routes {
                if !health::health_ok(upstream).await {
                    tracing::warn!(server = %addr, "Health check failed");
                } else {
                    tracing::debug!(server = %addr, "Health check passed");
                }
            }
        }
    });

    // remove
    tokio::spawn(async move {
        tracing::info!("Starting dead upstream reaper");
        loop {
            //TODO: make configable
            tokio::time::sleep(Duration::from_secs(5)).await;

            let table = state_clone.routing_table.load();
            let dead_addrs: Vec<String> = table
                .routes
                .iter()
                .filter(|(_, upstream)| {
                    upstream.health_state.load(Ordering::Acquire)
                        == HealthState::Dead as u8
                })
                .map(|(addr, _)| addr.clone())
                .collect();

            if dead_addrs.is_empty() {
                continue;
            }

            for addr in &dead_addrs {
                tracing::warn!(server = %addr, "Removing dead upstream");
            }

            let mut new_routes = table.routes.clone();
            for addr in &dead_addrs {
                new_routes.remove(addr);
            }

            let new_router = router::build_tree(state_clone.clone());
            let new_table =
                RoutingTable { router: new_router, routes: new_routes };
            state_clone.routing_table.store(Arc::new(new_table));
            tracing::warn!(
                removed_upstream_count = dead_addrs.len(),
                remaining_upstream_count =
                    state_clone.routing_table.load().routes.len(),
                "Removed dead upstreams from routing table"
            );

            for entry in state_clone.idle_queue.iter_mut() {
                let queue = entry.value();
                let len = queue.len();
                for _ in 0..len {
                    if let Some(upstream) = queue.pop() {
                        if upstream.health_state.load(Ordering::Acquire)
                            != HealthState::Dead as u8
                        {
                            queue.push(upstream);
                        }
                    }
                }
            }
        }
    });
}
