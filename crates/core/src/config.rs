//! Process config loaded from `RWK_` environment variables.

use std::net::SocketAddr;

use figment::Figment;
use figment::providers::{Env, Serialized};
use garde::Validate;
use serde::{Deserialize, Serialize};

/// Default session secret used in development. Production must override this.
pub const DEFAULT_SESSION_SECRET: &str = "dev-only-change-me-please";

/// Runtime environment selected by `RWK_ENV`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Test,
    Production,
}

/// Global application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[garde(custom(reject_default_secret_in_production))]
pub struct Config {
    #[garde(skip)]
    pub env: Environment,
    #[garde(dive)]
    pub server: ServerConfig,
    #[garde(dive)]
    pub database: DatabaseConfig,
    #[garde(dive)]
    pub session: SessionConfig,
    #[garde(dive)]
    pub mail: MailConfig,
    #[garde(dive)]
    pub oauth: OauthConfig,
    #[garde(url)]
    pub app_url: String,
    #[garde(skip)]
    pub log_format: LogFormat,
    #[garde(skip)]
    pub run_migrations: bool,
}

/// HTTP bind address and optional SPA static directory.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[garde(length(min = 1))]
    pub host: String,
    #[garde(range(min = 1))]
    pub port: u16,
    /// Directory of the Vite `dist` output. Served with SPA fallback.
    #[garde(length(min = 1))]
    pub static_dir: String,
}

/// Postgres connection.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    #[garde(length(min = 1))]
    pub url: String,
}

/// Cookie session settings (wired in the auth phase).
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SessionConfig {
    #[garde(length(min = 16))]
    pub secret: String,
    #[garde(length(min = 1))]
    pub cookie_name: String,
}

/// SMTP settings (wired in the mail phase).
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MailConfig {
    #[garde(length(min = 3))]
    pub from: String,
    #[garde(length(min = 1))]
    pub smtp_host: String,
    #[garde(range(min = 1))]
    pub smtp_port: u16,
    #[garde(skip)]
    pub smtp_username: Option<String>,
    #[garde(skip)]
    pub smtp_password: Option<String>,
}

/// OAuth client settings (wired in the OAuth phase). Empty disables a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct OauthConfig {
    #[garde(skip)]
    pub google_client_id: Option<String>,
    #[garde(skip)]
    pub google_client_secret: Option<String>,
    #[garde(skip)]
    pub github_client_id: Option<String>,
    #[garde(skip)]
    pub github_client_secret: Option<String>,
}

/// Tracing formatter selected by `RWK_LOG_FORMAT`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

/// Failure loading or validating [`Config`].
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Figment(Box<figment::Error>),
    #[error("invalid config: {0}")]
    Validation(garde::Report),
}

impl From<figment::Error> for ConfigError {
    fn from(error: figment::Error) -> Self {
        Self::Figment(Box::new(error))
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // garde custom validators take `&()`.
fn reject_default_secret_in_production(config: &Config, _: &()) -> garde::Result {
    if config.env == Environment::Production && config.session.secret == DEFAULT_SESSION_SECRET {
        return Err(garde::Error::new(
            "session.secret must not be the default development value when env is production",
        ));
    }
    Ok(())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            env: Environment::Development,
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
                static_dir: "apps/web/dist".into(),
            },
            database: DatabaseConfig {
                url: "postgres://rwk:rwk@localhost:5432/rwk".into(),
            },
            session: SessionConfig {
                secret: DEFAULT_SESSION_SECRET.into(),
                cookie_name: "rwk_session".into(),
            },
            mail: MailConfig {
                from: "noreply@localhost".into(),
                smtp_host: "localhost".into(),
                smtp_port: 1025,
                smtp_username: None,
                smtp_password: None,
            },
            oauth: OauthConfig::default(),
            app_url: "http://localhost:8080".into(),
            log_format: LogFormat::Pretty,
            run_migrations: true,
        }
    }
}

impl Config {
    /// Load from process env (`RWK_` prefix, `__` nested keys) over defaults.
    pub fn from_env() -> Result<Self, ConfigError> {
        let figment = Figment::from(Serialized::defaults(Self::default()))
            .merge(Env::prefixed("RWK_").split("__"));
        Self::from_figment(&figment)
    }

