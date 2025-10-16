use axum::{http::HeaderMap, response::IntoResponse};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::errors::{Result, WebHooksError};

pub fn verify_signature(
    payload_bytes: &[u8],
    headers: &HeaderMap,
    secret: &str,
) -> Result<impl IntoResponse> {
    type HmacSha256 = Hmac<Sha256>;

    let signature = match headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(header) => header,
        None => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    let signature_hex = match signature.strip_prefix("sha256=") {
        Some(s) => s,
        None => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err(WebHooksError::VerifySignatureFailed.into()),
    };

    let mut mac = if let Ok(key) = <HmacSha256 as Mac>::new_from_slice(secret.as_bytes()) {
        key
    } else {
        return Err(WebHooksError::VerifySignatureFailed.into());
    };
    mac.update(&payload_bytes);

    mac.verify_slice(&signature_bytes)
        .map_err(|_| WebHooksError::VerifySignatureFailed)?;

    Ok("Ok")
}
