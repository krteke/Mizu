use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::errors::Result;
use crate::infrastructure::github::webhook::WebhookHandler;

pub async fn process_webhook_payload(
    state: &Arc<AppState>,
    payload: &WebhookEvent,
) -> Result<WebhookEventType> {
    let repo_name = payload.get_repository_name()?;

    let allowed_repos = state.allowed_repositories.read().await;
    if !allowed_repos.contains(&repo_name) {
        tracing::info!("Repository {} is not allowed", repo_name);
        return Err(anyhow::anyhow!("Repository not allowed").into());
    }

    tracing::info!("Processing webhook event for repository: {}", repo_name);

    // match &payload.kind {
    //     WebhookEventType::Push => {
    //         if let Err(err) = process_push_event(state, payload).await {
    //             tracing::error!("Failed to process push event: {}", err);
    //         }
    //     }
    //     event_type => {
    //         tracing::info!("Received unsupported event type: {:?}", event_type);
    //     }
    // }
    Ok(payload.kind.clone())
}
