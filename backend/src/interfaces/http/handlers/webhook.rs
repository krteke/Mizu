use axum::body::Bytes;
use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use gray_matter::{Matter, ParsedEntity, engine::YAML};
use octocrab::models::webhook_events::WebhookEvent;
use sqlx::PgPool;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domain::articles::{Article, ArticleFrontMatter};
use crate::errors::{Result, WebHooksError};
use crate::infrastructure::github::api_client::GithubApiClient;
use crate::infrastructure::github::client::GithubClient;
use crate::infrastructure::github::signature::verify_signature;
use crate::infrastructure::github::webhook::WebhookHandler;

pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    verify_signature(&body, &headers, &state.github_webhook_secret)?;

    let event_type = match headers.get("X-GitHub-Event") {
        Some(header) => match header.to_str() {
            Ok(t) => t,
            Err(_) => return Err(WebHooksError::InvalidHeader("X-GitHub-Event".to_string()).into()),
        },
        None => return Err(WebHooksError::MissingHeader("X-GitHub-Event".to_string()).into()),
    };

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(event) => event,
        Err(err) => return Err(err.into()),
    };

    // 捕获所有错误，始终返回 200 OK 避免 GitHub 无意义重试
    if let Err(err) = process_push_event(&state, &event).await {
        tracing::error!("Failed to process webhook payload: {}", err);
    }

    Ok("Webhook received".to_string())
}

// Process push event
async fn process_push_event(state: &Arc<AppState>, webhook_event: &WebhookEvent) -> Result<()> {
    // 如果无法获取基本信息，记录错误但不传播
    let repo_name = match webhook_event.get_repository_name() {
        Ok(name) => name,
        Err(err) => {
            tracing::error!("Failed to get repository name: {}", err);
            return Ok(());
        }
    };

    let owner = match webhook_event.get_repository_owner() {
        Ok(owner) => owner,
        Err(err) => {
            tracing::error!("Failed to get repository owner: {}", err);
            return Ok(());
        }
    };

    tracing::info!("Processing push event for repository: {}", repo_name);

    let changed_files = webhook_event.get_push_file_changes();

    for file_change in changed_files {
        if is_valid_file(&file_change.file_path) {
            match file_change.status.as_str() {
                "added" => {
                    if let Err(err) = process_added_file(
                        &state.db_pool,
                        &owner,
                        &repo_name,
                        &file_change.file_path,
                        &state.github_token,
                    )
                    .await
                    {
                        tracing::error!("Failed to process added file: {}", err);
                    } else {
                        tracing::info!("File {} added", file_change.file_path);
                    }
                }
                "modified" => {
                    if let Err(err) = process_modified_file(
                        &state.db_pool,
                        &owner,
                        &repo_name,
                        &file_change.file_path,
                        &state.github_token,
                    )
                    .await
                    {
                        tracing::error!("Failed to process modified file: {}", err);
                    } else {
                        tracing::info!("File {} modified", file_change.file_path);
                    }
                }
                "removed" => {
                    if let Err(err) =
                        process_removed_file(&state.db_pool, &file_change.file_path).await
                    {
                        tracing::error!("Failed to process removed file: {}", err);
                    } else {
                        tracing::info!("File {} removed", file_change.file_path);
                    }
                }
                _ => {
                    tracing::info!(
                        "Unknown file change status: {} for file {}",
                        file_change.status,
                        file_change.file_path
                    );
                }
            }
        }
    }

    Ok(())
}

// Check if a file is valid
fn is_valid_file(file_path: &str) -> bool {
    use std::path::Path;

    let allowed_extensions = ["md", "mdx"];

    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| allowed_extensions.contains(&ext))
        .unwrap_or(false)
}
