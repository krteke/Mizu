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

const DEFAULT_MAX_CONNECTIONS: u32 = 50; // 默认的最大连接数
const DEFAULT_INDEX_NAME: &str = "articles"; // 默认的搜索索引名称

// 定义应用的状态结构体 AppState，它将在整个应用中共享
pub struct AppState {
    pub article_service: Arc<ArticleService>,
    pub app_config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        // 异步初始化搜索服务。
        let search_service = MeiliSearchService::new(&config, DEFAULT_INDEX_NAME).await?;
        tracing::info!("Search service initialized successfully.");
        // JWT 密钥，用于后续的应用状态共享。
        let jwt_secret = config.jwt_secret;

        // 设置数据库连接池选项。
        let pool = PgPoolOptions::new()
            .max_connections(DEFAULT_MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(3)) // 获取连接的超时时间为 3 秒
            .connect(&config.database_url) // 连接到数据库
            .await // 等待连接完成
            .expect("无法连接到数据库。");

        #[cfg(feature = "webhook")]
        let github_webhook_secret = config.github_webhook_secret;

        #[cfg(feature = "webhook")]
        let allowed_repositories = config.allowed_repositories;

        #[cfg(feature = "webhook")]
        let github_token = config.github_token;

        #[cfg(feature = "webhook")]
        let github_client = GithubApiClient::new(&github_token)?;

        let app_config = Arc::new(AppConfig::new(
            &jwt_secret,
            #[cfg(feature = "webhook")]
            &github_webhook_secret,
            #[cfg(feature = "webhook")]
            &github_token,
            #[cfg(feature = "webhook")]
            allowed_repositories,
        ));

        let article_service = Arc::new(ArticleService::new(
            Arc::new(SqlxArticleRepository::new(pool)),
            #[cfg(feature = "webhook")]
            Arc::new(github_client),
            Arc::new(search_service),
            app_config.clone(),
        ));

        // 创建应用共享状态 AppState 的实例。
        let state = Self {
            article_service,
            app_config,
        };

        Ok(state)
    }
}
