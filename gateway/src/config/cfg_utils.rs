use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::config::error::ConfigError;
use crate::config::models::Config;
use crate::server::app_state::AppState;
use crate::server::models::{Protocol, UpstreamServer};
use crate::server::routing::routes;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

pub fn watch_config(
    path: &str,
    state: Arc<RwLock<AppState>>,
) -> notify::Result<RecommendedWatcher> {
    let (tx, mut rx) = mpsc::channel(1);

    // blocking_send is used because the notify callback runs in a sync context
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            let _ = tx.blocking_send(res);
        })?;

    watcher.watch(Path::new(path), RecursiveMode::NonRecursive)?;
    println!("Watching config file: {}", path);

    let path = Arc::new(path.to_string());

    tokio::spawn(async move {
        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // Debounce: Wait 150ms to let file writes settle and coalesce events
                        tokio::time::sleep(Duration::from_millis(150)).await;

                        // Drain any other events that occurred during the sleep
                        while rx.try_recv().is_ok() {}

                        println!("Reloading config...");
                        match reload_config(&path) {
                            Ok(new_config) => {
                                *CONFIG.write() = new_config;
                                routes::build_tree();
                                //TODO: Move to seperate function
                                let conf = CONFIG.read();
                                for route in &conf.routes {
                                    let mut servers =
                                        Vec::<Arc<UpstreamServer>>::new();
                                    //TODO!:THESE DO NOT INITALIZE UNTIL RELOAD
                                    for server in &route.1.endpoints {
                                        servers.push(Arc::new(
                                            UpstreamServer::new(
                                                *server,
                                                //TODO: make dynamic from config
                                                100,
                                                Protocol::HTTP2,
                                            ),
                                        ));
                                    }

                                    state
                                        .write()
                                        .routes
                                        .insert(route.0.clone(), servers);
                                }
                                println!("Config hot-reloaded");
                            }
                            Err(e) => {
                                eprintln!("Failed to reload config: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watcher error: {}", e);
                }
            }
        }
    });

    Ok(watcher)
}

pub fn reload_config(path: &str) -> Result<Config, ConfigError> {
    let contents = fs::read_to_string(path);

    let config = match contents {
        Ok(contents) => serde_yaml::from_str::<Config>(&contents)?,
        Err(_) => Config::default(),
    };

    Ok(config)
}

pub fn save_config() -> rusqlite::Result<()> {
    todo!()
}
