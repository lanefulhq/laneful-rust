//! Data models for the Laneful Email API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An email address with an optional display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// The email address.
    pub email: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl EmailAddress {
    /// Create a new email address.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    /// Create a new email address with a display name.
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

impl<S: Into<String>> From<S> for EmailAddress {
    fn from(email: S) -> Self {
        Self::new(email)
    }
}

/// An email attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// The filename of the attachment.
    pub file_name: String,
    /// Base64-encoded content of the attachment.
    pub content: String,
    /// MIME type of the attachment.
    pub content_type: String,
}

impl Attachment {
    /// Create a new attachment.
    pub fn new(
        file_name: impl Into<String>,
        content: impl Into<String>,
        content_type: impl Into<String>,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            content: content.into(),
            content_type: content_type.into(),
        }
    }
}

/// Email tracking settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tracking {
    /// Track when recipients open emails (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opens: Option<bool>,
    /// Track when recipients click links (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clicks: Option<bool>,
    /// Track unsubscribe events (default: true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsubscribes: Option<bool>,
    /// Optional unsubscribe group ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsubscribe_group_id: Option<u64>,
}

/// A single email to be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Sender email address (required).
    pub from: EmailAddress,
    /// Primary recipients (required).
    pub to: Vec<EmailAddress>,
    /// Email subject line (required).
    pub subject: String,
    /// Plain text email content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    /// HTML email content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,
    /// Reply-to address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<EmailAddress>,
    /// Carbon copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,
    /// Blind carbon copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
    /// File attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    /// Custom email headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Template ID for pre-built templates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// Template data for variable substitution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_data: Option<serde_json::Value>,
    /// Unix timestamp for scheduled sending (max 72 hours future).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_time: Option<u64>,
    /// Custom data sent to webhook endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_data: Option<HashMap<String, String>>,
    /// Label for organizing and tracking emails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Email tracking settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking: Option<Tracking>,
}

/// Request body for sending emails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    /// List of emails to send.
    pub emails: Vec<Email>,
}

/// Response from the send email endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailResponse {
    /// Status of the request (e.g., "accepted").
    pub status: String,
}

/// Error response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message.
    pub error: String,
}
