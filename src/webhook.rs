//! Webhook signature verification and event types.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Email send request accepted.
pub const WEBHOOK_EVENT_REQUEST: &str = "request";
/// Email successfully delivered.
pub const WEBHOOK_EVENT_DELIVERY: &str = "delivery";
/// Email opened.
pub const WEBHOOK_EVENT_OPEN: &str = "open";
/// Link clicked.
pub const WEBHOOK_EVENT_CLICK: &str = "click";
/// Email dropped.
pub const WEBHOOK_EVENT_DROP: &str = "drop";
/// Spam complaint.
pub const WEBHOOK_EVENT_SPAM_COMPLAINT: &str = "spam_complaint";
/// Recipient unsubscribed.
pub const WEBHOOK_EVENT_UNSUBSCRIBE: &str = "unsubscribe";
/// Email bounced.
pub const WEBHOOK_EVENT_BOUNCE: &str = "bounce";

/// Known webhook event types, including `request`.
pub const WEBHOOK_EVENT_TYPES: &[&str] = &[
    WEBHOOK_EVENT_REQUEST,
    WEBHOOK_EVENT_DELIVERY,
    WEBHOOK_EVENT_OPEN,
    WEBHOOK_EVENT_CLICK,
    WEBHOOK_EVENT_DROP,
    WEBHOOK_EVENT_SPAM_COMPLAINT,
    WEBHOOK_EVENT_UNSUBSCRIBE,
    WEBHOOK_EVENT_BOUNCE,
];

/// Returns true if `event` is a documented Laneful webhook event type.
pub fn is_valid_webhook_event(event: &str) -> bool {
    WEBHOOK_EVENT_TYPES.contains(&event)
}

/// Verifies the signature of a webhook payload.
///
/// # Arguments
///
/// * `secret` - The webhook secret key
/// * `payload` - The raw webhook payload body as bytes
/// * `signature` - The signature from the webhook header
///
/// # Returns
///
/// `true` if the signature is valid, `false` otherwise.
///
/// # Example
///
/// ```
/// use laneful_rs::verify_webhook_signature;
///
/// let secret = "my-webhook-secret";
/// let payload = br#"{"event":"email.sent"}"#;
/// let signature = "expected-signature-hex";
///
/// if verify_webhook_signature(secret, payload, signature) {
///     println!("Webhook signature is valid!");
/// }
/// ```
pub fn verify_webhook_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload);
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison to prevent timing attacks
    expected.as_bytes().ct_eq(signature.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_event_is_valid() {
        assert!(is_valid_webhook_event("request"));
        assert!(WEBHOOK_EVENT_TYPES.contains(&WEBHOOK_EVENT_REQUEST));
        assert!(!is_valid_webhook_event("not-a-real-event"));
    }
}
