//! Laneful API client.

use crate::error::{LanefulError, Result};
use crate::models::{ApiErrorResponse, Email, SendEmailRequest, SendEmailResponse};
#[cfg(feature = "async")]
use std::sync::OnceLock;

/// Client for the Laneful Email API.
#[derive(Debug, Clone)]
pub struct LanefulClient {
    /// Base URL for API calls.
    base_url: String,
    /// API key for authentication.
    api_key: String,
    /// Blocking HTTP client (always available).
    #[cfg(feature = "async")]
    blocking_client: OnceLock<reqwest::blocking::Client>,
    #[cfg(not(feature = "async"))]
    blocking_client: reqwest::blocking::Client,
    /// Async HTTP client (available when async feature is enabled).
    #[cfg(feature = "async")]
    async_client: reqwest::Client,
}

impl LanefulClient {
    /// Create a new Laneful client using a fully qualified base URL.
    ///
    /// The endpoint should be a full base URL like `https://custom-endpoint.api.laneful.com`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Full base URL for your organization (e.g., "https://custom-endpoint.api.laneful.com")
    /// * `api_key` - Your API key from account settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::LanefulClient;
    ///
    /// let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key").unwrap();
    /// ```
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();

        if endpoint.is_empty() {
            return Err(LanefulError::ConfigError("endpoint cannot be empty".into()));
        }

        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err(LanefulError::ConfigError(
                "endpoint must be a fully qualified URL (e.g., https://custom-endpoint.api.laneful.com)"
                    .into(),
            ));
        }

        Self::with_base_url(endpoint, api_key)
    }

    /// Create a new Laneful client with a custom base URL.
    ///
    /// Use this when you need to connect to a custom API endpoint.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The full base URL (e.g., "https://custom.api.example.com")
    /// * `api_key` - Your API key from account settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::LanefulClient;
    ///
    /// let client = LanefulClient::with_base_url(
    ///     "https://custom.api.laneful.com",
    ///     "my-api-key"
    /// ).unwrap();
    /// ```
    pub fn with_base_url(base_url: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let api_key = api_key.into();

        if base_url.is_empty() {
            return Err(LanefulError::ConfigError("base_url cannot be empty".into()));
        }

        if api_key.is_empty() {
            return Err(LanefulError::ConfigError("api_key cannot be empty".into()));
        }

        #[cfg(feature = "async")]
        let blocking_client = OnceLock::new();
        #[cfg(not(feature = "async"))]
        let blocking_client = reqwest::blocking::Client::new();

        #[cfg(feature = "async")]
        let async_client = reqwest::Client::new();

        Ok(Self {
            base_url,
            api_key,
            blocking_client,
            #[cfg(feature = "async")]
            async_client,
        })
    }

    /// Get the API URL for the send endpoint.
    fn api_url(&self) -> String {
        format!("{}/v1/email/send", self.base_url)
    }

    #[cfg(feature = "async")]
    fn blocking_client(&self) -> &reqwest::blocking::Client {
        self.blocking_client
            .get_or_init(reqwest::blocking::Client::new)
    }

    #[cfg(not(feature = "async"))]
    fn blocking_client(&self) -> &reqwest::blocking::Client {
        &self.blocking_client
    }

    // ==================== Sync API (always available) ====================

    /// Send multiple emails synchronously.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::{LanefulClient, Email};
    ///
    /// let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key").unwrap();
    /// let email = Email::builder()
    ///     .from("sender@example.com", None)
    ///     .to("recipient@example.com", None)
    ///     .subject("Hello")
    ///     .text_content("Hello, world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let response = client.send(vec![email]).unwrap();
    /// ```
    pub fn send(&self, emails: Vec<Email>) -> Result<SendEmailResponse> {
        let request = SendEmailRequest { emails };

        let response = self
            .blocking_client()
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()?;

        self.handle_response_sync(response)
    }

    /// Send a single email synchronously.
    ///
    /// This is a convenience method that wraps [`send`](Self::send).
    pub fn send_one(&self, email: Email) -> Result<SendEmailResponse> {
        self.send(vec![email])
    }

    /// Handle the HTTP response for sync calls.
    fn handle_response_sync(
        &self,
        response: reqwest::blocking::Response,
    ) -> Result<SendEmailResponse> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json()?)
        } else {
            let error_response: ApiErrorResponse = response.json().unwrap_or(ApiErrorResponse {
                error: format!("HTTP error: {}", status),
            });
            Err(LanefulError::ApiError(error_response.error))
        }
    }

    // ==================== Async API (feature-gated) ====================

    /// Send multiple emails asynchronously.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::{LanefulClient, Email};
    ///
    /// # async fn example() {
    /// let client = LanefulClient::new("https://custom-endpoint.api.laneful.com", "my-api-key").unwrap();
    /// let email = Email::builder()
    ///     .from("sender@example.com", None)
    ///     .to("recipient@example.com", None)
    ///     .subject("Hello")
    ///     .text_content("Hello, world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let response = client.send_async(vec![email]).await.unwrap();
    /// # }
    /// ```
    #[cfg(feature = "async")]
    pub async fn send_async(&self, emails: Vec<Email>) -> Result<SendEmailResponse> {
        let request = SendEmailRequest { emails };

        let response = self
            .async_client
            .post(self.api_url())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        self.handle_response_async(response).await
    }

    /// Send a single email asynchronously.
    ///
    /// This is a convenience method that wraps [`send_async`](Self::send_async).
    #[cfg(feature = "async")]
    pub async fn send_one_async(&self, email: Email) -> Result<SendEmailResponse> {
        self.send_async(vec![email]).await
    }

    /// Handle the HTTP response for async calls.
    #[cfg(feature = "async")]
    async fn handle_response_async(
        &self,
        response: reqwest::Response,
    ) -> Result<SendEmailResponse> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_response: ApiErrorResponse =
                response.json().await.unwrap_or(ApiErrorResponse {
                    error: format!("HTTP error: {}", status),
                });
            Err(LanefulError::ApiError(error_response.error))
        }
    }
}
