use std::{
    net::SocketAddr,
    sync::{atomic::Ordering, Arc},
};

use rand::{SeedableRng, seq::IndexedRandom, rngs::SmallRng};

use crate::{server::app_state::AppState};

pub fn p2c_pick(route: &str, state: Arc<AppState>) -> SocketAddr {
    let config = state.config.load();
    let routes = state.routes.load();
    let target = config.routes.get(route).expect("Route Should Exist");

    let endpoints = &target.endpoints;
    let mut rng = SmallRng::from_os_rng();
    // only power of 2 choices for now

    if endpoints.is_empty() {
        config.default_server
    } else {
        let server1 = endpoints.choose(&mut rng).unwrap();
        let server2 = endpoints.choose(&mut rng).unwrap();

        let upstream_server1 = routes.get(server1).unwrap();
        let upstream_server2 = routes.get(server2).unwrap();

        if upstream_server1.active_connctions.load(Ordering::Relaxed)
            < upstream_server2.active_connctions.load(Ordering::Relaxed)
        {
            *server1
        } else {
            *server2
        }
    }
}
