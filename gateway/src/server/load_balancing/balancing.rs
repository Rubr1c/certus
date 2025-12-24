use std::{
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
};

use parking_lot::RwLock;
use rand::seq::IndexedRandom;

use crate::{config::cfg_utils::CONFIG, server::app_state::AppState};

pub fn p2c_pick(route: String, state: Arc<RwLock<AppState>>) -> SocketAddr {
    let config_guard = CONFIG.read();
    let target = config_guard.routes.get(&route).expect("Route Should Exist");

    let endpoints = &target.endpoints;
    let mut rng = rand::rng();
    // only power of 2 choices for now

    if endpoints.is_empty() {
        CONFIG.read().default_server
    } else {
        let server1 = endpoints.choose(&mut rng).unwrap();
        let server2 = endpoints.choose(&mut rng).unwrap();

        let state_gaurd = state.read();
        let upstream_server1 = state_gaurd.routes.get(server1).unwrap();
        let upstream_server2 = state_gaurd.routes.get(server2).unwrap();

        if upstream_server1.active_connctions.load(Ordering::Relaxed)
            < upstream_server2.active_connctions.load(Ordering::Relaxed)
        {
            *server1
        } else {
            *server2
        }
    }
}
