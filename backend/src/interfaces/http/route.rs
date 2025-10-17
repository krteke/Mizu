use axum::Router;
use std::sync::Arc;

use crate::{app_state::AppState, interfaces::http::handlers::not_found};

mod api {
    use super::*;
    use axum::routing::get;

    use crate::interfaces::http::handlers::{
        articles::{get_post_digital, get_posts},
        search::get_search_results,
    };

    pub fn router() -> Router<Arc<AppState>> {
        axum::Router::new()
            .route("/search", get(get_search_results))
            .route("/posts", get(get_posts))
            .route("/posts/{category}/{id}", get(get_post_digital))
    }
}

#[cfg(feature = "webhook")]
mod webhook {
    use super::*;
    use axum::routing::post;

    use crate::interfaces::http::handlers::webhook::github_webhook;

    pub fn router() -> Router<Arc<AppState>> {
        axum::Router::new().route("/webhook/github", post(github_webhook))
    }
}

pub fn router() -> Router<Arc<AppState>> {
    let mut api_router = api::router();

    #[cfg(feature = "webhook")]
    {
        api_router = api_router.merge(webhook::router());
    }

    Router::new()
        .nest("/api", api_router)
        .fallback(not_found::handle_404)
}
