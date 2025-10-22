use axum::body::Bytes;
use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use octocrab::models::webhook_events::WebhookEvent;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::errors::{Result, WebHooksError};
use crate::infrastructure::github::signature::verify_signature;

/// HTTP handler for GitHub webhook events
///
/// This endpoint receives webhook notifications from GitHub when repository
/// events occur (e.g., push events). It verifies the request signature,
/// parses the event payload, and processes it asynchronously.
///
/// # Request Format
///
/// ```text
/// POST /webhook/github
/// Headers:
///   X-Hub-Signature-256: sha256={signature}
///   X-GitHub-Event: {event_type}
///   Content-Type: application/json
/// Body: {webhook_payload}
/// ```
///
/// # Security
///
/// The webhook signature is verified using HMAC-SHA256 to ensure the request
/// actually came from GitHub and hasn't been tampered with. Requests with
/// invalid signatures are rejected with a 401 Unauthorized response.
///
/// # Error Handling Strategy
///
/// This handler follows GitHub's webhook best practices:
/// - Always returns 200 OK once signature is verified, even if processing fails
/// - Errors during processing are logged but don't fail the request
/// - This prevents GitHub from retrying webhook deliveries unnecessarily
///
/// # Arguments
///
/// * `State(state)` - Shared application state containing services and configuration
/// * `headers` - HTTP headers including signature and event type
/// * `body` - Raw request body as bytes (needed for signature verification)
///
/// # Returns
///
/// * `Ok("Webhook received")` - Request was authenticated and queued for processing
/// * `Err(WebHooksError::VerifySignatureFailed)` - Signature verification failed
/// * `Err(WebHooksError::MissingHeader)` - Required header is missing
/// * `Err(SomeError)` - Other error occurred during parsing
///
/// # Response Codes
///
/// - `200 OK` - Webhook received and accepted (even if processing fails)
/// - `401 Unauthorized` - Signature verification failed
/// - `400 Bad Request` - Missing required headers or invalid payload
///
/// # Supported Events
///
/// Currently supports:
/// - **Push events**: Processes file changes when commits are pushed
/// - Other event types are logged but not processed
///
/// # Example Request
///
/// ```bash
/// curl -X POST http://localhost:8124/api/webhook/github \
///   -H "X-Hub-Signature-256: sha256=abc123..." \
///   -H "X-GitHub-Event: push" \
///   -H "Content-Type: application/json" \
///   -d '{"ref":"refs/heads/main","commits":[...]}'
/// ```
///
/// # Processing Flow
///
/// 1. Verify HMAC-SHA256 signature from `X-Hub-Signature-256` header
/// 2. Extract event type from `X-GitHub-Event` header
/// 3. Parse webhook payload using octocrab's WebhookEvent parser
/// 4. Pass event to article service for asynchronous processing
/// 5. Log any processing errors but still return 200 OK
/// 6. Return success response to GitHub
///
/// # GitHub Documentation
///
/// See: https://docs.github.com/en/webhooks
pub async fn github_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    // Step 1: Verify the webhook signature to ensure authenticity
    // This prevents unauthorized requests and ensures payload integrity
    // Uses HMAC-SHA256 with the configured webhook secret
    verify_signature(&body, &headers, &state.app_config.github_webhook_secret)?;

    // Step 2: Extract the event type from the X-GitHub-Event header
    // This tells us what kind of webhook event occurred (push, pull_request, etc.)
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| WebHooksError::MissingHeader("X-GitHub-Event".to_string()))?;

    // Step 3: Parse the webhook payload into a structured WebhookEvent
    // The octocrab library handles the complex JSON parsing based on event type
    let event = WebhookEvent::try_from_header_and_body(event_type, &body)?;

    // Step 4: Process the webhook event asynchronously
    // Catch all errors and log them, but don't fail the request
    // This prevents GitHub from retrying webhook deliveries for transient errors
    // that we can handle later (e.g., temporary database issues)
    if let Err(err) = state
        .article_service
        .process_github_webhook_event(&event)
        .await
    {
        tracing::error!("Failed to process webhook payload: {}", err);
    }

    // Step 5: Always return 200 OK after signature verification
    // This tells GitHub we received the webhook successfully
    // Any processing errors are logged but don't cause GitHub to retry
    Ok("Webhook received".to_string())
}

#[cfg(test)]
mod tests {

    // TODO: Add integration tests for webhook handler
    // - Test signature verification with valid and invalid signatures
    // - Test handling of different event types (push, pull_request, etc.)
    // - Test error handling (missing headers, invalid payloads)
    // - Test that processing errors don't fail the request
    // - Mock the article service to verify it's called correctly
}
