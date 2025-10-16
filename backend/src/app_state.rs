use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "webhook")]
use std::collections::HashSet;
use std::time::Duration;
#[cfg(feature = "webhook")]
use tokio::sync::RwLock;

use crate::config::Config;
use crate::errors::Result;
use crate::infrastructure::search::index::SearchService;

const DEFAULT_MAX_CONNECTIONS: u32 = 50; // 默认的最大连接数
const DEFAULT_INDEX_NAME: &str = "articles"; // 默认的搜索索引名称

// 定义应用的状态结构体 AppState，它将在整个应用中共享
pub struct AppState {
    // 数据库连接池
    pub db_pool: PgPool,
    // 用于 JWT (JSON Web Token) 的密钥
    pub jwt_secret: String,
    // 搜索服务
    pub search_service: SearchService,
    #[cfg(feature = "webhook")]
    pub github_webhook_secret: String,
    #[cfg(feature = "webhook")]
    pub allowed_repositories: RwLock<HashSet<String>>,
    #[cfg(feature = "webhook")]
    pub github_token: String,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        // 异步初始化搜索服务。
        let search_service = SearchService::new(&config, DEFAULT_INDEX_NAME).await?;
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
        let allowed_repositories = RwLock::new(config.allowed_repositories);

        #[cfg(feature = "webhook")]
        let github_token = config.github_token;

        // 创建应用共享状态 AppState 的实例。
        let state = AppState {
            db_pool: pool,                  // 数据库连接池
            jwt_secret: jwt_secret,         // JWT 密钥
            search_service: search_service, // 搜索服务
            #[cfg(feature = "webhook")]
            github_webhook_secret,
            #[cfg(feature = "webhook")]
            allowed_repositories,
            #[cfg(feature = "webhook")]
            github_token,
        };

        Ok(state)
    }
}
