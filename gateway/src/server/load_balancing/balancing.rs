use std::{
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
};

use parking_lot::RwLock;
use rand::seq::IndexedRandom;

use crate::{config::cfg_utils::CONFIG, server::app_state::AppState};

pub fn p2c_pick(
    route: String,
    state: Arc<RwLock<AppState>>,
) -> SocketAddr {
    let state_guard = state.read();
    let target = state_guard.routes.get(&route);
    let mut rng = rand::rng();
    // only power of 2 choices for now
    match target {
        Some(adrs) => {
            if adrs.is_empty() {
                CONFIG.read().default_server
            } else {
                let server1 = adrs.choose(&mut rng).unwrap();
                let server2 = adrs.choose(&mut rng).unwrap();

                if server1.active_connctions.load(Ordering::Relaxed)
                    < server2.active_connctions.load(Ordering::Relaxed)
                {
                    server1.address
                } else {
                    server2.address
                }
            }
        }
        None => CONFIG.read().default_server,
    }
}
