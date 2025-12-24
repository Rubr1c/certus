use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use parking_lot::RwLock;

use crate::server::models::{UpstreamServer, Protocol};
use crate::config::cfg_utils::CONFIG;

#[derive(Default, Clone)]
pub struct AppState {
    pub routes: HashMap<SocketAddr, Arc<UpstreamServer>>,
}

pub fn init_server_state(state: Arc<RwLock<AppState>>) {
    let conf = CONFIG.read();
    for route in &conf.routes {
        for server in &route.1.endpoints {
            let upstream = Arc::new(UpstreamServer::new(
                *server,
                //TODO: make dynamic from config
                100,
                Protocol::HTTP1,
            ));

            state.write().routes.insert(*server, upstream);
        }
    }
}
