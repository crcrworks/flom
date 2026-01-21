mod config;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::FlomConfig;
use flom_core::{FlomError, FlomResult};

pub use config::{ApiConfig, DefaultConfig, FlomConfig as FlomConfigData, OutputConfig};

#[cfg(test)]
pub(crate) static TEST_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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

pub fn config_exists() -> FlomResult<bool> {
    let path = config_path()?;
    Ok(path.exists())
}

pub fn resolve_odesli_key(config: &FlomConfig) -> Option<String> {
    if let Ok(value) = env::var("FLOM_ODESLI_KEY")
        && !value.trim().is_empty() {
            return Some(value);
        }
    config.api.odesli_key.clone()
}

pub fn resolve_default_target(config: &FlomConfig) -> Option<String> {
    if let Ok(value) = env::var("FLOM_DEFAULT_TARGET")
        && !value.trim().is_empty() {
            return Some(value);
        }
    config.default.target.clone()
}

pub fn resolve_user_country(config: &FlomConfig) -> String {
    if let Ok(value) = env::var("FLOM_USER_COUNTRY") {
        let normalized = value.trim();
        if !normalized.is_empty() {
            return normalized.to_string();
        }
    }
    config
        .default
        .user_country
        .clone()
        .unwrap_or_else(|| "US".to_string())
}

pub fn resolve_simple_output(config: &FlomConfig) -> Option<bool> {
    if let Ok(value) = env::var("FLOM_OUTPUT_SIMPLE") {
        let normalized = value.to_lowercase();
        return Some(normalized == "1" || normalized == "true" || normalized == "yes");
    }
    config.output.simple
}

pub fn set_config_value(key_path: &str, value: &str) -> FlomResult<()> {
    let path = config_path()?;
    let content = if path.exists() {
        fs::read_to_string(&path)
            .map_err(|err| FlomError::Config(format!("failed to read config: {err}")))?
    } else {
        String::new()
    };

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .unwrap_or_default();

    let parts: Vec<&str> = key_path.split('.').collect();
    if parts.len() < 2 {
        return Err(FlomError::Config(
            "key path must have at least 2 parts (e.g., 'api.odesli_key')".to_string(),
        ));
    }

    let table = doc.as_table_mut();
    let mut current = table;
    for part in &parts[..parts.len() - 1] {
        current = current
            .entry(part)
            .or_insert(toml_edit::Item::Table(Default::default()))
            .as_table_mut()
            .ok_or_else(|| {
                FlomError::Config(format!("cannot set nested value in '{}'", key_path))
            })?;
    }

    let last_part = parts.last().unwrap();
    current[last_part] = toml_edit::value(value);

    let content = doc.to_string();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| FlomError::Config(format!("failed to create config dir: {err}")))?;
    }
    fs::write(&path, content)
        .map_err(|err| FlomError::Config(format!("failed to write config: {err}")))?;

    Ok(())
}

pub fn open_in_editor() -> FlomResult<()> {
    let path = config_path()?;
    if !path.exists() {
        save_config(&FlomConfig::default())?;
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "vim".to_string()
        } else if cfg!(target_os = "windows") {
            "notepad".to_string()
        } else {
            "nano".to_string()
        }
    });

    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|err| FlomError::Config(format!("failed to open editor '{}': {}", editor, err)))?;

    if !status.success() {
        return Err(FlomError::Config(format!(
            "editor exited with status: {}",
            status
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_valid() {
        let toml_content = r#"
            [api]
            odesli_key = "test-key"

            [default]
            target = "spotify"
            user_country = "US"

            [output]
            simple = false
        "#;
        let config = toml::from_str::<FlomConfig>(toml_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.api.odesli_key, Some("test-key".to_string()));
        assert_eq!(config.default.target, Some("spotify".to_string()));
        assert_eq!(config.default.user_country, Some("US".to_string()));
        assert_eq!(config.output.simple, Some(false));
    }

    #[test]
    fn test_config_load_invalid() {
        let invalid_toml = "invalid [toml content";
        let result = toml::from_str::<FlomConfig>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_default_target_env() {
        let _lock = TEST_ENV_MUTEX.lock().unwrap();
        let config = FlomConfig::default();
        unsafe {
            env::set_var("FLOM_DEFAULT_TARGET", "spotify");
        }
        let result = resolve_default_target(&config);
        assert_eq!(result, Some("spotify".to_string()));
        unsafe {
            env::remove_var("FLOM_DEFAULT_TARGET");
        }
    }

    #[test]
    fn test_resolve_user_country_env() {
        let _lock = TEST_ENV_MUTEX.lock().unwrap();
        let config = FlomConfig::default();
        unsafe {
            env::set_var("FLOM_USER_COUNTRY", "JP");
        }
        let result = resolve_user_country(&config);
        assert_eq!(result, "JP");
        unsafe {
            env::remove_var("FLOM_USER_COUNTRY");
        }
    }

    #[test]
    fn test_resolve_user_country_default() {
        let config = FlomConfig::default();
        let result = resolve_user_country(&config);
        assert_eq!(result, "US");
    }
}
