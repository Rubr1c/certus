use std::{
    cell::RefCell,
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
};

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use tracing::instrument;

use crate::{config::RouteConfig, server::upstream::UpstreamServer};

thread_local! {
    /// Small random number generator that has one instance per thread
    /// to avoid too many syscalls
    static THREAD_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
}

// only power of 2 choices for now

/// Picks randomly between 2 servers and picks the one
/// with the least load
///
/// # Arguments
///
/// * `routes` - map of all addresses to servers
/// * `target` - the config for the route targeted
/// * `config` - gateway config
#[inline]
#[instrument(name = "lb_p2c", skip_all)]
pub fn p2c_pick<'a>(
    routes: &'a HashMap<SocketAddr, Arc<UpstreamServer>>,
    target: &'a RouteConfig,
    default_server: &'a SocketAddr,
) -> &'a SocketAddr {
    tracing::info!("Finding endpoint");
    let endpoints = &target.endpoints;
    if endpoints.is_empty() {
        tracing::warn!("No endpoints found");
        tracing::info!("returning default server");
        return default_server;
    }

    THREAD_RNG.with(|rng_cell| {
        let mut rng = rng_cell.borrow_mut();

        let server1 = endpoints.choose(&mut rng).unwrap();
        let server2 = endpoints.choose(&mut rng).unwrap();

        let upstream_server1 = routes.get(server1).unwrap();
        let upstream_server2 = routes.get(server2).unwrap();

        tracing::info!("Selecting server will least load");
        if upstream_server1.active_connctions.load(Ordering::Acquire)
            < upstream_server2.active_connctions.load(Ordering::Acquire)
        {
            server1
        } else {
            server2
        }
    })
}
