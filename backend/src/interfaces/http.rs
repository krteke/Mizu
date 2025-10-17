//! HTTP interface layer for the application
//!
//! This module contains all HTTP-related code including request handlers,
//! response DTOs (Data Transfer Objects), and route configurations.
//! It serves as the presentation layer in the clean architecture pattern.
//!
//! # Architecture
//!
//! This module follows a layered structure:
//! - **handlers**: HTTP request handlers for different endpoints
//! - **dtos**: Data Transfer Objects for request/response serialization
//! - **route**: Route configuration and URL mapping
//!
//! # Module Structure
//!
//! ```text
//! interfaces/http/
//! ├── handlers/
//! │   ├── articles.rs     - Article retrieval endpoints
//! │   ├── search.rs       - Search functionality endpoints
//! │   ├── webhook.rs      - GitHub webhook receiver (webhook feature)
//! │   └── not_found.rs    - 404 error handler
//! ├── dtos.rs             - Request/Response data structures
//! └── route.rs            - Route configuration and URL mapping
//! ```
//!
//! # Design Principles
//!
//! - **Thin Controllers**: Handlers contain minimal logic, delegating to service layer
//! - **DTO Pattern**: Separate DTOs from domain entities for API stability
//! - **Error Handling**: All handlers return `Result<T, SomeError>` for consistent error responses
//! - **Stateless**: Handlers are stateless functions receiving application state via Axum's `State` extractor
//!
//! # Example Handler Flow
//!
//! ```text
//! HTTP Request
//!     ↓
//! Handler (interfaces/http/handlers)
//!     ↓ extracts params
//! Service Layer (application)
//!     ↓ business logic
//! Repository/Search (infrastructure)
//!     ↓ data access
//! Database/Search Engine
//!     ↓
//! Response ← Handler ← Service ← Repository
//! ```
//!
//! # Available Endpoints
//!
//! ## Public API
//!
//! - `GET /api/search` - Full-text search across articles
//! - `GET /api/posts` - List articles by category
//! - `GET /api/posts/{category}/{id}` - Get a specific article
//!
//! ## Webhook Endpoints (webhook feature only)
//!
//! - `POST /api/webhook/github` - Receive GitHub webhook events
//!
//! # Example Usage
//!
//! ```rust
//! use backend::interfaces::http::route::router;
//! use backend::app_state::AppState;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize application state
//!     let config = Config::new()?;
//!     let state = Arc::new(AppState::new(config).await?);
//!
//!     // Create router with all routes configured
//!     let app = router().with_state(state);
//!
//!     // Serve the application
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:8124").await?;
//!     axum::serve(listener, app).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Error Handling
//!
//! All handlers return errors that implement `IntoResponse`, automatically
//! converting application errors into appropriate HTTP responses:
//!
//! - 4xx errors for client issues (bad request, not found, etc.)
//! - 5xx errors for server issues (database errors, etc.)
//! - Consistent JSON error format across all endpoints
//!
//! # Testing
//!
//! HTTP handlers can be tested using Axum's testing utilities:
//!
//! ```rust
//! use axum::body::Body;
//! use axum::http::{Request, StatusCode};
//! use tower::ServiceExt;
//!
//! #[tokio::test]
//! async fn test_get_posts() {
//!     let app = router().with_state(test_state());
//!
//!     let response = app
//!         .oneshot(Request::builder()
//!             .uri("/api/posts?category=article&page=1")
//!             .body(Body::empty())
//!             .unwrap())
//!         .await
//!         .unwrap();
//!
//!     assert_eq!(response.status(), StatusCode::OK);
//! }
//! ```

pub mod dtos;
pub mod handlers;
pub mod route;
