use crate::id::ModuleID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoamiResponse {
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishResponse {
    pub author: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub author: String,
    pub name: String,
    pub version: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfoResponse {
    pub author: String,
    pub name: String,
    pub description: String,
    pub latest: Option<String>,
    pub redirect: Option<ModuleID>,
}

impl ModuleInfoResponse {
    pub fn canonical(&self) -> String {
        format!("{}/{}", self.author, self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub author: String,
    pub name: String,
    pub description: String,
    pub latest: String,
    pub downloads: i64,
    pub featured: bool,
}

impl SearchResponse {
    pub fn canonical(&self) -> String {
        format!("{}/{}", self.author, self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestResponse {
    pub time: Option<u64>,
}
