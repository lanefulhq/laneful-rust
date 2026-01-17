# laneful-rs

Rust SDK for the Laneful email sending API.

## Install

Add to `Cargo.toml`:

```toml
laneful-rs = "0.1"
```

Async API:

```toml
laneful-rs = { version = "0.1", features = ["async"] }
```

TLS backend (optional; defaults to rustls; use native-tls to switch):

```toml
laneful-rs = { version = "0.1", features = ["native-tls"] }
```

## Quick usage

```rust
use laneful_rs::{Email, LanefulClient};

let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key")?;

let email = Email::builder()
    .from("sender@example.com", Some("Sender"))
    .to("recipient@example.com", Some("Recipient"))
    .subject("Hello from Laneful!")
    .text_content("This is a test email.")
    .build()?;

let response = client.send_one(email)?;
println!("Sent: {:?}", response);
```

Async usage (requires the `async` feature):

```rust
use laneful_rs::{Email, LanefulClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key")?;

    let email = Email::builder()
        .from("sender@example.com", Some("Sender"))
        .to("recipient@example.com", Some("Recipient"))
        .subject("Hello from Laneful (async)!")
        .text_content("This is a test email.")
        .build()?;

    let response = client.send_one_async(email).await?;
    println!("Sent: {:?}", response);
    Ok(())
}
```

## Examples

Set env vars:

```bash
export LANEFUL_ENDPOINT="https://custom-endpoint.api.laneful.com"
export LANEFUL_API_KEY="my-api-key"
```

Run the async example:

```bash
cargo run --example async --features async -- --from sender@example.com --to recipient@example.com
```

Run the sync example:

```bash
cargo run --example sync -- --from sender@example.com --to recipient@example.com
```

## Notes

- Async methods are available when the `async` feature is enabled.
- TLS backend is `rustls` by default; use `native-tls` to switch.
