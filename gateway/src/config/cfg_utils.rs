use std::fs;

use clap::Parser;

use crate::config::error::ConfigError;
use crate::config::models::{CmdArgs, Config};

pub fn read_config() -> Result<Config, ConfigError> {
    let args = CmdArgs::try_parse()?;

    let config_path = args
        .config
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("certus.config.yaml");

    let contents = fs::read_to_string(config_path)?;

    let config = serde_yaml::from_str::<Config>(&contents)?;

    Ok(config)
}
