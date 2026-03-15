use std::{path::Path, sync::Arc, time::Duration};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{fs, sync::mpsc};

use crate::{
    config::Config,
    config::error::ConfigError,
    server::app_state::{self, AppState},
};

/// Watches the config file and when changed updates all related structs
///
/// # Arguments
///
/// * `path` - filepath of the config file
/// * `state` - app state that stores all configs
/// * `args` - command line arguments
///
/// # Errors
///
/// Returns an error if:
/// * Watcher failes to initalize
/// * Failed to read file due to incorrect path or other reason
///
/// # Example
/// ```ignore
/// let app_state: Arc<AppState> = Arc::new(/* */);
/// let args = Arc::new(CmdArgs::try_parse().unwrap());
/// let _watcher = match watch_config("config.yaml", app_state.clone(), args.clone()).await {
///     Ok(watcher) => Some(watcher),
///     Err(_) => None
/// };
/// ```
pub async fn watch_config(
    path: &str,
    state: Arc<AppState>,
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

                        tracing::info!("Realoding config");
                        match reload_config(&path).await {
                            Ok(new_config) => {
                                state.config.store(Arc::new(new_config));
                                app_state::init_server_state(state.clone())
                                    .await;
                                tracing::info!("Config hot-reloaded");
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to reload config: {}",
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Watcher error: {}", e);
                }
            }
        }
    });

    Ok(watcher)
}

/// Reads a file and parses it to yaml for the config
///
/// # Arguments
///
/// * `path` - filepath of the config file
///
/// # Errors
///
/// Returns an error if:
/// * Failed to read file due to incorrect path or other reason
/// * Failed to parse to yaml
///
/// # Examples
///
/// ```ignore
/// let config = match reload_config("config.yaml").await {
///     Ok(c) => c,
///     Err(_) => Config::default(),
/// };
/// ```
pub async fn reload_config(path: &str) -> Result<Config, ConfigError> {
    let contents = fs::read_to_string(path).await?;

    Ok(serde_yaml::from_str::<Config>(&contents)?)
}
