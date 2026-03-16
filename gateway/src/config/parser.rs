use tokio::fs;

use super::{error::ConfigError, types::Config};

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
