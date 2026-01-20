//! Webhook signature verification utilities.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

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
