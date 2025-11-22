use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("CLI argument parsing failed: {0}")]
    Clap(#[from] clap::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
