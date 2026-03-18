use std::{collections::HashMap, sync::Arc};

use crossbeam::queue::SegQueue;

use crate::{
    db,
    middleware::{cache::static_cache, router},
    upstream::{health, server},
};

use super::{app_state, routing_table};

pub async fn init(state: Arc<app_state::AppState>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();
    let mut healthy_upstreams = 0usize;
    let mut warmed_static_routes = 0usize;

    tracing::info!(
        route_count = config.routes.len(),
        save_config = state.args.save,
        "Rebuilding routing state"
    );

    for (route, route_config) in config.routes.iter() {
        let mut is_static_and_not_fetched = route_config.is_static;

        tracing::debug!(
            route,
            endpoint_count = route_config.endpoints.len(),
            is_static = route_config.is_static,
            needs_auth = route_config.needs_auth,
            no_cache = route_config.no_cache,
            "Initializing route"
        );

        for server in &route_config.endpoints {
            let upstream = Arc::new(server::UpstreamServer::new(
                server.clone(),
                route_config.max_connections,
                route_config.http_version,
            ));

            if is_static_and_not_fetched {
                tracing::debug!(
                    route,
                    upstream = %upstream.pool.server_addr,
                    "Warming static cache for route"
                );
                static_cache::send_and_save(
                    &state.static_cache,
                    &upstream,
                    route,
                    config.connection.connect_timeout,
                )
                .await;
                is_static_and_not_fetched = false;
                warmed_static_routes += 1;
            }

            let ok = health::health_ok(&upstream).await;

            if ok {
                new_routes_map.insert(server.clone(), upstream);
                healthy_upstreams += 1;
            } else {
                tracing::warn!(server = ?server, "Health not ok for server")
            }
        }
    }

    state.idle_queue.clear();
    for (route, route_config) in config.routes.iter() {
        let queue = SegQueue::new();
        for server in &route_config.endpoints {
            if let Some(upstream) = new_routes_map.get(server) {
                queue.push(upstream.clone());
            }
        }
        if !queue.is_empty() {
            state.idle_queue.insert(route.clone(), queue);
        }
    }

    let new_router = router::build_tree(state.clone());

    let new_table = routing_table::RoutingTable {
        router: new_router,
        routes: new_routes_map,
    };

    state.routing_table.store(Arc::new(new_table));
    tracing::info!(
        route_count = config.routes.len(),
        healthy_upstreams,
        idle_queue_count = state.idle_queue.len(),
        warmed_static_routes,
        "Routing state rebuilt"
    );

    if state.args.save {
        let conn_guard = state.db_conn.lock();
        tracing::debug!("Persisting config snapshot to sqlite");
        if let Err(e) = db::repository::config::save(&conn_guard, &config) {
            tracing::error!(err = ?e, "Failed to save config");
        }
    }
}
