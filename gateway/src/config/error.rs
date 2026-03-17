use thiserror::Error;

/// An error type for any error that can happen related to the config
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("YAML parsing failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
