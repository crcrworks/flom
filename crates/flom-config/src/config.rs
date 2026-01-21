use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    pub odesli_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultConfig {
    pub target: Option<String>,
    pub user_country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    pub simple: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlomConfig {
    pub api: ApiConfig,
    pub default: DefaultConfig,
    pub output: OutputConfig,
}

#[cfg(test)]
mod tests {
    use super::FlomConfig;
    use crate::{load_config, resolve_default_target, resolve_user_country};
    use flom_core::FlomError;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct EnvGuard {
        key: &'static str,
        prev: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let prev = env::var(key).ok();
            unsafe {
                env::set_var(key, value);
            }
            Self { key, prev }
        }

        fn remove(key: &'static str) -> Self {
            let prev = env::var(key).ok();
            unsafe {
                env::remove_var(key);
            }
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(prev) = &self.prev {
                unsafe {
                    env::set_var(self.key, prev);
                }
            } else {
                unsafe {
                    env::remove_var(self.key);
                }
            }
        }
    }

    fn temp_home_dir() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let mut dir = env::temp_dir();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        dir.push(format!("flom-test-{}", counter));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_config_load_valid() {
        let _lock = crate::TEST_ENV_MUTEX.lock().unwrap();

        let toml_content = r#"
            [api]
            odesli_key = "test-key"

            [default]
            target = "spotify"
            user_country = "US"

            [output]
            simple = false
        "#;
        let home_dir = temp_home_dir();
        let home_dir_string = home_dir.to_string_lossy().to_string();
        let _home_guard = EnvGuard::set("HOME", &home_dir_string);
        let config_dir = home_dir.join(".flom");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), toml_content).unwrap();

        let config = load_config().unwrap();
        assert_eq!(config.api.odesli_key, Some("test-key".to_string()));
        assert_eq!(config.default.target, Some("spotify".to_string()));
        assert_eq!(config.default.user_country, Some("US".to_string()));
        assert_eq!(config.output.simple, Some(false));

        fs::remove_dir_all(&home_dir).unwrap();
    }

    #[test]
    fn test_config_load_invalid() {
        let _lock = crate::TEST_ENV_MUTEX.lock().unwrap();

        let invalid_toml = "invalid [toml content";
        let home_dir = temp_home_dir();
        let home_dir_string = home_dir.to_string_lossy().to_string();
        let _home_guard = EnvGuard::set("HOME", &home_dir_string);
        let config_dir = home_dir.join(".flom");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), invalid_toml).unwrap();

        let result = load_config();
        match result {
            Err(FlomError::Config(msg)) => assert!(msg.contains("failed to parse config")),
            _ => panic!("Expected Config error"),
        }

        fs::remove_dir_all(&home_dir).unwrap();
    }

    #[test]
    fn test_resolve_default_target_env() {
        let _lock = crate::TEST_ENV_MUTEX.lock().unwrap();
        let mut config = FlomConfig::default();
        config.default.target = Some("itunes".to_string());
        let _guard = EnvGuard::set("FLOM_DEFAULT_TARGET", "spotify");
        let result = resolve_default_target(&config);
        assert_eq!(result, Some("spotify".to_string()));
    }

    #[test]
    fn test_resolve_user_country_env() {
        let _lock = crate::TEST_ENV_MUTEX.lock().unwrap();
        let mut config = FlomConfig::default();
        config.default.user_country = Some("DE".to_string());
        let _guard = EnvGuard::set("FLOM_USER_COUNTRY", "JP");
        let result = resolve_user_country(&config);
        assert_eq!(result, "JP");
    }

    #[test]
    fn test_resolve_user_country_default() {
        let _guard = EnvGuard::remove("FLOM_USER_COUNTRY");
        let config = FlomConfig::default();
        let result = resolve_user_country(&config);
        assert_eq!(result, "US");
    }
}
