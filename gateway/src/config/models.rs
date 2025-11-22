use clap::Parser;
use clap::command;
use serde::Deserialize;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CmdArgs {
    #[arg(short, long)]
    pub config: Option<String>,
}

//TEST
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
}
