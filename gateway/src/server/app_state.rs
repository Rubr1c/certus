use std::sync::Arc;

use crate::server::models::UpstreamServer;

#[derive(Default, Clone)]
pub struct AppState {
    pub routes: Vec<Arc<UpstreamServer>>,
}
