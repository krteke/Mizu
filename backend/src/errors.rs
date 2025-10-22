use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

/// Errors related to search service operations
///
/// These errors occur when interacting with the Meilisearch service,
/// including configuration issues and missing API keys.
#[derive(Debug, Error)]
pub enum SearchError {
    /// Meilisearch URL is not set in environment variables
    #[error("Can not get MEILISEARCH_URL from env.")]
    MeilisearchUrlMissing,

    /// Meilisearch master key is not set in environment variables
    #[error("Can not get MEILI_MASTER_KEY from env.")]
    MasterKeyMissing,

    /// Default search API key was not found in Meilisearch
    #[error("Can not found default search api key.")]
    DefaultSearchApiKeyNotFound,

    /// Default admin API key was not found in Meilisearch
    #[error("Can not found default admin api key.")]
    DefaultAdminApiKeyNotFound,

    /// Custom API key with the specified name was not found
    #[error("Can not found this key, it is {0}, is it valid?")]
    CustomApiKeyNotFound(String),
}

/// Errors related to database operations
///
/// These errors occur during database interactions, including connection
/// failures and query execution errors.
#[derive(Debug, Error)]
pub enum DBError {
    /// Database URL is not set in environment variables
    #[error("Can not get DATABASE_URL from env.")]
    DatabaseUrlMissing,

    /// Database query failed, wrapping the original sqlx error
    #[error("Database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

/// Errors specific to article retrieval operations
///
/// These errors occur when fetching or querying articles from the database.
#[derive(Debug, Error)]
pub enum GetPostsError {
    /// Invalid article category was provided
    #[error("Invalid Category type.")]
    CategoryError,

    /// Requested article was not found in the database
    #[error("Article not found")]
    ArticleNotFound,
}

/// Errors related to GitHub webhook operations
///
/// These errors occur when processing incoming webhooks from GitHub,
/// including signature verification and payload parsing issues.
#[derive(Debug, Error)]
pub enum WebHooksError {
    /// GitHub webhook signature verification failed
    #[error("Github webhook verification failed")]
    VerifySignatureFailed,

    /// HTTP header contains invalid value
    #[error("Invalid {0} header")]
    InvalidHeader(String),

    /// Required HTTP header is missing
    #[error("Missing {0} header")]
    MissingHeader(String),

    /// Could not extract repository name from webhook payload
    #[error("Could not extract repository name from webhook event")]
    MissingRepositoryName,

    /// GitHub webhook secret is not set in environment variables
    #[error("Can not get GITHUB_WEBHOOK_SECRET from environment")]
    GithubWebhookSecretMissing,

    /// Webhook event type is not supported by this application
    #[error("Unsupported webhook event")]
    UnsupportedWebhookEvent,
}

/// Errors related to decoding operations (webhook feature only)
///
/// These errors occur when decoding base64 content or UTF-8 strings,
/// typically when processing file content from GitHub API responses.
#[cfg(feature = "webhook")]
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Base64 decoding failed
    #[error(transparent)]
    DecodeBase64(#[from] base64::DecodeError),

    /// UTF-8 string decoding failed
    #[error(transparent)]
    DecodeUtf8(#[from] std::string::FromUtf8Error),
}

/// Errors related to parsing operations
///
/// These errors occur when parsing JSON data or article front matter
/// from markdown files.
#[derive(Debug, Error)]
pub enum ParseError {
    /// JSON parsing failed
    #[error(transparent)]
    JsonParseError(#[from] serde_json::Error),

    /// Article front matter parsing failed (webhook feature only)
    #[cfg(feature = "webhook")]
    #[error(transparent)]
    ArticleParseError(#[from] gray_matter::Error),
}

/// Top-level unified error enum containing all possible application errors
///
/// This enum aggregates all specific error types into a single error type
/// that can be returned from any application function. Using `#[error(transparent)]`
/// allows the original error message to be displayed directly instead of
/// showing "SomeError::Variant(source_error)".
///
/// This follows the error handling pattern where domain-specific errors are
/// wrapped into a single application error type for easier propagation and handling.
#[derive(Debug, Error)]
pub enum SomeError {
    /// Search service related errors
    #[error(transparent)]
    Search(#[from] SearchError),

    /// Database operation errors
    #[error(transparent)]
    Database(#[from] DBError),

    /// Errors from the Meilisearch SDK
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
    /// Article retrieval errors
    #[error(transparent)]
    GetPosts(#[from] GetPostsError),

    /// GitHub webhook processing errors
    #[error(transparent)]
    WebHooks(#[from] WebHooksError),

    /// Parsing errors (JSON, YAML, etc.)
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// Decoding errors (base64, UTF-8)
    #[cfg(feature = "webhook")]
    #[error(transparent)]
    Decode(#[from] DecodeError),

    /// Generic errors from anyhow for cases not covered by specific error types
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Implement From<sqlx::Error> for SomeError to provide specific conversion
///
/// This implementation ensures that sqlx::Error is automatically converted to
/// SomeError::Database(DBError::QueryFailed(err)) when using the `?` operator.
/// This is more specific than the automatic conversion from thiserror's `#[from]`
/// attribute and avoids ambiguity in error handling.
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

/// Implement axum's IntoResponse trait for SomeError
///
/// This is the key integration point between our custom error type and the
/// axum web framework. When a handler function returns Result<_, SomeError>
/// and the result is Err, axum calls this method to convert the error into
/// a standard HTTP response.
///
/// The implementation maps each error variant to an appropriate HTTP status code
/// and user-friendly error message, ensuring that sensitive internal details
/// are not leaked to clients. For 5xx errors (server issues), only generic
/// messages are returned while details are logged.
impl IntoResponse for SomeError {
    fn into_response(self) -> axum::response::Response {
        // Match the error variant to determine HTTP status code, error code, and user message
        // This prevents leaking sensitive implementation details to clients
        let (status, error_code, user_message): (StatusCode, &str, &str) = match &self {
            // Server configuration errors (5xx) - don't expose specific configuration details
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
            SomeError::Decode(DecodeError::DecodeBase64(_))
            | SomeError::Decode(DecodeError::DecodeUtf8(_)) => (
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

        // Choose log level based on status code
        // 4xx errors are typically client issues, log at warn level
        // 5xx errors are server issues, log at error level
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

        // Construct JSON response body with error code for client-side error identification
        let body = Json(json!({
            "error": user_message,
            "error_code": error_code,
            "status": "error"
        }));

        // Combine status code and JSON body into a complete HTTP response
        (status, body).into_response()
    }
}

/// Global Result type alias for the application
///
/// This type alias allows using `Result<T>` throughout the project instead of
/// the more verbose `std::result::Result<T, SomeError>`, making the code cleaner
/// and more consistent.
///
/// # Example
///
/// ```rust
/// use backend::errors::Result;
///
/// fn some_function() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, SomeError>;
