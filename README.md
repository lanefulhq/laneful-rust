[![laneful-rs](https://img.shields.io/crates/v/laneful-rs.svg)](https://crates.io/crates/laneful-rs)


# laneful-rs

Rust SDK for the Laneful email sending API.

## Install

Add to `Cargo.toml`:

```toml
laneful-rs = "0.2"
```

Async API:

```toml
laneful-rs = { version = "0.2", features = ["async"] }
```

TLS backend (optional; defaults to rustls; use native-tls to switch):

```toml
laneful-rs = { version = "0.2", features = ["native-tls"] }
```

## Quick usage

```rust
use laneful_rs::{Email, LanefulClient};

let client = LanefulClient::new("https://custom-endpoint.send.laneful.net", "my-api-key")?;

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
    let client = LanefulClient::new("https://custom-endpoint.send.laneful.net", "my-api-key")?;

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

### Mail settings

```rust
use laneful_rs::{Email, LanefulClient, MailSettings, Tracking};

let email = Email::builder()
    .from("sender@example.com", Some("Sender"))
    .to("recipient@example.com", Some("Recipient"))
    .subject("Sandbox email")
    .text_content("This email is sent with sandbox mode.")
    .from_header("sender@example.com", Some("Newsletter"))
    .tracking(Tracking {
        opens: Some(true),
        clicks: Some(true),
        unsubscribes: Some(false),
        unsubscribe_group_id: None,
        unsubscribe_group_name: Some("Newsletters".into()),
    })
    .build()?;

let response = client.send_one_with_mail_settings(
    email,
    Some(MailSettings {
        sandbox_mode: Some(true),
        return_message_ids: Some(true),
    }),
)?;
println!("Status: {} {:?}", response.status, response.message_ids);
```

## Domain, unsubscribe groups, and analytics

These endpoints live on the organization API host. Point the client at it:

```rust
let client = LanefulClient::new("https://api.laneful.net", "my-api-key")?;
```

Development uses `https://api.dev.laneful.net`.

### Unsubscribe groups

```rust
use laneful_rs::ListUnsubscribeGroupsParams;

let groups = client.list_unsubscribe_groups(42, Some(&ListUnsubscribeGroupsParams {
    limit: Some(50),
    ..Default::default()
}))?;
let created = client.create_unsubscribe_group(42, "Newsletters")?;
let updated = client.update_unsubscribe_group(42, created.unsubscribe_group_id, "Weekly Newsletters")?;
```

### Domains

```rust
use laneful_rs::{CreateDomainRequest, ListDomainsParams, UpdateDomainRequest};

let listing = client.list_domains(42, Some(&ListDomainsParams {
    limit: Some(50),
    ..Default::default()
}))?;
let domain = client.create_domain(42, &CreateDomainRequest {
    domain: "mydomain.com".into(),
    tracking: "tracking".into(),
    return_path: "return-path".into(),
    require_tls: None,
    email_track_id: None,
})?;
client.get_domain(42, "mydomain.com")?;
client.verify_domain(42, "mydomain.com")?;

// Set the email track; pass "" to clear it, or None to leave it unchanged
client.update_domain(
    42,
    "mydomain.com",
    &UpdateDomainRequest::set("e59f0a35-05bc-4516-b585-c06f69c3e67e"),
)?;

client.delete_domain(42, "mydomain.com")?;
```

### Deliverability analytics

```rust
use laneful_rs::{
    ListDomainSpamRatioRadarParams, ListGooglePostmasterSpamReportsParams, ListSndsReportsParams,
};

let radar = client.list_domain_spam_ratio_radar(Some(&ListDomainSpamRatioRadarParams {
    workspace_ids: vec![1, 2],
    domain: Some("example.com".into()),
    start_date: Some("2026-09-01".into()),
    end_date: Some("2026-09-08".into()),
    ..Default::default()
}))?;

let postmaster = client.list_google_postmaster_spam_reports(Some(
    &ListGooglePostmasterSpamReportsParams {
        domain: Some("example.com".into()),
        ..Default::default()
    },
))?;

let snds = client.list_snds_reports(Some(&ListSndsReportsParams {
    ip: Some("203.0.113.5".into()),
    ..Default::default()
}))?;
```

## Examples

Set env vars:

```bash
export LANEFUL_ENDPOINT="https://custom-endpoint.send.laneful.net"
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

See `examples/` for mail settings, domains, unsubscribe groups, and analytics.

## Notes

- Async methods are available when the `async` feature is enabled.
- TLS backend is `rustls` by default; use `native-tls` to switch.
- Webhook event types include `request`, `delivery`, `open`, `click`, `drop`, `spam_complaint`, `unsubscribe`, and `bounce`.
