use axum::{
    Router,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use laneful_rs::verify_webhook_signature;

const SIGNATURE_HEADER: &str = "X-Webhook-Signature";

async fn webhook_handler(headers: HeaderMap, body: Bytes) -> StatusCode {
    let secret = std::env::var("LANEFUL_WEBHOOK_SECRET")
        .expect("LANEFUL_WEBHOOK_SECRET env var is required");

    let signature = match headers.get(SIGNATURE_HEADER) {
        Some(sig) => sig.to_str().unwrap_or_default(),
        None => {
            println!("Missing signature header");
            return StatusCode::UNAUTHORIZED;
        }
    };

    if !verify_webhook_signature(&secret, &body, signature) {
        println!("Invalid webhook signature");
        return StatusCode::UNAUTHORIZED;
    }

    println!("════════════════════════════════════════════════════════════");
    println!("✓ Webhook signature verified\n");
    println!("Headers:");
    for (name, value) in headers.iter() {
        println!("  {}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
    let payload = String::from_utf8_lossy(&body);
    println!("\nPayload:\n{}\n", payload);

    StatusCode::OK
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3344".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new().route("/", post(webhook_handler));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Webhook server listening on http://localhost:{}/", port);
    axum::serve(listener, app).await.unwrap();
}
