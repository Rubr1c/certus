use std::{collections::HashMap, sync::Arc};

use crate::server::models::UpstreamServer;

#[derive(Default, Clone)]
pub struct AppState {
    pub routes: HashMap<String, Vec<Arc<UpstreamServer>>>,
}
