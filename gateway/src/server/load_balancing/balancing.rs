use std::{
    cell::RefCell,
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
};

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};

use crate::server::app_state::AppState;

thread_local! {
    static THREAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
}

pub fn p2c_pick(route: &str, state: Arc<AppState>) -> SocketAddr {
    let config = state.config.load();
    let routes = state.routes.load();
    let target = config.routes.get(route).expect("Route Should Exist");

    let endpoints = &target.endpoints;
    // only power of 2 choices for now
    if endpoints.is_empty() {
        return config.default_server;
    }

    THREAD_RNG.with(|rng_cell| {
        let mut rng = rng_cell.borrow_mut();

        let server1 = endpoints.choose(&mut rng).unwrap();
        let server2 = endpoints.choose(&mut rng).unwrap();

        let upstream_server1 = routes.get(server1).unwrap();
        let upstream_server2 = routes.get(server2).unwrap();

        if upstream_server1.active_connctions.load(Ordering::Acquire)
            < upstream_server2.active_connctions.load(Ordering::Acquire)
        {
            *server1
        } else {
            *server2
        }
    })
}
