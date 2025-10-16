use axum::{http::StatusCode, response::IntoResponse};

pub async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "404 Not Found. The requested resource does not exist.",
    )
}
