use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("YAML parsing failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
