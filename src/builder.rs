//! Builder pattern for constructing emails.

use crate::error::{LanefulError, Result};
use crate::models::{Attachment, Email, EmailAddress, Tracking};
use std::collections::HashMap;

const MAX_RECIPIENTS: usize = 1000;
const MAX_TAG_LENGTH: usize = 100;
const MAX_WEBHOOK_DATA_KEYS: usize = 20;
const MAX_WEBHOOK_DATA_KEY_LENGTH: usize = 50;
const MAX_WEBHOOK_DATA_VALUE_LENGTH: usize = 100;

/// Builder for constructing [`Email`] instances.
#[derive(Debug, Default)]
pub struct EmailBuilder {
    from: Option<EmailAddress>,
    to: Vec<EmailAddress>,
    subject: Option<String>,
    text_content: Option<String>,
    html_content: Option<String>,
    reply_to: Option<EmailAddress>,
    cc: Vec<EmailAddress>,
    bcc: Vec<EmailAddress>,
    attachments: Vec<Attachment>,
    headers: HashMap<String, String>,
    template_id: Option<String>,
    template_data: Option<serde_json::Value>,
    send_time: Option<u64>,
    webhook_data: HashMap<String, String>,
    tag: Option<String>,
    tracking: Option<Tracking>,
}

impl EmailBuilder {
    /// Create a new email builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sender email address.
    pub fn from(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        self.from = Some(match name {
            Some(n) => EmailAddress::with_name(email, n),
            None => EmailAddress::new(email),
        });
        self
    }

    /// Add a primary recipient.
    pub fn to(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        self.to.push(match name {
            Some(n) => EmailAddress::with_name(email, n),
            None => EmailAddress::new(email),
        });
        self
    }

    /// Add multiple primary recipients.
    pub fn to_many(mut self, recipients: Vec<EmailAddress>) -> Self {
        self.to.extend(recipients);
        self
    }

    /// Set the email subject.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the plain text content.
    pub fn text_content(mut self, content: impl Into<String>) -> Self {
        self.text_content = Some(content.into());
        self
    }

    /// Set the HTML content.
    pub fn html_content(mut self, content: impl Into<String>) -> Self {
        self.html_content = Some(content.into());
        self
    }

    /// Set the reply-to address.
    pub fn reply_to(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        self.reply_to = Some(match name {
            Some(n) => EmailAddress::with_name(email, n),
            None => EmailAddress::new(email),
        });
        self
    }

    /// Add a CC recipient.
    pub fn cc(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        self.cc.push(match name {
            Some(n) => EmailAddress::with_name(email, n),
            None => EmailAddress::new(email),
        });
        self
    }

    /// Add a BCC recipient.
    pub fn bcc(mut self, email: impl Into<String>, name: Option<&str>) -> Self {
        self.bcc.push(match name {
            Some(n) => EmailAddress::with_name(email, n),
            None => EmailAddress::new(email),
        });
        self
    }

    /// Add an attachment.
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add a custom header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the template ID.
    pub fn template_id(mut self, id: impl Into<String>) -> Self {
        self.template_id = Some(id.into());
        self
    }

    /// Set the template data.
    pub fn template_data(mut self, data: serde_json::Value) -> Self {
        self.template_data = Some(data);
        self
    }

    /// Set the scheduled send time (Unix timestamp).
    pub fn send_time(mut self, timestamp: u64) -> Self {
        self.send_time = Some(timestamp);
        self
    }

    /// Add webhook data.
    pub fn webhook_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.webhook_data.insert(key.into(), value.into());
        self
    }

    /// Set the email tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set tracking settings.
    pub fn tracking(mut self, tracking: Tracking) -> Self {
        self.tracking = Some(tracking);
        self
    }

    /// Build the email.
    pub fn build(self) -> Result<Email> {
        let from = self
            .from
            .ok_or_else(|| LanefulError::ValidationError("from address is required".into()))?;

        let recipient_count = self.to.len() + self.cc.len() + self.bcc.len();

        if recipient_count == 0 {
            return Err(LanefulError::ValidationError(
                "at least one recipient (to, cc, or bcc) is required".into(),
            ));
        }

        if recipient_count > MAX_RECIPIENTS {
            return Err(LanefulError::ValidationError(
                "recipient limit exceeded (max 1000 across to/cc/bcc)".into(),
            ));
        }

        let subject = self
            .subject
            .ok_or_else(|| LanefulError::ValidationError("subject is required".into()))?;

        if self.text_content.is_none() && self.html_content.is_none() && self.template_id.is_none()
        {
            return Err(LanefulError::ValidationError(
                "either text_content, html_content, or template_id is required".into(),
            ));
        }

        if let Some(tag) = &self.tag {
            if tag.len() > MAX_TAG_LENGTH {
                return Err(LanefulError::ValidationError(
                    "tag length exceeds 100 characters".into(),
                ));
            }
        }

        if !self.webhook_data.is_empty() {
            if self.webhook_data.len() > MAX_WEBHOOK_DATA_KEYS {
                return Err(LanefulError::ValidationError(
                    "webhook_data exceeds 10 keys".into(),
                ));
            }

            for (key, value) in &self.webhook_data {
                if key.len() > MAX_WEBHOOK_DATA_KEY_LENGTH {
                    return Err(LanefulError::ValidationError(
                        "webhook_data key length exceeds 50 characters".into(),
                    ));
                }
                if value.len() > MAX_WEBHOOK_DATA_VALUE_LENGTH {
                    return Err(LanefulError::ValidationError(
                        "webhook_data value length exceeds 100 characters".into(),
                    ));
                }
            }
        }

        Ok(Email {
            from,
            to: self.to,
            subject,
            text_content: self.text_content,
            html_content: self.html_content,
            reply_to: self.reply_to,
            cc: if self.cc.is_empty() {
                None
            } else {
                Some(self.cc)
            },
            bcc: if self.bcc.is_empty() {
                None
            } else {
                Some(self.bcc)
            },
            attachments: if self.attachments.is_empty() {
                None
            } else {
                Some(self.attachments)
            },
            headers: if self.headers.is_empty() {
                None
            } else {
                Some(self.headers)
            },
            template_id: self.template_id,
            template_data: self.template_data,
            send_time: self.send_time,
            webhook_data: if self.webhook_data.is_empty() {
                None
            } else {
                Some(self.webhook_data)
            },
            tag: self.tag,
            tracking: self.tracking,
        })
    }
}

impl Email {
    /// Create a new email builder.
    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }
}
