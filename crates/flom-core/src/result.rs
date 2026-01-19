use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub source_url: String,
    pub target_url: Option<String>,
    pub source_platform: Option<String>,
    pub target_platform: Option<String>,
    pub source_info: Option<MediaInfo>,
    pub target_info: Option<MediaInfo>,
    pub warning: Option<String>,
}
