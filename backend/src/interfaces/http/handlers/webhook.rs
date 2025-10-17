use axum::body::Bytes;
use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use octocrab::models::webhook_events::WebhookEvent;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::errors::{Result, WebHooksError};
use crate::infrastructure::github::signature::verify_signature;

pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    verify_signature(&body, &headers, &state.app_config.github_webhook_secret)?;

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| WebHooksError::MissingHeader("X-GitHub-Event".to_string()))?;

    let evnet = WebhookEvent::try_from_header_and_body(event_type, &body)?;

    // 捕获所有错误，始终返回 200 OK 避免 GitHub 无意义重试
    if let Err(err) = state
        .article_service
        .process_github_webhook_event(&evnet)
        .await
    {
        tracing::error!("Failed to process webhook payload: {}", err);
    }

    Ok("Webhook received".to_string())
}
