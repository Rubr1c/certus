use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc};

use crate::{
    cli,
    config::Config,
    logging::LogEntryDTO,
    metrics::MetricEvent,
    middleware::{cache, rate_limit},
    schema::ReqResSchemaDTO,
    upstream::server,
};

use super::routing_table;

/// Holds state of whole app passed to the reroute function
pub struct AppState {
    pub routing_table: ArcSwap<routing_table::RoutingTable>,
    pub config: ArcSwap<Config>,
    pub cache: cache::backend::DynBackend,
    pub static_cache: cache::backend::StaticBackend,
    pub user_tokens: rate_limit::backend::DynBackend,
    pub db_conn: Arc<Mutex<rusqlite::Connection>>,
    pub idle_queue: DashMap<String, SegQueue<Arc<server::UpstreamServer>>>,
    pub metrics_tx: mpsc::Sender<MetricEvent>,
    pub schema_tx: mpsc::Sender<ReqResSchemaDTO>,
    pub log_tx: Option<broadcast::Sender<LogEntryDTO>>,
    pub metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>>,
    pub args: Arc<cli::CmdArgs>,
}

impl AppState {
    pub async fn new(
        config: Config,
        conn: Arc<Mutex<rusqlite::Connection>>,
        metrics_tx: mpsc::Sender<MetricEvent>,
        schema_tx: mpsc::Sender<ReqResSchemaDTO>,
        log_tx: Option<broadcast::Sender<LogEntryDTO>>,
        metrics_broadcast_tx: Option<broadcast::Sender<MetricEvent>>,
        args: Arc<cli::CmdArgs>,
    ) -> Self {
        Self {
            routing_table: ArcSwap::from_pointee(routing_table::RoutingTable {
                router: matchit::Router::new(),
                routes: HashMap::new(),
            }),
            cache: cache::backend::build(&config.cache).await,
            static_cache: cache::backend::build_static(&config.cache).await,
            user_tokens: rate_limit::backend::build(&config.rate_limit).await,
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
