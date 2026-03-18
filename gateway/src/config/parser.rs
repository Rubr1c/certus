use tokio::fs;

use super::{Config, error::ConfigError};

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
/// let config = match reload("config.yaml").await {
///     Ok(c) => c,
///     Err(_) => Config::default(),
/// };
/// ```
pub async fn reload(path: &str) -> Result<Config, ConfigError> {
    tracing::debug!(config_path = path, "Reading config file");
    let contents = fs::read_to_string(path).await?;

    Ok(serde_yaml::from_str::<Config>(&contents)?)
}
