#[cfg(feature = "webhook")]
use std::collections::HashSet;

use config::{Config as ConfigLoader, Environment, File};
use serde::Deserialize;
#[cfg(feature = "webhook")]
use tokio::sync::RwLock;

use crate::errors::Result;

/// Application configuration structure
///
/// This struct holds all configuration values needed by the application,
/// loaded from environment variables and configuration files.
///
/// Configuration sources are loaded in the following order (later sources override earlier ones):
/// 1. config.toml file (if present)
/// 2. Environment variables
///
/// # Fields
///
/// * `database_url` - PostgreSQL database connection string
/// * `meilisearch_url` - URL of the Meilisearch instance
/// * `meili_master_key` - Master key for Meilisearch authentication
/// * `jwt_secret` - Secret key for JWT token generation and validation
/// * `host` - Optional server host address (defaults to 0.0.0.0 if not set)
/// * `port` - Optional server port number (defaults to 8124 if not set)
/// * `github_webhook_secret` - Secret for validating GitHub webhook signatures (webhook feature only)
/// * `allowed_repositories` - Set of repository names allowed to trigger webhooks (webhook feature only)
/// * `github_token` - GitHub personal access token for API access (webhook feature only)
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// PostgreSQL database connection URL
    /// Format: postgres://username:password@host:port/database
    pub database_url: String,

    /// Meilisearch server URL
    /// Example: http://localhost:7700
    pub meilisearch_url: String,

    /// Meilisearch master key for administrative operations
    pub meili_master_key: String,

    /// Secret key for JWT token signing and verification
    pub jwt_secret: String,

    /// Optional server host address
    /// If not provided, defaults to 0.0.0.0 (listens on all interfaces)
    pub host: Option<String>,

    /// Optional server port number
    /// If not provided, defaults to 8124
    pub port: Option<u16>,

    /// GitHub webhook secret for signature verification
    /// Only available when the "webhook" feature is enabled
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,

    /// Set of repository names (format: "owner/repo") allowed to trigger webhooks
    /// Only available when the "webhook" feature is enabled
    #[cfg(feature = "webhook")]
    #[serde(default)]
    pub allowed_repositories: HashSet<String>,

    /// GitHub personal access token for API operations
    /// Only available when the "webhook" feature is enabled
    #[cfg(feature = "webhook")]
    pub github_token: String,
}

impl Config {
    /// Load configuration from environment variables and config file
    ///
    /// This function creates a new Config instance by loading values from:
    /// 1. config.toml file (optional, will not fail if missing)
    /// 2. Environment variables (takes precedence over config file)
    ///
    /// Environment variables should be in uppercase and prefixed with nothing.
    /// Example: DATABASE_URL, MEILISEARCH_URL, JWT_SECRET
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The loaded configuration or an error if required values are missing
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Required configuration values are missing
    /// * Configuration values have invalid format
    /// * Unable to deserialize configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::config::Config;
    ///
    /// let config = Config::new().expect("Failed to load configuration");
    /// println!("Database URL: {}", config.database_url);
    /// ```
    pub fn new() -> Result<Self> {
        // Build configuration from multiple sources
        let builder = ConfigLoader::builder()
            // Add config.toml file as a source (optional, won't fail if missing)
            .add_source(File::with_name("config.toml").required(false))
            // Add environment variables as a source (takes precedence)
            .add_source(Environment::default());

        // Build the configuration
        let settings = builder.build()?;

        // Deserialize into the Config struct
        let config = settings.try_deserialize()?;

        Ok(config)
    }
}

/// Runtime application configuration
///
/// This struct holds configuration values that may need to be updated at runtime.
/// Unlike `Config`, which is loaded once at startup, `AppConfig` can be modified
/// during application execution (particularly for webhook-related settings).
///
/// Fields that may change at runtime (like `allowed_repositories`) are wrapped
/// in `RwLock` to allow safe concurrent access and modification.
pub struct AppConfig {
    /// JWT secret key for token operations
    pub jwt_secret: String,

    /// GitHub webhook secret for signature verification
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,

    /// GitHub API access token
    #[cfg(feature = "webhook")]
    pub github_token: String,

    /// Set of allowed repository names, wrapped in RwLock for runtime updates
    /// This allows the list to be reloaded from config file without restarting the server
    #[cfg(feature = "webhook")]
    pub allowed_repositories: RwLock<HashSet<String>>,
}

impl AppConfig {
    /// Create a new AppConfig instance
    ///
    /// # Arguments
    ///
    /// * `jwt_secret` - Secret key for JWT operations
    /// * `github_webhook_secret` - GitHub webhook verification secret (webhook feature only)
    /// * `github_token` - GitHub API access token (webhook feature only)
    /// * `allowed_repositories` - Initial set of allowed repositories (webhook feature only)
    ///
    /// # Returns
    ///
    /// A new AppConfig instance with all values initialized
    ///
    /// # Example
    ///
    /// ```rust
    /// use backend::config::AppConfig;
    /// use std::collections::HashSet;
    ///
    /// let config = AppConfig::new(
    ///     "my_jwt_secret",
    ///     #[cfg(feature = "webhook")]
    ///     "my_webhook_secret",
    ///     #[cfg(feature = "webhook")]
    ///     "ghp_token",
    ///     #[cfg(feature = "webhook")]
    ///     HashSet::new()
    /// );
    /// ```
    pub fn new(
        jwt_secret: &str,
        #[cfg(feature = "webhook")] github_webhook_secret: &str,
        #[cfg(feature = "webhook")] github_token: &str,
        #[cfg(feature = "webhook")] allowed_repositories: HashSet<String>,
    ) -> Self {
        Self {
            jwt_secret: jwt_secret.to_string(),
            #[cfg(feature = "webhook")]
            github_webhook_secret: github_webhook_secret.to_string(),
            #[cfg(feature = "webhook")]
            github_token: github_token.to_string(),
            #[cfg(feature = "webhook")]
            allowed_repositories: RwLock::new(allowed_repositories),
        }
    }
}
