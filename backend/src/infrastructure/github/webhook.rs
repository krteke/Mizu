use octocrab::models::webhook_events::{
    WebhookEvent, WebhookEventType, payload::WebhookEventPayload,
};
use serde::{Deserialize, Serialize};

use crate::errors::{Result, WebHooksError};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChange {
    pub file_path: String,
    pub status: String,
    pub row_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PushEvent {
    pub commits: Option<Vec<CommitInfo>>,
    pub repository: RepositoryInfo,
    pub sender: OwnerInfo,
}

#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub full_name: String,
    pub owner: OwnerInfo,
}

#[derive(Debug, Deserialize)]
pub struct OwnerInfo {
    pub login: String,
}

pub trait WebhookHandler {
    fn get_push_file_changes(&self) -> Vec<FileChange>;

    fn get_repository_name(&self) -> Result<String>;

    fn get_repository_owner(&self) -> Result<String>;
}

// pub fn parse_webhook_event(payload: &serde_json::Value) -> Result<PushEvent> {
//     Ok(serde_json::from_value(payload.clone())?)
// }

impl WebhookHandler for WebhookEvent {
    fn get_push_file_changes(&self) -> Vec<FileChange> {
        let mut changes = Vec::new();

        if self.kind == WebhookEventType::Push {
            if let WebhookEventPayload::Push(push_payload) = &self.specific {
                for commit in &push_payload.commits {
                    for file in &commit.added {
                        changes.push(FileChange {
                            file_path: file.clone(),
                            status: "added".to_string(),
                            row_url: None,
                        });
                    }

                    for file in &commit.removed {
                        changes.push(FileChange {
                            file_path: file.clone(),
                            status: "removed".to_string(),
                            row_url: None,
                        });
                    }

                    for file in &commit.modified {
                        changes.push(FileChange {
                            file_path: file.clone(),
                            status: "modified".to_string(),
                            row_url: None,
                        });
                    }
                }
            }
        }

        changes
    }

    fn get_repository_name(&self) -> Result<String> {
        match &self.specific {
            WebhookEventPayload::Push(_) | WebhookEventPayload::PullRequest(_) => self
                .repository
                .as_ref()
                .and_then(|repo| repo.full_name.clone())
                .ok_or_else(|| WebHooksError::MissingRepositoryName.into()),

            _ => Err(WebHooksError::UnsupportedWebhookEvent.into()),
        }
    }

    fn get_repository_owner(&self) -> Result<String> {
        match &self.specific {
            WebhookEventPayload::Push(_) | WebhookEventPayload::PullRequest(_) => self
                .repository
                .as_ref()
                .and_then(|repo| repo.owner.as_ref())
                .map(|owner| owner.login.clone())
                .ok_or_else(|| WebHooksError::MissingRepositoryName.into()),

            _ => Err(WebHooksError::UnsupportedWebhookEvent.into()),
        }
    }
}
