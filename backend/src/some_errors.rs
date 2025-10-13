use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

// --- SearchError ---
// 定义与搜索服务相关的错误
#[derive(Debug, Error)]
pub enum SearchError {
    // Meilisearch URL 未在环境变量中设置
    #[error("Can not get MEILISEARCH_URL from env.")]
    MeilisearchUrlMissing,
    // Meilisearch 主密钥未在环境变量中设置
    #[error("Can not get MEILI_MASTER_KEY from env.")]
    MasterKeyMissing,
    // 找不到默认的搜索 API 密钥
    #[error("Can not found default search api key.")]
    DefaultSearchApiKeyNotFound,
    // 找不到默认的管理员 API 密钥
    #[error("Can not found default admin api key.")]
    DefaultAdminApiKeyNotFound,
    // 找不到指定名称的 API 密钥
    #[error("Can not found this key, it is {0}, is it valid?")]
    CustomApiKeyNotFound(String),
}

// --- DBError ---
// 定义与数据库操作相关的错误
#[derive(Debug, Error)]
pub enum DBError {
    // 数据库 URL 未在环境变量中设置
    #[error("Can not get DATABASE_URL from env.")]
    DatabaseUrlMissing,
    // 数据库查询失败，包装了来自 sqlx 的原始错误
    #[error("Database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

// --- GetPostsError ---
// 定义获取文章列表时的特定错误
#[derive(Debug, Error)]
pub enum GetPostsError {
    // 无效的文章分类
    #[error("Invalid Category type.")]
    CategoryError,
    // 文章未找到
    #[error("Article not found")]
    ArticleNotFound,
}

// --- WebHooksError ---
// 定义与 Webhook 相关的错误
#[derive(Debug, Error)]
pub enum WebHooksError {
    // Webhook 验证失败
    #[error("Github webhook verification failed")]
    VerifySignatureFailed,
    #[error("Invalid {0} header")]
    InvalidHeader(String),
    #[error("Missing {0} header")]
    MissingHeader(String),
    #[error("Could not extract repository name from webhook event")]
    MissingRepositoryName,
    #[error("Can not get GITHUB_WEBHOOK_SECRET from environment")]
    GithubWebhookSecretMissing,
}

// --- SomeError ---
// 定义一个顶层的、统一的错误枚举，它包含了项目中所有可能的业务错误。
// 使用 #[error(transparent)] 可以让错误信息直接显示来源错误的信息，而不是 "SomeError::Variant(source_error)"。
#[derive(Debug, Error)]
pub enum SomeError {
    // 包装 SearchError
    #[error(transparent)]
    Search(#[from] SearchError),
    // 包装 DBError
    #[error(transparent)]
    Database(#[from] DBError),
    // 包装来自 meilisearch_sdk 的错误
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
    // 包装 GetPostsError
    #[error(transparent)]
    GetPosts(#[from] GetPostsError),
    // 包装 WebHooksError
    #[error(transparent)]
    WebHooks(#[from] WebHooksError),
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    // 包装来自 anyhow 的通用错误，用于处理其他未明确分类的错误
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// 为 SomeError 实现 From<sqlx::Error> trait。
// 这样，当 sqlx::Error 出现时（例如使用 `?` 操作符），
// 它可以被自动转换为 SomeError::Database(DBError::QueryFailed(err))。
// 这比 thiserror 自动生成的 `#[from]` 更具体，避免了歧义。
impl From<sqlx::Error> for SomeError {
    fn from(err: sqlx::Error) -> Self {
        SomeError::Database(DBError::QueryFailed(err))
    }
}

impl From<config::ConfigError> for SomeError {
    fn from(error: config::ConfigError) -> Self {
        if let config::ConfigError::NotFound(field) = &error {
            match field.as_str() {
                "database_url" => SomeError::Database(DBError::DatabaseUrlMissing),
                "meilisearch_url" => SomeError::Search(SearchError::MeilisearchUrlMissing),
                "meili_master_key" => SomeError::Search(SearchError::MasterKeyMissing),
                "jwt_secret" => SomeError::Other(anyhow::Error::msg(
                    "Can not get JWT_SECRET from environment",
                )),
                "github_webhook_secret" => {
                    SomeError::WebHooks(WebHooksError::GithubWebhookSecretMissing)
                }
                _ => SomeError::Other(anyhow::anyhow!(error)),
            }
        } else {
            SomeError::from(anyhow::anyhow!(error))
        }
    }
}

// 为 SomeError 实现 axum 的 IntoResponse trait。
// 这是将自定义错误类型与 axum Web 框架集成的关键。
// 当处理器函数返回 Result<_, SomeError> 并且是 Err 时，axum 会调用这个方法
// 将我们的自定义错误转换为一个标准的 HTTP 响应。
impl IntoResponse for SomeError {
    fn into_response(self) -> axum::response::Response {
        // 使用 tracing 记录内部错误信息，这对于调试非常重要。
        // `%self` 会使用 `Display` trait 格式化错误，也就是 `#[error("...")]` 定义的内容。
        tracing::error!(error = %self,"Unhandled application error");

        // 根据具体的错误变体，匹配出对应的 HTTP 状态码和返回给用户的友好错误信息。
        // 这样做可以避免向客户端泄露敏感的内部实现细节。
        let (status, user_message) = match &self {
            // 服务器配置错误
            SomeError::Search(SearchError::MeilisearchUrlMissing)
            | SomeError::Search(SearchError::MasterKeyMissing)
            | SomeError::Database(DBError::DatabaseUrlMissing)
            | SomeError::WebHooks(WebHooksError::GithubWebhookSecretMissing) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Server misconfiguration")
            }

            // Meilisearch 服务本身的问题
            SomeError::Meilisearch(_) => (StatusCode::BAD_GATEWAY, "Search service unavailable"),

            // 搜索服务配置问题
            SomeError::Search(
                SearchError::DefaultSearchApiKeyNotFound
                | SearchError::DefaultAdminApiKeyNotFound
                | SearchError::CustomApiKeyNotFound(_),
            ) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Search service misconfigured",
            ),

            // 数据库查询失败
            SomeError::Database(DBError::QueryFailed(_)) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database temporarily unavailable",
            ),

            // 来自客户端的无效请求
            SomeError::GetPosts(GetPostsError::CategoryError)
            | SomeError::JsonParseError(_)
            | SomeError::WebHooks(WebHooksError::MissingHeader(_))
            | SomeError::WebHooks(WebHooksError::InvalidHeader(_)) => (
                StatusCode::BAD_REQUEST,
                "Something was invalid in requests.",
            ),

            SomeError::GetPosts(GetPostsError::ArticleNotFound) => (
                StatusCode::NOT_FOUND,
                "The article is not found in the database.",
            ),

            SomeError::WebHooks(WebHooksError::VerifySignatureFailed) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized")
            }

            SomeError::WebHooks(WebHooksError::MissingRepositoryName) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Missing repository name")
            }

            // 所有其他未预料到的内部错误
            SomeError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
            ),
        };

        // 构建返回给客户端的 JSON body
        let body = Json(json!({
            "error": user_message,
            "status": "error"
        }));

        // 将状态码和 JSON body 组合成一个完整的 HTTP 响应
        (status, body).into_response()
    }
}

// 定义一个全局的 Result 类型别名。
// 在整个项目中，我们可以使用 `Result<T>` 来代替 `std::result::Result<T, SomeError>`，
// 这样可以使代码更简洁。
pub type Result<T> = std::result::Result<T, SomeError>;
