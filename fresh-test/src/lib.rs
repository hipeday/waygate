use serde::{Deserialize, Serialize};

pub mod macros;

#[derive(Debug, Deserialize)]
pub struct HttpBinGet {
    pub url: String,
    pub args: serde_json::Value,
    pub headers: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SearchQuery {
    pub q: String,
    pub page: u32,
}