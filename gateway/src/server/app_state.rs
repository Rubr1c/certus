use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use bb8_redis::RedisConnectionManager;
use dashmap::DashMap;
use matchit::Router;
use moka::sync::Cache;
use parking_lot::Mutex;
use rusqlite::Connection;

use crate::{
    config::{CacheType, CmdArgs, Config},
    db::db_utils,
    server::{
        connection,
        middleware::{
            cache::{
                CacheKey, CachedResponse, DynCacheBackend, StaticCacheBackend,
                static_cache,
            },
            rate_limit::TokenBucket,
            router,
        },
        upstream::UpstreamServer,
    },
};

/// Struct holding route related data that should be
/// updated at the same time
pub struct RoutingTable {
    pub router: Router<String>,
    pub routes: HashMap<String, Arc<UpstreamServer>>,
}

/// Holds state of whole app passed to the reroute function
pub struct AppState {
    pub routing_table: ArcSwap<RoutingTable>,
    pub config: ArcSwap<Config>,
    pub cache: DynCacheBackend,
    pub static_cache: StaticCacheBackend,
    pub user_tokens: DashMap<IpAddr, TokenBucket>,
    pub db_conn: Arc<Mutex<Connection>>,
}

impl AppState {
    pub async fn new(config: Config, conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(RoutingTable {
                router: Router::new(),
                routes: HashMap::new(),
            }),
            cache: match &config.cache.cache_type {
                CacheType::InMemory => {
                    let mut cache =
                        Cache::<CacheKey, CachedResponse>::builder();
                    match config.cache.ttl {
                        Some(secs) => {
                            cache =
                                cache.time_to_live(Duration::from_secs(secs));
                        }
                        _ => {}
                    };

                    match config.cache.tti {
                        Some(secs) => {
                            cache =
                                cache.time_to_idle(Duration::from_secs(secs));
                        }
                        _ => {}
                    }

                    DynCacheBackend::InMemory(
                        cache.max_capacity(config.cache.size).build(),
                    )
                }
                CacheType::Redis { url } => {
                    DynCacheBackend::Redis(create_pool(url).await)
                }
            },
            static_cache: match &config.cache.cache_type {
                CacheType::InMemory => {
                    StaticCacheBackend::InMemory(DashMap::new())
                }
                CacheType::Redis { url } => {
                    StaticCacheBackend::Redis(create_pool(url).await)
                }
            },
            config: ArcSwap::from_pointee(config),
            user_tokens: DashMap::new(),
            db_conn: conn,
        }
    }
}

pub async fn create_pool(url: &String) -> bb8::Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(url.as_str())
        .expect("Failed to open redis client");

    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to make connection bb8 pool")
}

pub async fn init_server_state(state: Arc<AppState>, args: Arc<CmdArgs>) {
    let config = state.config.load();

    let mut new_routes_map = HashMap::new();

    for (route, route_config) in config.routes.iter() {
        let mut is_static_and_not_fetched = route_config.is_static;

        for server in &route_config.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
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

            let ok = connection::health_ok(&upstream).await;

            if ok {
                new_routes_map.insert(server.clone(), upstream);
            } else {
                tracing::error!(server = ?server, "Health not ok for server")
            }
        }
    }

    let new_router = router::build_tree(state.clone());

    let new_table = RoutingTable { router: new_router, routes: new_routes_map };

    state.routing_table.store(Arc::new(new_table));

    if args.save {
        let conn_guard = state.db_conn.lock();
        if let Err(e) = db_utils::save_config(&conn_guard, &config) {
            tracing::error!(err = ?e, "Failed to save config");
        }
    }
}
