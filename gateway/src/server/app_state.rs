use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::server::models::UpstreamServer;

#[derive(Default, Clone)]
pub struct AppState {
    pub routes: HashMap<SocketAddr, Arc<UpstreamServer>>,
}
