use serde::Deserialize;

pub mod log_controller;
pub mod metrics_controller;
pub mod schema_controller;

fn default_per_page() -> u32 {
    20
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}
