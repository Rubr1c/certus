use std::{collections::HashMap, sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use matchit::Router;
use moka::sync::Cache;
use parking_lot::Mutex;
use rusqlite::Connection;
use tokio::sync::{broadcast, mpsc};

use crate::{
    cli::CmdArgs,
    config::types::{Config, StorageType},
    connection,
    logging::types::LogEntryDTO,
    metrics::types::MetricEvent,
    middleware::{
        cache::{
            backend::{DynCacheBackend, StaticCacheBackend},
            key::OwnedCacheKey,
            response::CachedResponse,
        },
        rate_limit::backend::DynRateLimitBackend,
    },
    schema::types::ReqResSchemaDTO,
    upstream::server::UpstreamServer,
};

use super::routing_table::RoutingTable;

/// Holds state of whole app passed to the reroute function
pub struct AppState {
    pub routing_table: ArcSwap<RoutingTable>,
    pub config: ArcSwap<Config>,
    pub cache: DynCacheBackend,
    pub static_cache: StaticCacheBackend,
    pub user_tokens: DynRateLimitBackend,
    pub db_conn: Arc<Mutex<Connection>>,
    pub idle_queue: DashMap<String, SegQueue<Arc<UpstreamServer>>>,
    pub metrics_tx: mpsc::Sender<MetricEvent>,
    pub schema_tx: mpsc::Sender<ReqResSchemaDTO>,
    pub log_tx: Option<broadcast::Sender<LogEntryDTO>>,
    pub metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>>,
    pub args: Arc<CmdArgs>,
}

impl AppState {
    pub async fn new(
        config: Config,
        conn: Arc<Mutex<Connection>>,
        metrics_tx: mpsc::Sender<MetricEvent>,
        schema_tx: mpsc::Sender<ReqResSchemaDTO>,
        log_tx: Option<broadcast::Sender<LogEntryDTO>>,
        metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>>,
        args: Arc<CmdArgs>,
    ) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(RoutingTable {
                router: Router::new(),
                routes: HashMap::new(),
            }),
            cache: match &config.cache.cache_type {
                StorageType::InMemory => {
                    let mut cache =
                        Cache::<OwnedCacheKey, CachedResponse>::builder();
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
                StorageType::Redis { url } => {
                    let pool = connection::redis::create_pool(url).await;
                    DynCacheBackend::Redis { pool, ttl: config.cache.ttl }
                }
            },
            static_cache: match &config.cache.cache_type {
                StorageType::InMemory => {
                    StaticCacheBackend::InMemory(DashMap::new())
                }
                StorageType::Redis { url } => StaticCacheBackend::Redis(
                    connection::redis::create_pool(url).await,
                ),
            },
            user_tokens: match &config.rate_limit.rl_type {
                StorageType::InMemory => DynRateLimitBackend::InMemory(
                    Cache::builder()
                        .time_to_idle(Duration::from_secs(3600))
                        .max_capacity(100000)
                        .build(),
                ),
                StorageType::Redis { url } => DynRateLimitBackend::Redis(
                    connection::redis::create_pool(url).await,
                ),
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
