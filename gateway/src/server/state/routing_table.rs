use std::{collections::HashMap, sync::Arc};

use matchit::Router;

use crate::upstream::server::UpstreamServer;

/// Struct holding route related data that should be
/// updated at the same time
pub struct RoutingTable {
    pub router: Router<Arc<str>>,
    pub routes: HashMap<String, Arc<UpstreamServer>>,
}
