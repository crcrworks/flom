mod config;

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::config::FlomConfig;
use flom_core::{FlomError, FlomResult};

pub use config::{ApiConfig, DefaultConfig, FlomConfig as FlomConfigData, OutputConfig};

pub fn config_path() -> FlomResult<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| FlomError::Config("home directory not found".to_string()))?;
    Ok(home.join(".flom").join("config.toml"))
}

pub fn load_config() -> FlomResult<FlomConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(FlomConfig::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|err| FlomError::Config(format!("failed to read config: {err}")))?;
    let config = toml::from_str(&content)
        .map_err(|err| FlomError::Config(format!("failed to parse config: {err}")))?;
    Ok(config)
}

pub fn save_config(config: &FlomConfig) -> FlomResult<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| FlomError::Config(format!("failed to create config dir: {err}")))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|err| FlomError::Config(format!("failed to serialize config: {err}")))?;
    fs::write(&path, content)
        .map_err(|err| FlomError::Config(format!("failed to write config: {err}")))?;
    Ok(())
}

pub fn resolve_odesli_key(config: &FlomConfig) -> Option<String> {
    if let Ok(value) = env::var("FLOM_ODESLI_KEY") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    config.api.odesli_key.clone()
}

pub fn resolve_default_target(config: &FlomConfig) -> Option<String> {
    if let Ok(value) = env::var("FLOM_DEFAULT_TARGET") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    config.default.target.clone()
}

pub fn resolve_simple_output(config: &FlomConfig) -> Option<bool> {
    if let Ok(value) = env::var("FLOM_OUTPUT_SIMPLE") {
        let normalized = value.to_lowercase();
        return Some(normalized == "1" || normalized == "true" || normalized == "yes");
    }
    config.output.simple
}
