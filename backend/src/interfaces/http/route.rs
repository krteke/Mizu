use axum::Router;
use std::sync::Arc;

use crate::{app_state::AppState, interfaces::http::handlers::not_found};

/// API routes module
///
/// This module defines all public API routes for the application.
/// Routes are organized under the `/api` prefix for versioning and clarity.
///
/// # Available Endpoints
///
/// - `GET /api/search` - Full-text search across articles
/// - `GET /api/posts` - List articles by category with pagination
/// - `GET /api/posts/{category}/{id}` - Get a specific article by ID
mod api {
    use super::*;
    use axum::routing::get;

    use crate::interfaces::http::handlers::{
        articles::{get_post_digital, get_posts},
        search::get_search_results,
    };

    /// Create the API router with all public endpoints
    ///
    /// This function configures all API routes that are always available,
    /// regardless of feature flags. These routes handle article retrieval
    /// and search functionality.
    ///
    /// # Routes
    ///
    /// - **Search**
    ///   - `GET /search?q={query}&page={page}` - Search articles
    ///
    /// - **Articles**
    ///   - `GET /posts?category={category}&page={page}&page_size={size}` - List articles
    ///   - `GET /posts/{category}/{id}` - Get specific article
    ///
    /// # Returns
    ///
    /// A configured `Router` that can be nested under `/api`
    ///
    /// # Example Usage
    ///
    /// ```text
    /// GET /api/search?q=rust&page=1
    /// GET /api/posts?category=article&page=1&page_size=20
    /// GET /api/posts/article/my-first-post
    /// ```
    pub fn router() -> Router<Arc<AppState>> {
        axum::Router::new()
            // Search endpoint: Full-text search with highlighting
            .route("/search", get(get_search_results))
            // List articles by category with pagination
            .route("/posts", get(get_posts))
            // Get a single article by category and ID
            .route("/posts/{category}/{id}", get(get_post_digital))
    }
}

/// Webhook routes module (conditional compilation)
///
/// This module is only compiled when the "webhook" feature flag is enabled.
/// It defines routes for receiving GitHub webhook events to automatically
/// update articles when repository content changes.
///
/// # Feature Flag
///
/// Enabled with: `cargo build --features webhook`
///
/// # Security
///
/// Webhook endpoints verify request signatures using HMAC-SHA256 to ensure
/// requests actually come from GitHub and haven't been tampered with.
///
/// # Available Endpoints
///
/// - `POST /api/webhook/github` - Receive GitHub webhook events
#[cfg(feature = "webhook")]
mod webhook {
    use super::*;
    use axum::routing::post;

    use crate::interfaces::http::handlers::webhook::github_webhook;

    /// Create the webhook router
    ///
    /// Configures the GitHub webhook endpoint that receives push events
    /// and other notifications from GitHub when repository content changes.
    ///
    /// # Routes
    ///
    /// - **GitHub Webhook**
    ///   - `POST /webhook/github` - Receive GitHub webhook events
    ///   - Requires valid HMAC-SHA256 signature in `X-Hub-Signature-256` header
    ///   - Processes push events to sync articles from repository
    ///
    /// # Returns
    ///
    /// A configured `Router` that can be merged with the main API router
    ///
    /// # Configuration
    ///
    /// To enable webhooks:
    /// 1. Build with webhook feature: `cargo build --features webhook`
    /// 2. Set `GITHUB_WEBHOOK_SECRET` environment variable
    /// 3. Configure webhook in GitHub repository settings
    ///
    /// # Example Request
    ///
    /// ```text
    /// POST /api/webhook/github
    /// X-Hub-Signature-256: sha256=abc123...
    /// X-GitHub-Event: push
    /// Content-Type: application/json
    ///
    /// {JSON webhook payload}
    /// ```
    pub fn router() -> Router<Arc<AppState>> {
        axum::Router::new().route("/webhook/github", post(github_webhook))
    }
}

/// Create the main application router with all routes configured
///
/// This is the top-level router configuration that combines all route modules
/// and sets up the routing hierarchy. It nests API routes under `/api` and
/// configures a fallback handler for 404 errors.
///
/// # Architecture
///
/// ```text
/// /
/// ├── /api/
/// │   ├── /search                    (GET)
/// │   ├── /posts                     (GET)
/// │   ├── /posts/{category}/{id}     (GET)
/// │   └── /webhook/github            (POST, webhook feature only)
/// └── /* (fallback)                  (404 handler)
/// ```
///
/// # Feature Flags
///
/// - **Default**: API routes only (search, articles)
/// - **webhook**: Adds GitHub webhook endpoint
///
/// # Returns
///
/// A fully configured `Router<Arc<AppState>>` ready to be served by Axum
///
/// # Example Usage
///
/// ```rust
/// use backend::interfaces::http::route::router;
/// use backend::app_state::AppState;
/// use std::sync::Arc;
///
/// let state = Arc::new(AppState::new(config).await?);
/// let app = router().with_state(state);
/// ```
///
/// # URL Examples
///
/// ```text
/// GET  /api/search?q=rust&page=1
/// GET  /api/posts?category=article&page=1&page_size=20
/// GET  /api/posts/article/introduction-to-rust
/// POST /api/webhook/github (webhook feature only)
/// GET  /nonexistent-path -> 404
/// ```
pub fn router() -> Router<Arc<AppState>> {
    // Start with the base API router (always available)
    let mut api_router = api::router();

    // Conditionally merge webhook routes if feature is enabled
    // This allows the webhook functionality to be compiled out when not needed
    #[cfg(feature = "webhook")]
    {
        api_router = api_router.merge(webhook::router());
    }

    // Build the main router with:
    // 1. All API routes nested under /api prefix
    // 2. A fallback handler for any unmatched routes (404)
    Router::new()
        .nest("/api", api_router)
        .fallback(not_found::handle_404)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_compiles() {
        // Basic compilation test to ensure router configuration is valid
        // This test doesn't require a runtime but verifies the route structure
        let _router = api::router();
    }

    #[cfg(feature = "webhook")]
    #[test]
    fn test_webhook_router_compiles() {
        // Verify webhook router compiles when feature is enabled
        let _router = webhook::router();
    }

    // TODO: Add integration tests for route matching
    // - Test that each route matches the expected path
    // - Test that query parameters are properly extracted
    // - Test that path parameters work correctly
    // - Test 404 fallback handler
    // - Test CORS headers if added
}
