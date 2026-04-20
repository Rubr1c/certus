use serde::Deserialize;

pub mod args;
pub mod config;
pub mod docs;
pub mod log;
pub mod metrics;
pub mod route;
pub mod schema;

#[inline(always)]
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
