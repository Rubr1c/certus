use std::{path::Path, sync::Arc, time::Duration};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::server::state::{self, app_state::AppState};

use super::parser;

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
/// let _watcher = match watch("config.yaml", app_state.clone(), args.clone()).await {
///     Ok(watcher) => Some(watcher),
///     Err(_) => None
/// };
/// ```
#[inline(always)]
pub async fn watch(
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
    tracing::info!(config_path = path, "Watching config file");

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

                        tracing::info!(config_path = %path, "Realoding config");
                        match parser::reload(&path).await {
                            Ok(new_config) => {
                                state.config.store(Arc::new(new_config));
                                state::initializer::init(state.clone()).await;
                                tracing::info!(
                                    config_path = %path,
                                    "Config hot-reloaded"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    config_path = %path,
                                    "Failed to reload config: {}",
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(config_path = %path, "Watcher error: {}", e);
                }
            }
        }
    });

    Ok(watcher)
}
