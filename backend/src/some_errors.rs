use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Can not get MEILISEARCH_URL from env.")]
    MeilisearchUrlMissing,
    #[error("Can not get MEILI_MASTER_KEY from env.")]
    MasterKeyMissing,
    #[error("Can not found default search api key.")]
    DefaultSearchApiKeyNotFound,
    #[error("Can not found default admin api key.")]
    DefaultAdminApiKeyNotFound,
    #[error("Can not found this key, it is {0}, is it valid?")]
    KeyNameNotFound(String),
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Can not get DATABASE_URL from env.")]
    DatabaseUrlMissing,
    #[error("Database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum GetPostsError {
    #[error("Invalid Category type.")]
    CategoryError,
}

#[derive(Debug, Error)]
pub enum SomeError {
    #[error(transparent)]
    Search(#[from] SearchError),
    #[error(transparent)]
    Database(#[from] DBError),
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
    #[error(transparent)]
    GetPosts(#[from] GetPostsError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for SomeError {
    fn from(err: sqlx::Error) -> Self {
        SomeError::Database(DBError::QueryFailed(err))
    }
}

impl IntoResponse for SomeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error = %self,"Unhandled application error");

        let (status, user_message) = match &self {
            SomeError::Search(SearchError::MeilisearchUrlMissing)
            | SomeError::Search(SearchError::MasterKeyMissing)
            | SomeError::Database(DBError::DatabaseUrlMissing) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Server misconfiguration")
            }

            SomeError::Meilisearch(_) => (StatusCode::BAD_GATEWAY, "Search service unavailable"),

            SomeError::Search(
                SearchError::DefaultSearchApiKeyNotFound
                | SearchError::DefaultAdminApiKeyNotFound
                | SearchError::KeyNameNotFound(_),
            ) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Search service misconfigured",
            ),

            SomeError::Database(DBError::QueryFailed(_)) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database temporarily unavailable",
            ),

            SomeError::GetPosts(GetPostsError::CategoryError) => (
                StatusCode::BAD_REQUEST,
                "Something was invalid in requests.",
            ),

            SomeError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
            ),
        };

        let body = Json(json!({
            "error": user_message,
            "status": "error"
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, SomeError>;
