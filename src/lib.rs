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
//! laneful-rs = { version = "0.2", features = ["async"] }
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
mod org;
mod webhook;

pub use builder::EmailBuilder;
pub use client::LanefulClient;
pub use error::{LanefulError, Result};
pub use models::{
    ApiErrorResponse, Attachment, Email, EmailAddress, MailSettings, SendEmailRequest,
    SendEmailResponse, Tracking,
};
pub use org::{
    CreateDomainRequest, Domain, DomainSpamRatioRadar, GooglePostmasterSpamReport,
    ListDomainSpamRatioRadarParams, ListDomainSpamRatioRadarResponse, ListDomainsParams,
    ListDomainsResponse, ListGooglePostmasterSpamReportsParams,
    ListGooglePostmasterSpamReportsResponse, ListSndsReportsParams, ListSndsReportsResponse,
    ListUnsubscribeGroupsParams, ListUnsubscribeGroupsResponse, QueryItems, SNDS_FILTER_GREEN,
    SNDS_FILTER_RED, SNDS_FILTER_UNKNOWN, SNDS_FILTER_YELLOW, SndsReport, SuccessResponse,
    UnsubscribeGroup, UpdateDomainRequest,
};
pub use webhook::{
    WEBHOOK_EVENT_BOUNCE, WEBHOOK_EVENT_CLICK, WEBHOOK_EVENT_DELIVERY, WEBHOOK_EVENT_DROP,
    WEBHOOK_EVENT_OPEN, WEBHOOK_EVENT_REQUEST, WEBHOOK_EVENT_SPAM_COMPLAINT, WEBHOOK_EVENT_TYPES,
    WEBHOOK_EVENT_UNSUBSCRIBE, is_valid_webhook_event, verify_webhook_signature,
};
