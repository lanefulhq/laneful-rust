//! Laneful API client.

use crate::error::{LanefulError, Result};
use crate::models::{ApiErrorResponse, Email, MailSettings, SendEmailRequest, SendEmailResponse};
use crate::org::{
    CreateDomainRequest, Domain, ListDomainSpamRatioRadarParams, ListDomainSpamRatioRadarResponse,
    ListDomainsParams, ListDomainsResponse, ListGooglePostmasterSpamReportsParams,
    ListGooglePostmasterSpamReportsResponse, ListSndsReportsParams, ListSndsReportsResponse,
    ListUnsubscribeGroupsParams, ListUnsubscribeGroupsResponse, SuccessResponse, UnsubscribeGroup,
    UpdateDomainRequest, parse_domain, parse_list_domains, parse_unsubscribe_group,
};
use reqwest::Method;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Serialize;
use serde::de::DeserializeOwned;

const USER_AGENT_VALUE: &str = concat!("laneful-rust/", env!("CARGO_PKG_VERSION"));

/// Client for the Laneful Email API.
///
/// Email sending uses a send host (`https://your-endpoint.send.laneful.net`).
/// Domain, unsubscribe-group, and analytics endpoints use the organization
/// API host (`https://api.laneful.net`).
#[derive(Debug, Clone)]
pub struct LanefulClient {
    /// Base URL for API calls.
    base_url: String,
    /// API key for authentication.
    api_key: String,
    /// Blocking HTTP client (always available).
    blocking_client: reqwest::blocking::Client,
    /// Async HTTP client (available when async feature is enabled).
    #[cfg(feature = "async")]
    async_client: reqwest::Client,
}

fn default_headers(api_key: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|err| LanefulError::ConfigError(err.to_string()))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    Ok(headers)
}

