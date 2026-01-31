use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    #[serde(default = "default_max_retention_seconds")]
    pub max_retention_seconds: i64,

    #[serde(default = "default_cleanup_interval_seconds")]
    pub cleanup_interval_seconds: u64,

    #[serde(default = "default_max_logs")]
    pub max_logs: Option<i64>,

    #[serde(default = "default_max_spans")]
    pub max_spans: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            db_path: default_db_path(),
            max_retention_seconds: default_max_retention_seconds(),
            cleanup_interval_seconds: default_cleanup_interval_seconds(),
            max_logs: default_max_logs(),
            max_spans: default_max_spans(),
        }
    }
}

fn default_listen_addr() -> String {
    "0.0.0.0:3333".to_string()
}

fn default_db_path() -> PathBuf {
    PathBuf::from("arboretum.db")
}

fn default_max_retention_seconds() -> i64 {
    7200 // 2 hours
}

fn default_cleanup_interval_seconds() -> u64 {
    300 // 5 minutes
}

fn default_max_logs() -> Option<i64> {
    None
}

fn default_max_spans() -> Option<i64> {
    None
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration from environment variables (fallback/override)
    pub fn from_env() -> Self {
        let listen_addr =
            std::env::var("ARBORETUM_LISTEN_ADDR").unwrap_or_else(|_| default_listen_addr());

        let db_path = std::env::var("ARBORETUM_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_db_path());

        let max_retention_seconds = std::env::var("ARBORETUM_MAX_RETENTION_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_max_retention_seconds);

        let cleanup_interval_seconds = std::env::var("ARBORETUM_CLEANUP_INTERVAL_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_cleanup_interval_seconds);

        let max_logs = std::env::var("ARBORETUM_MAX_LOGS")
            .ok()
            .and_then(|s| s.parse().ok());

        let max_spans = std::env::var("ARBORETUM_MAX_SPANS")
            .ok()
            .and_then(|s| s.parse().ok());

        Self {
            listen_addr,
            db_path,
            max_retention_seconds,
            cleanup_interval_seconds,
            max_logs,
            max_spans,
        }
    }

    /// Merge with environment variable overrides
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(addr) = std::env::var("ARBORETUM_LISTEN_ADDR") {
            self.listen_addr = addr;
        }

        if let Ok(path) = std::env::var("ARBORETUM_DB_PATH") {
            self.db_path = PathBuf::from(path);
        }

        if let Ok(val) = std::env::var("ARBORETUM_MAX_RETENTION_SECONDS") {
            if let Ok(parsed) = val.parse() {
                self.max_retention_seconds = parsed;
            }
        }

        if let Ok(val) = std::env::var("ARBORETUM_CLEANUP_INTERVAL_SECONDS") {
            if let Ok(parsed) = val.parse() {
                self.cleanup_interval_seconds = parsed;
            }
        }

        if let Ok(val) = std::env::var("ARBORETUM_MAX_LOGS") {
            self.max_logs = val.parse().ok();
        }

        if let Ok(val) = std::env::var("ARBORETUM_MAX_SPANS") {
            self.max_spans = val.parse().ok();
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.listen_addr, "0.0.0.0:3333");
        assert_eq!(config.max_retention_seconds, 7200);
        assert_eq!(config.cleanup_interval_seconds, 300);
    }

    #[test]
    fn test_from_env() {
        unsafe {
            std::env::set_var("ARBORETUM_LISTEN_ADDR", "127.0.0.1:8080");
            std::env::set_var("ARBORETUM_MAX_RETENTION_SECONDS", "3600");
        }

        let config = Config::from_env();
        assert_eq!(config.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.max_retention_seconds, 3600);

        unsafe {
            std::env::remove_var("ARBORETUM_LISTEN_ADDR");
            std::env::remove_var("ARBORETUM_MAX_RETENTION_SECONDS");
        }
    }
}
