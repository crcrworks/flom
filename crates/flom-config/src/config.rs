use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    pub odesli_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub target: Option<String>,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self { target: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub simple: Option<bool>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { simple: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlomConfig {
    pub api: ApiConfig,
    pub default: DefaultConfig,
    pub output: OutputConfig,
}
