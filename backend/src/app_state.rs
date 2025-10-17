use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;

use crate::application::article_service::ArticleService;
use crate::config::{AppConfig, Config};
use crate::errors::Result;
use crate::infrastructure::db::sqlx_repo::SqlxArticleRepository;
#[cfg(feature = "webhook")]
use crate::infrastructure::github::api_client::GithubApiClient;
use crate::infrastructure::search::index::MeiliSearchService;

/// Default maximum number of database connections in the pool
const DEFAULT_MAX_CONNECTIONS: u32 = 50;

/// Default name for the search index in Meilisearch
const DEFAULT_INDEX_NAME: &str = "articles";

/// Application state structure that holds all shared resources
///
/// This structure is shared across all request handlers and contains
/// references to services and configuration needed throughout the application.
/// It's wrapped in Arc to enable safe sharing across async tasks.
pub struct AppState {
    /// Service layer for article-related business logic
    pub article_service: Arc<ArticleService>,

    /// Application configuration including secrets and settings
    pub app_config: Arc<AppConfig>,
}

impl AppState {
    /// Create a new AppState instance with all initialized dependencies
    ///
    /// This async function performs the following initialization steps:
    /// 1. Initialize the search service (Meilisearch)
    /// 2. Set up the database connection pool
    /// 3. Configure GitHub client (if webhook feature is enabled)
    /// 4. Create application configuration
    /// 5. Initialize the article service with all dependencies
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration loaded from environment variables and config files
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The initialized AppState or an error if initialization fails
    ///
    /// # Panics
    ///
    /// Panics if unable to connect to the database
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize the search service asynchronously
        // This connects to Meilisearch and verifies the connection
        let search_service = MeiliSearchService::new(&config, DEFAULT_INDEX_NAME).await?;
        tracing::info!("Search service initialized successfully.");

        // Extract JWT secret for later use in authentication
        let jwt_secret = config.jwt_secret;

        // Configure and create the database connection pool
        // The pool manages a set of database connections for efficient reuse
        let pool = PgPoolOptions::new()
            .max_connections(DEFAULT_MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(3)) // Timeout for acquiring a connection from the pool
            .connect(&config.database_url) // Establish connection to the database
            .await // Await the async connection operation
            .expect("Failed to connect to the database.");

        // Extract webhook-related configuration (only if webhook feature is enabled)
        #[cfg(feature = "webhook")]
        let github_webhook_secret = config.github_webhook_secret;

        #[cfg(feature = "webhook")]
        let allowed_repositories = config.allowed_repositories;

        #[cfg(feature = "webhook")]
        let github_token = config.github_token;

        // Initialize GitHub API client with the provided token
        #[cfg(feature = "webhook")]
        let github_client = GithubApiClient::new(&github_token)?;

        // Create application configuration wrapped in Arc for thread-safe sharing
        let app_config = Arc::new(AppConfig::new(
            &jwt_secret,
            #[cfg(feature = "webhook")]
            &github_webhook_secret,
            #[cfg(feature = "webhook")]
            &github_token,
            #[cfg(feature = "webhook")]
            allowed_repositories,
        ));

        // Initialize the article service with all required dependencies
        // This follows the dependency injection pattern for better testability
        let article_service = Arc::new(ArticleService::new(
            Arc::new(SqlxArticleRepository::new(pool)),
            #[cfg(feature = "webhook")]
            Arc::new(github_client),
            Arc::new(search_service),
            app_config.clone(),
        ));

        // Construct and return the AppState instance
        let state = Self {
            article_service,
            app_config,
        };

        Ok(state)
    }
}
