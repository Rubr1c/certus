use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, atomic::Ordering},
};

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use tracing::instrument;

use crate::{config::types::RouteConfig, upstream::server::UpstreamServer};

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
#[instrument(name = "p2c", skip_all)]
pub fn p2c_pick<'a>(
    routes: &'a HashMap<String, Arc<UpstreamServer>>,
    target: &'a RouteConfig,
    default_server: &'a String,
) -> &'a String {
    tracing::info!("Finding endpoint");
    let endpoints = &target.endpoints;
    if endpoints.is_empty() {
        tracing::warn!("No endpoints found");
        tracing::info!("returning default server");
        return default_server;
    }

    if endpoints.len() == 1 {
        return &endpoints[0];
    }

    THREAD_RNG.with(|rng_cell| {
        let mut rng = rng_cell.borrow_mut();

        let [addr1, addr2]: [String; 2] =
            endpoints.choose_multiple_array(&mut *rng).unwrap();

        let (key1, upstream1) = routes.get_key_value(addr1.as_str()).unwrap();
        let (key2, upstream2) = routes.get_key_value(addr2.as_str()).unwrap();

        tracing::info!("Selecting server with least load");
        if upstream1.active_connctions.load(Ordering::Acquire)
            <= upstream2.active_connctions.load(Ordering::Acquire)
        {
            key1
        } else {
            key2
        }
    })
}
