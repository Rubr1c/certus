use crate::config::cfg_utils::CONFIG;
use parking_lot::RwLock;
use trie_rs::{Trie, TrieBuilder};

pub static ROUTE_TRIE: RwLock<Option<Trie<String>>> = RwLock::new(None);

pub fn build_tree() {
    let route_conf = &CONFIG.read().routes;

    let routes = route_conf
        .iter()
        .map(|route| {
            route
                .path
                .split("/")
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    let mut builder = TrieBuilder::new();

    for route in routes {
        builder.push(route);
    }

    *ROUTE_TRIE.write() = Some(builder.build());
}

pub fn get_server(route: String) -> String {
    let split_route =
        route.split("/").map(|e| e.to_string()).collect::<Vec<String>>();

    let guard = ROUTE_TRIE.read();
    let routes = match guard.as_ref() {
        Some(trie) => {
            trie.common_prefix_search(split_route).collect::<Vec<Vec<String>>>()
        }
        None => {
            drop(guard);
            build_tree();
            ROUTE_TRIE
                .read()
                .as_ref()
                .unwrap()
                .common_prefix_search(split_route)
                .collect::<Vec<Vec<String>>>()
        }
    };

    let target = routes.iter().max_by(|v1, v2| v1.len().cmp(&v2.len()));

    match target {
        Some(t) => t.join("/"),
        None => "todo".to_string(),
    }
}
