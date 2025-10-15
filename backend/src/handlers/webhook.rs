use crate::{
    common::AppState,
    github_api::GithubApiClient,
    github_webhook::WebhookHandler,
    some_errors::{Result, WebHooksError},
};

use axum::body::Bytes;
use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use gray_matter::{Matter, ParsedEntity, engine::YAML};
use hmac::{Hmac, Mac};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use sha2::Sha256;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    if !verify_signature(&body, &headers, &state.github_webhook_secret) {
        return Err(WebHooksError::VerifySignatureFailed.into());
    }

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
    if let Err(err) = process_webhook_payload(&state, &event).await {
        tracing::error!("Failed to process webhook payload: {}", err);
    }

    Ok("Webhook received".to_string())
}

fn verify_signature(payload_bytes: &[u8], headers: &HeaderMap, secret: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    let signature = match headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(header) => header,
        None => return false,
    };

    let signature_hex = match signature.strip_prefix("sha256=") {
        Some(s) => s,
        None => return false,
    };

    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let mut mac = if let Ok(key) = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()) {
        key
    } else {
        return false;
    };
    mac.update(&payload_bytes);

    mac.verify_slice(&signature_bytes).is_ok()
}

async fn process_webhook_payload(state: &Arc<AppState>, payload: &WebhookEvent) -> Result<()> {
    let repo_name = payload.get_repository_name()?;

    let allowed_repos = state.allowed_repositories.read().await;
    if !allowed_repos.contains(&repo_name) {
        tracing::info!("Repository {} is not allowed", repo_name);
        return Ok(());
    }

    tracing::info!("Processing webhook event for repository: {}", repo_name);

    match &payload.kind {
        WebhookEventType::Push => {
            if let Err(err) = process_push_event(state, payload).await {
                tracing::error!("Failed to process push event: {}", err);
            }
        }
        event_type => {
            tracing::info!("Received unsupported event type: {:?}", event_type);
        }
    }
    Ok(())
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
        if is_valid_file(&file_change.filename) {
            match file_change.status.as_str() {
                "added" => {
                    if let Err(err) = process_added_file(
                        &state.db_pool,
                        &owner,
                        &repo_name,
                        &file_change.filename,
                        &state.github_token,
                    )
                    .await
                    {
                        tracing::error!("Failed to process added file: {}", err);
                    } else {
                        tracing::info!("File {} added", file_change.filename);
                    }
                }
                "modified" => {
                    if let Err(err) = process_modified_file(
                        &state.db_pool,
                        &owner,
                        &repo_name,
                        &file_change.filename,
                        &state.github_token,
                    )
                    .await
                    {
                        tracing::error!("Failed to process modified file: {}", err);
                    } else {
                        tracing::info!("File {} modified", file_change.filename);
                    }
                }
                "removed" => {
                    if let Err(err) =
                        process_removed_file(&state.db_pool, &file_change.filename).await
                    {
                        tracing::error!("Failed to process removed file: {}", err);
                    } else {
                        tracing::info!("File {} removed", file_change.filename);
                    }
                }
                _ => {
                    tracing::info!(
                        "Unknown file change status: {} for file {}",
                        file_change.status,
                        file_change.filename
                    );
                }
            }
        }
    }

    Ok(())
}

async fn process_added_file(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    file_path: &str,
    github_token: &str,
) -> Result<()> {
    let client = GithubApiClient::new(github_token)?;
    let content = client.get_file_content(owner, repo, file_path).await?;

    let matter = Matter::<YAML>::new();
    let _result: ParsedEntity = matter.parse(&content)?;

    // TODO: 提取 frontmatter 并插入数据库
    // TODO: 更新搜索索引

    Ok(())
}

async fn process_modified_file(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    file_path: &str,
    github_token: &str,
) -> Result<()> {
    // 修改文件的处理逻辑与添加相同：获取最新内容并更新
    let client = GithubApiClient::new(github_token)?;
    let content = client.get_file_content(owner, repo, file_path).await?;

    let matter = Matter::<YAML>::new();
    let _result: ParsedEntity = matter.parse(&content)?;

    // TODO: 提取 frontmatter 并更新数据库
    // TODO: 更新搜索索引

    Ok(())
}

async fn process_removed_file(pool: &PgPool, file_path: &str) -> Result<()> {
    // TODO: 从文件路径提取文章 ID
    // TODO: 从数据库删除文章
    // TODO: 从搜索索引删除

    tracing::info!("Would remove file: {}", file_path);
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
