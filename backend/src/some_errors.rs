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
    #[error("Unsupported webhook event")]
    UnsupportedWebhookEvent,
}

#[cfg(feature = "webhook")]
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error(transparent)]
    DecodeBase64(#[from] base64::DecodeError),
    #[error(transparent)]
    DecodeUtf8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),
    #[cfg(feature = "webhook")]
    #[error(transparent)]
    ArticleParseError(#[from] gray_matter::Error),
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
    Parse(#[from] ParseError),
    #[cfg(feature = "webhook")]
    #[error(transparent)]
    Decode(#[from] DecodeError),
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

#[cfg(feature = "webhook")]
impl From<octocrab::Error> for SomeError {
    fn from(value: octocrab::Error) -> Self {
        SomeError::Other(value.into())
    }
}

#[cfg(feature = "webhook")]
impl From<base64::DecodeError> for SomeError {
    fn from(value: base64::DecodeError) -> Self {
        SomeError::Decode(value.into())
    }
}

#[cfg(feature = "webhook")]
impl From<std::string::FromUtf8Error> for SomeError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        SomeError::Decode(value.into())
    }
}

impl From<serde_json::Error> for SomeError {
    fn from(value: serde_json::Error) -> Self {
        SomeError::Parse(value.into())
    }
}

#[cfg(feature = "webhook")]
impl From<gray_matter::Error> for SomeError {
    fn from(value: gray_matter::Error) -> Self {
        SomeError::Parse(value.into())
    }
}

// 为 SomeError 实现 axum 的 IntoResponse trait。
// 这是将自定义错误类型与 axum Web 框架集成的关键。
// 当处理器函数返回 Result<_, SomeError> 并且是 Err 时，axum 会调用这个方法
// 将我们的自定义错误转换为一个标准的 HTTP 响应。
impl IntoResponse for SomeError {
    fn into_response(self) -> axum::response::Response {
        // 根据具体的错误变体，匹配出对应的 HTTP 状态码、错误代码和返回给用户的友好错误信息。
        // 这样做可以避免向客户端泄露敏感的内部实现细节。
        // 对于 5xx 错误（服务器内部问题），只返回通用消息，详细信息仅记录在日志中。
        let (status, error_code, user_message): (StatusCode, &str, &str) = match &self {
            // 服务器配置错误 (5xx) - 不暴露具体配置细节
            SomeError::Search(SearchError::MeilisearchUrlMissing)
            | SomeError::Search(SearchError::MasterKeyMissing)
            | SomeError::Search(SearchError::DefaultSearchApiKeyNotFound)
            | SomeError::Search(SearchError::DefaultAdminApiKeyNotFound)
            | SomeError::Search(SearchError::CustomApiKeyNotFound(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            ),

            SomeError::Database(DBError::DatabaseUrlMissing) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            ),

            SomeError::WebHooks(WebHooksError::GithubWebhookSecretMissing) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            ),

            // Meilisearch 服务本身的问题
            SomeError::Meilisearch(_) => (
                StatusCode::BAD_GATEWAY,
                "SERVICE_UNAVAILABLE",
                "Search service temporarily unavailable",
            ),

            // 数据库查询失败
            SomeError::Database(DBError::QueryFailed(_)) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "Service temporarily unavailable",
            ),

            // 来自客户端的无效请求 (4xx) - 可以提供适度详细的信息
            SomeError::GetPosts(GetPostsError::CategoryError) => (
                StatusCode::BAD_REQUEST,
                "INVALID_CATEGORY",
                "Invalid category parameter",
            ),

            SomeError::Parse(ParseError::JsonParseError(_)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_REQUEST",
                "Invalid request format",
            ),

            #[cfg(feature = "webhook")]
            SomeError::Parse(ParseError::ArticleParseError(_)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_CONTENT",
                "Invalid content format",
            ),

            SomeError::WebHooks(WebHooksError::MissingHeader(_)) => (
                StatusCode::BAD_REQUEST,
                "MISSING_HEADER",
                "Missing required header",
            ),

            SomeError::WebHooks(WebHooksError::InvalidHeader(_)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_HEADER",
                "Invalid header value",
            ),

            SomeError::WebHooks(WebHooksError::UnsupportedWebhookEvent) => (
                StatusCode::BAD_REQUEST,
                "UNSUPPORTED_EVENT",
                "Unsupported event type",
            ),

            SomeError::GetPosts(GetPostsError::ArticleNotFound) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found")
            }

            SomeError::WebHooks(WebHooksError::VerifySignatureFailed) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Unauthorized")
            }

            SomeError::WebHooks(WebHooksError::MissingRepositoryName) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_PAYLOAD",
                "Invalid webhook payload",
            ),

            #[cfg(feature = "webhook")]
            SomeError::Decode(DecodeError::DecodeBase64(_)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_ENCODING",
                "Invalid encoding",
            ),

            #[cfg(feature = "webhook")]
            SomeError::Decode(DecodeError::DecodeUtf8(_)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_ENCODING",
                "Invalid encoding",
            ),

            // 所有其他未预料到的内部错误
            SomeError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
            ),
        };

        // 根据状态码决定日志级别
        // 4xx 错误通常是客户端问题，使用 warn 级别
        // 5xx 错误是服务器问题，使用 error 级别
        if status.is_client_error() {
            tracing::warn!(
                status = status.as_u16(),
                error_code = error_code,
                error = %self,
                "Client error"
            );
        } else if status.is_server_error() {
            tracing::error!(
                status = status.as_u16(),
                error_code = error_code,
                error = %self,
                "Server error"
            );
        }

        // 构建返回给客户端的 JSON body，包含错误代码以便客户端识别错误类型
        let body = Json(json!({
            "error": user_message,
            "error_code": error_code,
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
