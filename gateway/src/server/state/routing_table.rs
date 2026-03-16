use std::{collections::HashMap, sync::Arc};

use crate::upstream::server;

/// Struct holding route related data that should be
/// updated at the same time
pub struct RoutingTable {
    pub router: matchit::Router<Arc<str>>,
    pub routes: HashMap<String, Arc<server::UpstreamServer>>,
}