    /// Extract and validate from an arbitrary figment.
    pub fn from_figment(figment: &Figment) -> Result<Self, ConfigError> {
        let mut config: Self = figment.extract()?;
        config.normalize();
        config.validate().map_err(ConfigError::Validation)?;
        Ok(config)
    }

    /// Deterministic config for tests. Reads `DATABASE_URL` when set.
    pub fn for_tests() -> Self {
        let mut config = Self {
            env: Environment::Test,
            server: ServerConfig {
                host: "127.0.0.1".into(),
                ..Self::default().server
            },
            session: SessionConfig {
                secret: "test-session-secret-not-for-production".into(),
                cookie_name: "rwk_session".into(),
            },
            ..Self::default()
        };
        if let Ok(url) = std::env::var("DATABASE_URL") {
            config.database.url = url;
        }
        config
    }

    /// Bind address for the HTTP server.
    pub fn listen_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.server.host, self.server.port).parse()
    }

    fn normalize(&mut self) {
        blank_to_none(&mut self.mail.smtp_username);
        blank_to_none(&mut self.mail.smtp_password);
        blank_to_none(&mut self.oauth.google_client_id);
        blank_to_none(&mut self.oauth.google_client_secret);
        blank_to_none(&mut self.oauth.github_client_id);
        blank_to_none(&mut self.oauth.github_client_secret);
    }
}

fn blank_to_none(value: &mut Option<String>) {
    if value.as_ref().is_some_and(String::is_empty) {
        *value = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use figment::providers::Serialized;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn extract_overlay(overlay: Config) -> Result<Config, ConfigError> {
        let figment = Figment::from(Serialized::defaults(Config::default()))
            .merge(Serialized::defaults(overlay));
        Config::from_figment(&figment)
    }

    #[test]
    fn default_config_validates() {
        let figment = Figment::from(Serialized::defaults(Config::default()));
        Config::from_figment(&figment).unwrap();
    }

    #[test]
    fn for_tests_is_valid() {
        let config = Config::for_tests();
        config.validate().unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.env, Environment::Test);
    }

    #[test]
    fn production_rejects_default_session_secret() {
        let overlay = Config {
            env: Environment::Production,
            ..Config::default()
        };
        let err = extract_overlay(overlay).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn production_accepts_non_default_session_secret() {
        let overlay = Config {
            env: Environment::Production,
            session: SessionConfig {
                secret: "a-sufficiently-long-production-secret".into(),
                cookie_name: "rwk_session".into(),
            },
            ..Config::default()
        };
        let loaded = extract_overlay(overlay).unwrap();
        assert_eq!(loaded.env, Environment::Production);
    }

    #[test]
    fn rejects_port_zero() {
        let overlay = Config {
            server: ServerConfig {
                port: 0,
                ..Config::default().server
            },
            ..Config::default()
        };
        let err = extract_overlay(overlay).unwrap_err();
        match err {
            ConfigError::Validation(report) => {
                let msg = report.to_string();
                assert!(msg.contains("server.port") || msg.contains("port"), "{msg}");
            }
            ConfigError::Figment(error) => panic!("expected validation, got {error:?}"),
        }
    }

    #[test]
    fn rejects_invalid_app_url() {
        let overlay = Config {
            app_url: "not-a-url".into(),
            ..Config::default()
        };
        let err = extract_overlay(overlay).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn rejects_short_session_secret() {
        let overlay = Config {
            session: SessionConfig {
                secret: "short".into(),
                cookie_name: "rwk_session".into(),
            },
            ..Config::default()
        };
        let err = extract_overlay(overlay).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn from_env_loads_and_rejects() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        // SAFETY: serialized by ENV_LOCK; restored before the guard drops.
        unsafe {
            std::env::set_var("RWK_SERVER__PORT", "9999");
            std::env::set_var("RWK_LOG_FORMAT", "json");
            std::env::set_var("RWK_APP_URL", "http://example.test");
        }
        let loaded = Config::from_env().unwrap();
        assert_eq!(loaded.server.port, 9999);
        assert_eq!(loaded.log_format, LogFormat::Json);
        assert_eq!(loaded.app_url, "http://example.test");

        unsafe {
            std::env::set_var("RWK_SERVER__PORT", "0");
        }
        let err = Config::from_env().unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));

        unsafe {
            std::env::remove_var("RWK_SERVER__PORT");
            std::env::remove_var("RWK_LOG_FORMAT");
            std::env::remove_var("RWK_APP_URL");
        }
    }
}
