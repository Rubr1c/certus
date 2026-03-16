use std::{collections::HashMap, sync::Arc};

use crossbeam::queue::SegQueue;

use crate::{
    db::repository::config_repo,
    middleware::{cache::static_cache, router},
    upstream::{health, server},
};

use super::{app_state, routing_table};

pub async fn init_server_state(state: Arc<app_state::AppState>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();

    for (route, route_config) in config.routes.iter() {
        let mut is_static_and_not_fetched = route_config.is_static;

        for server in &route_config.endpoints {
            let upstream = Arc::new(server::UpstreamServer::new(
                server.clone(),
                route_config.max_connections,
                route_config.http_version,
            ));

            if is_static_and_not_fetched {
                static_cache::send_and_save(
                    &state.static_cache,
                    &upstream,
                    route,
                    config.connection.connect_timeout,
                )
                .await;
                is_static_and_not_fetched = false;
            }

            let ok = health::health_ok(&upstream).await;

            if ok {
                new_routes_map.insert(server.clone(), upstream);
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

    if state.args.save {
        let conn_guard = state.db_conn.lock();
        if let Err(e) = config_repo::save_config(&conn_guard, &config) {
            tracing::error!(err = ?e, "Failed to save config");
        }
    }
}
