use axum::{http::HeaderMap, response::IntoResponse};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::errors::{Result, WebHooksError};

/// Verify the signature of a GitHub webhook request
///
/// This function validates that a webhook request actually came from GitHub by
/// verifying the HMAC-SHA256 signature included in the request headers. This is
/// a critical security measure to prevent unauthorized webhook requests.
///
/// # Security
///
/// GitHub signs webhook payloads with a secret key that you configure when setting
/// up the webhook. This function verifies that signature to ensure:
/// - The request came from GitHub, not a malicious actor
/// - The payload hasn't been tampered with during transit
/// - The webhook secret is correctly configured
///
/// # Algorithm
///
/// Uses HMAC-SHA256 (Hash-based Message Authentication Code with SHA-256):
/// 1. GitHub computes: HMAC-SHA256(payload, webhook_secret)
/// 2. Sends signature in `X-Hub-Signature-256` header as "sha256={hex_digest}"
/// 3. We recompute the HMAC with our local secret
/// 4. Compare the signatures in constant time to prevent timing attacks
///
/// # Arguments
///
/// * `payload_bytes` - The raw request body as bytes (before any parsing)
/// * `headers` - HTTP headers from the webhook request
/// * `secret` - The webhook secret configured in GitHub settings
///
/// # Returns
///
/// * `Ok("Ok")` - Signature is valid, request is authentic
/// * `Err(WebHooksError::VerifySignatureFailed)` - Signature verification failed
///
/// # Errors
///
/// Returns `VerifySignatureFailed` if:
/// - `X-Hub-Signature-256` header is missing
/// - Header value is not valid UTF-8
/// - Signature doesn't start with "sha256=" prefix
/// - Hex string cannot be decoded
/// - HMAC computation fails (invalid secret)
/// - Computed signature doesn't match provided signature
///
/// # Example
///
/// ```rust
/// use axum::body::Bytes;
/// use axum::http::HeaderMap;
/// use backend::infrastructure::github::signature::verify_signature;
///
/// async fn handle_webhook(headers: HeaderMap, body: Bytes) -> Result<()> {
///     let secret = std::env::var("GITHUB_WEBHOOK_SECRET")?;
///     verify_signature(&body, &headers, &secret)?;
///     // Process webhook...
///     Ok(())
/// }
/// ```
///
/// # Security Best Practices
///
/// - Always verify signatures before processing webhook payloads
/// - Store webhook secrets securely (environment variables, secrets manager)
/// - Use constant-time comparison to prevent timing attacks (handled by `verify_slice`)
/// - Rotate webhook secrets periodically
/// - Log failed verification attempts for security monitoring
///
/// # GitHub Documentation
///
/// See: https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries
pub fn verify_signature(
    payload_bytes: &[u8],
    headers: &HeaderMap,
    secret: &str,
) -> Result<impl IntoResponse> {
    // Type alias for HMAC-SHA256 for cleaner code
    type HmacSha256 = Hmac<Sha256>;

    // Extract the signature from the X-Hub-Signature-256 header
    // GitHub sends the signature in the format: "sha256={hex_digest}"
    let signature = match headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(header) => header,
        None => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    // Remove the "sha256=" prefix to get the hex-encoded signature
    let signature_hex = match signature.strip_prefix("sha256=") {
        Some(s) => s,
        None => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    // Decode the hex string to bytes
    // The signature is sent as hexadecimal representation of the HMAC digest
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    // Create HMAC instance with the webhook secret
    // This will be used to compute the expected signature
    let mut mac = if let Ok(key) = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()) {
        key
    } else {
        return Err(WebHooksError::VerifySignatureFailed.into());
    };

    // Compute HMAC of the payload using our secret
    mac.update(&payload_bytes);

    // Verify that our computed signature matches the one from GitHub
    // This uses constant-time comparison to prevent timing attacks
    mac.verify_slice(&signature_bytes)
        .map_err(|_| WebHooksError::VerifySignatureFailed)?;

    // Signature is valid - request is authenticated
    Ok("Ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_verify_signature_with_valid_signature() {
        // Test data
        let payload = b"test payload";
        let secret = "my_secret";

        // Compute expected signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let result = mac.finalize();
        let signature_hex = hex::encode(result.into_bytes());

        // Create headers with valid signature
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            format!("sha256={}", signature_hex).parse().unwrap(),
        );

        // Verification should succeed
        assert!(verify_signature(payload, &headers, secret).is_ok());
    }

    #[test]
    fn test_verify_signature_with_invalid_signature() {
        let payload = b"test payload";
        let secret = "my_secret";

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            "sha256=invalid_signature".parse().unwrap(),
        );

        // Verification should fail
        assert!(verify_signature(payload, &headers, secret).is_err());
    }

    #[test]
    fn test_verify_signature_with_missing_header() {
        let payload = b"test payload";
        let secret = "my_secret";
        let headers = HeaderMap::new();

        // Verification should fail when header is missing
        assert!(verify_signature(payload, &headers, secret).is_err());
    }

    #[test]
    fn test_verify_signature_with_wrong_prefix() {
        let payload = b"test payload";
        let secret = "my_secret";

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            "sha1=abc123".parse().unwrap(), // Wrong prefix
        );

        // Verification should fail with wrong prefix
        assert!(verify_signature(payload, &headers, secret).is_err());
    }
}
