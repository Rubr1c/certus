use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use parking_lot::RwLock;

use crate::config::models::Config;
use crate::server::models::{UpstreamServer, Protocol};

#[derive(Default, Clone)]
pub struct AppState {
    pub routes: HashMap<SocketAddr, Arc<UpstreamServer>>,
    pub config: Arc<RwLock<Config>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self { routes: HashMap::new(), config: Arc::new(RwLock::new(config)) }
    }
}


pub fn init_server_state(state: Arc<RwLock<AppState>>) {
    let mut state_guard = state.write();
    let config_arc = state_guard.config.clone();
    let config_guard = config_arc.read();
    for route in &config_guard.routes {
        for server in &route.1.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
                *server,
                //TODO: make dynamic from config
                100,
                Protocol::HTTP1,
            ));

            state_guard.routes.insert(*server, upstream);
        }
    }
}
