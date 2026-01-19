use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_auth_methods")]
    pub methods: Vec<String>,
    #[serde(default)]
    pub users: Vec<UserCredential>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserCredential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    #[serde(default)]
    pub worker_threads: usize,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_true")]
    pub tcp_nodelay: bool,
    #[serde(default = "default_true")]
    pub tcp_keepalive: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    #[serde(default = "default_max_connections_per_sec")]
    pub max_connections_per_sec: u32,
    #[serde(default)]
    pub max_bandwidth_per_connection: u64,
}

// Default values
fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    1080
}

fn default_max_connections() -> usize {
    10000
}

fn default_connection_timeout() -> u64 {
    300
}

fn default_auth_methods() -> Vec<String> {
    vec!["none".to_string()]
}

fn default_buffer_size() -> usize {
    8192
}

fn default_true() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_max_connections_per_sec() -> u32 {
    100
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            performance: PerformanceConfig::default(),
            logging: LoggingConfig::default(),
            limits: LimitsConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            port: default_port(),
            max_connections: default_max_connections(),
            connection_timeout_secs: default_connection_timeout(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            methods: default_auth_methods(),
            users: Vec::new(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0,
            buffer_size: default_buffer_size(),
            tcp_nodelay: default_true(),
            tcp_keepalive: default_true(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_connections_per_sec: default_max_connections_per_sec(),
            max_bandwidth_per_connection: 0,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.bind_address, "0.0.0.0");
        assert_eq!(config.server.port, 1080);
        assert_eq!(config.server.max_connections, 10000);
        assert!(!config.auth.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.server.port, deserialized.server.port);
        assert_eq!(config.server.bind_address, deserialized.server.bind_address);
    }

    #[test]
    fn test_user_credential() {
        let user = UserCredential {
            username: "test".to_string(),
            password: "pass123".to_string(),
        };

        assert_eq!(user.username, "test");
        assert_eq!(user.password, "pass123");
    }

    #[test]
    fn test_auth_config() {
        let mut config = AuthConfig::default();
        assert!(!config.enabled);

        config.enabled = true;
        config.users.push(UserCredential {
            username: "admin".to_string(),
            password: "secret".to_string(),
        });

        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].username, "admin");
    }

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert_eq!(config.buffer_size, 8192);
        assert!(config.tcp_nodelay);
        assert!(config.tcp_keepalive);
    }

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.port, 1080);
        assert_eq!(config.max_connections, 10000);
        assert_eq!(config.connection_timeout_secs, 300);
    }

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "pretty");
    }

    #[test]
    fn test_limits_config_defaults() {
        let config = LimitsConfig::default();
        assert_eq!(config.max_connections_per_sec, 100);
        assert_eq!(config.max_bandwidth_per_connection, 0);
    }
}