fn encode_path(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

impl LanefulClient {
    /// Create a new Laneful client using a fully qualified base URL.
    ///
    /// The endpoint should be a full base URL like `https://custom-endpoint.send.laneful.net`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Full base URL for your organization
    /// * `api_key` - Your API key from account settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::LanefulClient;
    ///
    /// let client = LanefulClient::new("https://custom-endpoint.send.laneful.net", "my-api-key").unwrap();
    /// ```
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();

        if endpoint.is_empty() {
            return Err(LanefulError::ConfigError("endpoint cannot be empty".into()));
        }

        if !(endpoint.starts_with("https://") || endpoint.starts_with("http://")) {
            return Err(LanefulError::ConfigError(
                "endpoint must be a fully qualified URL (e.g., https://custom-endpoint.send.laneful.net)"
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
    /// * `base_url` - The full base URL (e.g., "https://api.laneful.net")
    /// * `api_key` - Your API key from account settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::LanefulClient;
    ///
    /// let client = LanefulClient::with_base_url(
    ///     "https://api.laneful.net",
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

        let headers = default_headers(&api_key)?;
        let blocking_client = reqwest::blocking::Client::builder()
            .default_headers(headers.clone())
            .build()?;

        #[cfg(feature = "async")]
        let async_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            base_url,
            api_key,
            blocking_client,
            #[cfg(feature = "async")]
            async_client,
        })
    }

    fn v1_url(&self, path: &str) -> String {
        format!("{}/v1/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn request_sync<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
        query: &[(String, String)],
    ) -> Result<T> {
        let mut request = self
            .blocking_client
            .request(method, self.v1_url(path))
            .header("Authorization", format!("Bearer {}", self.api_key));

        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        Self::handle_response_sync(request.send()?)
    }

    fn handle_response_sync<T: DeserializeOwned>(
        response: reqwest::blocking::Response,
    ) -> Result<T> {
        let status = response.status();
        let text = response.text()?;
        Self::parse_response(status, text)
    }

    fn parse_response<T: DeserializeOwned>(status: reqwest::StatusCode, text: String) -> Result<T> {
        if status.is_success() {
            if text.trim().is_empty() {
                return serde_json::from_str("{}").map_err(Into::into);
            }
            return serde_json::from_str(&text).map_err(Into::into);
        }

        let error_response: ApiErrorResponse =
            serde_json::from_str(&text).unwrap_or(ApiErrorResponse {
                error: format!("HTTP error: {status}"),
            });
        Err(LanefulError::ApiError(error_response.error))
    }

    // ==================== Sync API (always available) ====================

    /// Send multiple emails synchronously.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::{LanefulClient, Email};
    ///
    /// let client = LanefulClient::new("https://custom-endpoint.send.laneful.net", "my-api-key").unwrap();
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
        self.send_with_mail_settings(emails, None)
    }

    /// Send multiple emails with optional mail settings.
    pub fn send_with_mail_settings(
        &self,
        emails: Vec<Email>,
        mail_settings: Option<MailSettings>,
    ) -> Result<SendEmailResponse> {
        let request = SendEmailRequest {
            emails,
            mail_settings,
        };
        self.request_sync(Method::POST, "/email/send", Some(&request), &[])
    }

    /// Send a single email synchronously.
    ///
    /// This is a convenience method that wraps [`send`](Self::send).
    pub fn send_one(&self, email: Email) -> Result<SendEmailResponse> {
        self.send(vec![email])
    }

    /// Send a single email with optional mail settings.
    pub fn send_one_with_mail_settings(
        &self,
        email: Email,
        mail_settings: Option<MailSettings>,
    ) -> Result<SendEmailResponse> {
        self.send_with_mail_settings(vec![email], mail_settings)
    }

    /// List unsubscribe groups for a workspace.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn list_unsubscribe_groups(
        &self,
        workspace_id: u64,
        params: Option<&ListUnsubscribeGroupsParams>,
    ) -> Result<ListUnsubscribeGroupsResponse> {
        let query = params
            .map(ListUnsubscribeGroupsParams::to_query)
            .unwrap_or_default();
        self.request_sync(
            Method::GET,
            &format!("/workspaces/{workspace_id}/unsubscribe-groups"),
            None::<&()>,
            &query,
        )
    }

    /// Create an unsubscribe group in a workspace.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn create_unsubscribe_group(
        &self,
        workspace_id: u64,
        name: impl Into<String>,
    ) -> Result<UnsubscribeGroup> {
        let body = serde_json::json!({ "name": name.into() });
        let value: serde_json::Value = self.request_sync(
            Method::POST,
            &format!("/workspaces/{workspace_id}/unsubscribe-groups"),
            Some(&body),
            &[],
        )?;
        parse_unsubscribe_group(value)
    }

    /// Update an unsubscribe group.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn update_unsubscribe_group(
        &self,
        workspace_id: u64,
        unsubscribe_group_id: u64,
        name: impl Into<String>,
    ) -> Result<UnsubscribeGroup> {
        let body = serde_json::json!({ "name": name.into() });
        let value: serde_json::Value = self.request_sync(
            Method::PATCH,
            &format!("/workspaces/{workspace_id}/unsubscribe-groups/{unsubscribe_group_id}"),
            Some(&body),
            &[],
        )?;
        parse_unsubscribe_group(value)
    }

    /// List sending domains for a workspace.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn list_domains(
        &self,
        workspace_id: u64,
        params: Option<&ListDomainsParams>,
    ) -> Result<ListDomainsResponse> {
        let query = params.map(ListDomainsParams::to_query).unwrap_or_default();
        let value: serde_json::Value = self.request_sync(
            Method::GET,
            &format!("/workspaces/{workspace_id}/domains"),
            None::<&()>,
            &query,
        )?;
        parse_list_domains(value)
    }

    /// Get a single sending domain by name.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn get_domain(&self, workspace_id: u64, domain: &str) -> Result<Domain> {
        let value: serde_json::Value = self.request_sync(
            Method::GET,
            &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
            None::<&()>,
            &[],
        )?;
        parse_domain(value)
    }

    /// Create a sending domain in a workspace.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn create_domain(
        &self,
        workspace_id: u64,
        request: &CreateDomainRequest,
    ) -> Result<Domain> {
        let value: serde_json::Value = self.request_sync(
            Method::POST,
            &format!("/workspaces/{workspace_id}/domains"),
            Some(request),
            &[],
        )?;
        parse_domain(value)
    }

    /// Update a domain's mutable settings (currently the email track).
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn update_domain(
        &self,
        workspace_id: u64,
        domain: &str,
        request: &UpdateDomainRequest,
    ) -> Result<Domain> {
        let value: serde_json::Value = self.request_sync(
            Method::PATCH,
            &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
            Some(request),
            &[],
        )?;
        parse_domain(value)
    }

    /// Trigger DNS verification for a domain.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn verify_domain(&self, workspace_id: u64, domain: &str) -> Result<Domain> {
        let value: serde_json::Value = self.request_sync(
            Method::POST,
            &format!(
                "/workspaces/{workspace_id}/domains/{}/verify",
                encode_path(domain)
            ),
            None::<&()>,
            &[],
        )?;
        parse_domain(value)
    }

    /// Delete a sending domain from a workspace.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn delete_domain(&self, workspace_id: u64, domain: &str) -> Result<SuccessResponse> {
        self.request_sync(
            Method::DELETE,
            &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
            None::<&()>,
            &[],
        )
    }

    /// List domains whose spam complaint ratio reached a critical level.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn list_domain_spam_ratio_radar(
        &self,
        params: Option<&ListDomainSpamRatioRadarParams>,
    ) -> Result<ListDomainSpamRatioRadarResponse> {
        let query = params
            .map(ListDomainSpamRatioRadarParams::to_query)
            .unwrap_or_default();
        self.request_sync(
            Method::GET,
            "/analytics/radar/domain-spam-ratio",
            None::<&()>,
            &query,
        )
    }

    /// List daily Google Postmaster Tools spam-rate reports.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn list_google_postmaster_spam_reports(
        &self,
        params: Option<&ListGooglePostmasterSpamReportsParams>,
    ) -> Result<ListGooglePostmasterSpamReportsResponse> {
        let query = params
            .map(ListGooglePostmasterSpamReportsParams::to_query)
            .unwrap_or_default();
        self.request_sync(
            Method::GET,
            "/analytics/google-postmaster/spam-reports",
            None::<&()>,
            &query,
        )
    }

    /// List daily Microsoft SNDS reports for the organization's sending IPs.
    ///
    /// Uses the organization API host (`https://api.laneful.net`).
    pub fn list_snds_reports(
        &self,
        params: Option<&ListSndsReportsParams>,
    ) -> Result<ListSndsReportsResponse> {
        let query = params
            .map(ListSndsReportsParams::to_query)
            .unwrap_or_default();
        self.request_sync(
            Method::GET,
            "/analytics/microsoft-snds/reports",
            None::<&()>,
            &query,
        )
    }

    // ==================== Async API (feature-gated) ====================

    #[cfg(feature = "async")]
    async fn request_async<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
        query: &[(String, String)],
    ) -> Result<T> {
        let mut request = self
            .async_client
            .request(method, self.v1_url(path))
            .header("Authorization", format!("Bearer {}", self.api_key));

        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        Self::parse_response(status, text)
    }

    /// Send multiple emails asynchronously.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use laneful_rs::{LanefulClient, Email};
    ///
    /// # async fn example() {
    /// let client = LanefulClient::new("https://custom-endpoint.send.laneful.net", "my-api-key").unwrap();
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
        self.send_with_mail_settings_async(emails, None).await
    }

    /// Send multiple emails asynchronously with optional mail settings.
    #[cfg(feature = "async")]
    pub async fn send_with_mail_settings_async(
        &self,
        emails: Vec<Email>,
        mail_settings: Option<MailSettings>,
    ) -> Result<SendEmailResponse> {
        let request = SendEmailRequest {
            emails,
            mail_settings,
        };
        self.request_async(Method::POST, "/email/send", Some(&request), &[])
            .await
    }

    /// Send a single email asynchronously.
    ///
    /// This is a convenience method that wraps [`send_async`](Self::send_async).
    #[cfg(feature = "async")]
    pub async fn send_one_async(&self, email: Email) -> Result<SendEmailResponse> {
        self.send_async(vec![email]).await
    }

    /// Send a single email asynchronously with optional mail settings.
    #[cfg(feature = "async")]
    pub async fn send_one_with_mail_settings_async(
        &self,
        email: Email,
        mail_settings: Option<MailSettings>,
    ) -> Result<SendEmailResponse> {
        self.send_with_mail_settings_async(vec![email], mail_settings)
            .await
    }

    /// List unsubscribe groups for a workspace asynchronously.
    #[cfg(feature = "async")]
    pub async fn list_unsubscribe_groups_async(
        &self,
        workspace_id: u64,
        params: Option<&ListUnsubscribeGroupsParams>,
    ) -> Result<ListUnsubscribeGroupsResponse> {
        let query = params
            .map(ListUnsubscribeGroupsParams::to_query)
            .unwrap_or_default();
        self.request_async(
            Method::GET,
            &format!("/workspaces/{workspace_id}/unsubscribe-groups"),
            None::<&()>,
            &query,
        )
        .await
    }

    /// Create an unsubscribe group in a workspace asynchronously.
    #[cfg(feature = "async")]
    pub async fn create_unsubscribe_group_async(
        &self,
        workspace_id: u64,
        name: impl Into<String>,
    ) -> Result<UnsubscribeGroup> {
        let body = serde_json::json!({ "name": name.into() });
        let value: serde_json::Value = self
            .request_async(
                Method::POST,
                &format!("/workspaces/{workspace_id}/unsubscribe-groups"),
                Some(&body),
                &[],
            )
            .await?;
        parse_unsubscribe_group(value)
    }

    /// Update an unsubscribe group asynchronously.
    #[cfg(feature = "async")]
    pub async fn update_unsubscribe_group_async(
        &self,
        workspace_id: u64,
        unsubscribe_group_id: u64,
        name: impl Into<String>,
    ) -> Result<UnsubscribeGroup> {
        let body = serde_json::json!({ "name": name.into() });
        let value: serde_json::Value = self
            .request_async(
                Method::PATCH,
                &format!("/workspaces/{workspace_id}/unsubscribe-groups/{unsubscribe_group_id}"),
                Some(&body),
                &[],
            )
            .await?;
        parse_unsubscribe_group(value)
    }

    /// List sending domains for a workspace asynchronously.
    #[cfg(feature = "async")]
    pub async fn list_domains_async(
        &self,
        workspace_id: u64,
        params: Option<&ListDomainsParams>,
    ) -> Result<ListDomainsResponse> {
        let query = params.map(ListDomainsParams::to_query).unwrap_or_default();
        let value: serde_json::Value = self
            .request_async(
                Method::GET,
                &format!("/workspaces/{workspace_id}/domains"),
                None::<&()>,
                &query,
            )
            .await?;
        parse_list_domains(value)
    }

    /// Get a single sending domain by name asynchronously.
    #[cfg(feature = "async")]
    pub async fn get_domain_async(&self, workspace_id: u64, domain: &str) -> Result<Domain> {
        let value: serde_json::Value = self
            .request_async(
                Method::GET,
                &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
                None::<&()>,
                &[],
            )
            .await?;
        parse_domain(value)
    }

    /// Create a sending domain in a workspace asynchronously.
    #[cfg(feature = "async")]
    pub async fn create_domain_async(
        &self,
        workspace_id: u64,
        request: &CreateDomainRequest,
    ) -> Result<Domain> {
        let value: serde_json::Value = self
            .request_async(
                Method::POST,
                &format!("/workspaces/{workspace_id}/domains"),
                Some(request),
                &[],
            )
            .await?;
        parse_domain(value)
    }

    /// Update a domain's email track asynchronously.
    #[cfg(feature = "async")]
    pub async fn update_domain_async(
        &self,
        workspace_id: u64,
        domain: &str,
        request: &UpdateDomainRequest,
    ) -> Result<Domain> {
        let value: serde_json::Value = self
            .request_async(
                Method::PATCH,
                &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
                Some(request),
                &[],
            )
            .await?;
        parse_domain(value)
    }

    /// Trigger DNS verification for a domain asynchronously.
    #[cfg(feature = "async")]
    pub async fn verify_domain_async(&self, workspace_id: u64, domain: &str) -> Result<Domain> {
        let value: serde_json::Value = self
            .request_async(
                Method::POST,
                &format!(
                    "/workspaces/{workspace_id}/domains/{}/verify",
                    encode_path(domain)
                ),
                None::<&()>,
                &[],
            )
            .await?;
        parse_domain(value)
    }

    /// Delete a sending domain from a workspace asynchronously.
    #[cfg(feature = "async")]
    pub async fn delete_domain_async(
        &self,
        workspace_id: u64,
        domain: &str,
    ) -> Result<SuccessResponse> {
        self.request_async(
            Method::DELETE,
            &format!("/workspaces/{workspace_id}/domains/{}", encode_path(domain)),
            None::<&()>,
            &[],
        )
        .await
    }

    /// List domain spam-ratio radar entries asynchronously.
    #[cfg(feature = "async")]
    pub async fn list_domain_spam_ratio_radar_async(
        &self,
        params: Option<&ListDomainSpamRatioRadarParams>,
    ) -> Result<ListDomainSpamRatioRadarResponse> {
        let query = params
            .map(ListDomainSpamRatioRadarParams::to_query)
            .unwrap_or_default();
        self.request_async(
            Method::GET,
            "/analytics/radar/domain-spam-ratio",
            None::<&()>,
            &query,
        )
        .await
    }

    /// List Google Postmaster spam reports asynchronously.
    #[cfg(feature = "async")]
    pub async fn list_google_postmaster_spam_reports_async(
        &self,
        params: Option<&ListGooglePostmasterSpamReportsParams>,
    ) -> Result<ListGooglePostmasterSpamReportsResponse> {
        let query = params
            .map(ListGooglePostmasterSpamReportsParams::to_query)
            .unwrap_or_default();
        self.request_async(
            Method::GET,
            "/analytics/google-postmaster/spam-reports",
            None::<&()>,
            &query,
        )
        .await
    }

    /// List Microsoft SNDS reports asynchronously.
    #[cfg(feature = "async")]
    pub async fn list_snds_reports_async(
        &self,
        params: Option<&ListSndsReportsParams>,
    ) -> Result<ListSndsReportsResponse> {
        let query = params
            .map(ListSndsReportsParams::to_query)
            .unwrap_or_default();
        self.request_async(
            Method::GET,
            "/analytics/microsoft-snds/reports",
            None::<&()>,
            &query,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;

    fn sample_email() -> Email {
        Email::builder()
            .from("sender@test.com", Some("Sender"))
            .to("recipient@test.com", Some("Recipient"))
            .subject("Test")
            .text_content("Hello")
            .build()
            .unwrap()
    }

    #[test]
    fn send_includes_mail_settings_and_message_ids() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/email/send")
            .match_header("user-agent", USER_AGENT_VALUE)
            .match_body(Matcher::PartialJsonString(
                r#"{"mail_settings":{"sandbox_mode":true,"return_message_ids":true}}"#.into(),
            ))
            .with_status(200)
            .with_body(r#"{"status":"accepted","message_ids":["msg_123"]}"#)
            .create();

        let client = LanefulClient::new(server.url(), "test-token").unwrap();
        let response = client
            .send_one_with_mail_settings(
                sample_email(),
                Some(MailSettings {
                    sandbox_mode: Some(true),
                    return_message_ids: Some(true),
                }),
            )
            .unwrap();

        mock.assert();
        assert_eq!(response.status, "accepted");
        assert_eq!(
            response.message_ids.as_deref(),
            Some(&["msg_123".to_string()][..])
        );
        assert_eq!(response.first_message_id(), Some("msg_123"));
    }

    #[test]
    fn list_unsubscribe_groups_parses_response() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/workspaces/42/unsubscribe-groups")
            .with_status(200)
            .with_body(r#"{"unsubscribe_groups":[{"unsubscribe_group_id":9,"name":"Newsletters","created_at":1}]}"#)
            .create();

        let client = LanefulClient::new(server.url(), "test-token").unwrap();
        let result = client.list_unsubscribe_groups(42, None).unwrap();
        mock.assert();
        assert_eq!(result.unsubscribe_groups[0].name, "Newsletters");
    }

    #[test]
    fn list_domains_sends_filter_query() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/workspaces/42/domains")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("limit".into(), "25".into()),
                Matcher::UrlEncoded("filter[domain]".into(), "example.com".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"domains":[{"domain":"example.com","verified":true}],"pagination":{"next_cursor":"next"}}"#)
            .create();

        let client = LanefulClient::new(server.url(), "test-token").unwrap();
        let result = client
            .list_domains(
                42,
                Some(&ListDomainsParams {
                    limit: Some(25),
                    filter_domain: Some("example.com".into()),
                    ..Default::default()
                }),
            )
            .unwrap();
        mock.assert();
        assert_eq!(result.domains[0].domain, "example.com");
        assert_eq!(result.next_cursor.as_deref(), Some("next"));
    }

    #[test]
    fn domain_mutations_hit_encoded_path() {
        let mut server = mockito::Server::new();
        let get = server
            .mock("GET", "/v1/workspaces/42/domains/example.com")
            .with_status(200)
            .with_body(r#"{"domain":{"domain":"example.com","verified":true}}"#)
            .create();
        let patch = server
            .mock("PATCH", "/v1/workspaces/42/domains/example.com")
            .match_body(r#"{"email_track_id":""}"#)
            .with_status(200)
            .with_body(r#"{"domain":"example.com","verified":true}"#)
            .create();
        let verify = server
            .mock("POST", "/v1/workspaces/42/domains/example.com/verify")
            .with_status(200)
            .with_body(r#"{"domain":"example.com","verified":true}"#)
            .create();
        let delete = server
            .mock("DELETE", "/v1/workspaces/42/domains/example.com")
            .with_status(200)
            .with_body(r#"{"message":"deleted"}"#)
            .create();

        let client = LanefulClient::new(server.url(), "test-token").unwrap();
        let domain = client.get_domain(42, "example.com").unwrap();
        assert_eq!(domain.domain, "example.com");
        client
            .update_domain(42, "example.com", &UpdateDomainRequest::clear())
            .unwrap();
        client.verify_domain(42, "example.com").unwrap();
        let deleted = client.delete_domain(42, "example.com").unwrap();
        assert_eq!(deleted.message, "deleted");

        get.assert();
        patch.assert();
        verify.assert();
        delete.assert();
    }

    #[test]
    fn analytics_repeats_workspace_ids() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/analytics/radar/domain-spam-ratio")
            .match_query(Matcher::Regex("workspace_ids=1&workspace_ids=2".into()))
            .with_status(200)
            .with_body(r#"{"radar":[]}"#)
            .create();

        let client = LanefulClient::new(server.url(), "test-token").unwrap();
        client
            .list_domain_spam_ratio_radar(Some(&ListDomainSpamRatioRadarParams {
                workspace_ids: vec![1, 2],
                ..Default::default()
            }))
            .unwrap();
        mock.assert();
    }

    #[test]
    fn rejects_empty_config() {
        assert!(LanefulClient::new("", "token").is_err());
        assert!(LanefulClient::new("https://test.laneful.net", "").is_err());
    }
}
