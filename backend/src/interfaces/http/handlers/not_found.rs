use axum::{http::StatusCode, response::IntoResponse};

/// HTTP handler for 404 Not Found errors
///
/// This handler is invoked when no route matches the incoming request.
/// It returns a simple text response with HTTP 404 status code.
///
/// # Response
///
/// * Status Code: `404 Not Found`
/// * Content Type: `text/plain`
/// * Body: User-friendly error message
///
/// # Usage
///
/// This handler is typically registered as a fallback route in the Axum router:
///
/// ```rust
/// use axum::Router;
/// use backend::interfaces::http::handlers::not_found::handle_404;
///
/// let router = Router::new()
///     .route("/api/posts", get(get_posts))
///     // ... other routes ...
///     .fallback(handle_404);
/// ```
///
/// # Example Response
///
/// ```text
/// HTTP/1.1 404 Not Found
/// Content-Type: text/plain
///
/// 404 Not Found. The requested resource does not exist.
/// ```
///
/// # Design Notes
///
/// This handler uses a simple text response rather than JSON for 404 errors
/// to maintain compatibility with browser expectations and to distinguish
/// between API-specific errors (which return JSON) and routing errors.
///
/// For API-specific errors (like "article not found"), use the application's
/// error handling system which returns structured JSON responses.
pub async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "404 Not Found. The requested resource does not exist.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_handle_404_returns_not_found_status() {
        let response = handle_404().await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
