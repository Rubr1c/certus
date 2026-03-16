use std::{collections::HashMap, sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use moka::sync::Cache;
use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::{
    cli,
    config::types,
    connection, logging, metrics,
    middleware::{cache, rate_limit},
    schema,
    upstream::server,
};

use super::routing_table;

/// Holds state of whole app passed to the reroute function
pub struct AppState {
    pub routing_table: ArcSwap<routing_table::RoutingTable>,
    pub config: ArcSwap<types::Config>,
    pub cache: cache::backend::DynBackend,
    pub static_cache: cache::backend::StaticBackend,
    pub user_tokens: rate_limit::backend::DynBackend,
    pub db_conn: Arc<Mutex<rusqlite::Connection>>,
    pub idle_queue: DashMap<String, SegQueue<Arc<server::UpstreamServer>>>,
    pub metrics_tx: mpsc::Sender<metrics::types::MetricEvent>,
    pub schema_tx: mpsc::Sender<schema::types::ReqResSchemaDTO>,
    pub log_tx: Option<broadcast::Sender<logging::types::LogEntryDTO>>,
    pub metrics_broadcast_tx:
        Option<broadcast::Sender<metrics::types::MetricEvent>>,
    pub args: Arc<cli::CmdArgs>,
}

impl AppState {
    pub async fn new(
        config: types::Config,
        conn: Arc<Mutex<rusqlite::Connection>>,
        metrics_tx: mpsc::Sender<metrics::types::MetricEvent>,
        schema_tx: mpsc::Sender<schema::types::ReqResSchemaDTO>,
        log_tx: Option<broadcast::Sender<logging::types::LogEntryDTO>>,
        metrics_broadcast_tx: Option<
            broadcast::Sender<metrics::types::MetricEvent>,
        >,
        args: Arc<cli::CmdArgs>,
    ) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(routing_table::RoutingTable {
                router: matchit::Router::new(),
                routes: HashMap::new(),
            }),
            cache: match &config.cache.cache_type {
                types::StorageType::InMemory => {
                    let mut cache = Cache::<
                        cache::key::OwnedCacheKey,
                        cache::response::CachedResponse,
                    >::builder();
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

                    cache::backend::DynBackend::InMemory(
                        cache.max_capacity(config.cache.size).build(),
                    )
                }
                types::StorageType::Redis { url } => {
                    let pool = connection::redis::create_pool(url).await;
                    cache::backend::DynBackend::Redis {
                        pool,
                        ttl: config.cache.ttl,
                    }
                }
            },
            static_cache: match &config.cache.cache_type {
                types::StorageType::InMemory => {
                    cache::backend::StaticBackend::InMemory(DashMap::new())
                }
                types::StorageType::Redis { url } => {
                    cache::backend::StaticBackend::Redis(
                        connection::redis::create_pool(url).await,
                    )
                }
            },
            user_tokens: match &config.rate_limit.rl_type {
                types::StorageType::InMemory => {
                    rate_limit::backend::DynBackend::InMemory(
                        Cache::builder()
                            .time_to_idle(Duration::from_secs(3600))
                            .max_capacity(100000)
                            .build(),
                    )
                }
                types::StorageType::Redis { url } => {
                    rate_limit::backend::DynBackend::Redis(
                        connection::redis::create_pool(url).await,
                    )
                }
            },
            idle_queue: DashMap::new(),
            config: ArcSwap::from_pointee(config),
            db_conn: conn,
            metrics_tx,
            schema_tx,
            log_tx,
            metrics_broadcast_tx,
            args,
        }
    }
}
