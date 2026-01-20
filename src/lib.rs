//! # Laneful Email SDK
//!
//! A Rust SDK for sending emails via the [Laneful](https://laneful.com) HTTP API.
//!
//! ## Features
//!
//! - **Sync API**: Always available (default)
//! - **Async API**: Enable with the `async` feature
//! - **TLS backends**: `native-tls` (default) or `rustls`
//!
//! ## Quick Start
//!
//! ```no_run
//! use laneful_rs::{LanefulClient, Email};
//!
//! let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key").unwrap();
//!
//! let email = Email::builder()
//!     .from("sender@example.com", Some("Sender Name"))
//!     .to("recipient@example.com", Some("Recipient"))
//!     .subject("Hello from Laneful!")
//!     .text_content("This is a test email.")
//!     .build()
//!     .unwrap();
//!
//! client.send_one(email).unwrap();
//! ```
//!
//! ## Using a Custom Base URL
//!
//! ```no_run
//! use laneful_rs::LanefulClient;
//!
//! let client = LanefulClient::with_base_url(
//!     "https://custom.api.example.com",
//!     "my-api-key"
//! ).unwrap();
//! ```
//!
//! ## Async Usage
//!
//! Enable the `async` feature in your `Cargo.toml`:
//!
//! ```toml
//! laneful-rs = { version = "0.1", features = ["async"] }
//! ```
//!
//! Then use the async methods:
//!
//! ```ignore
//! let response = client.send_one_async(email).await?;
//! ```

mod builder;
mod client;
mod error;
mod models;
mod webhook;

pub use builder::EmailBuilder;
pub use client::LanefulClient;
pub use error::{LanefulError, Result};
pub use models::{
    ApiErrorResponse, Attachment, Email, EmailAddress, SendEmailRequest, SendEmailResponse,
    Tracking,
};
pub use webhook::verify_webhook_signature;