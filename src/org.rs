//! Organization API models (domains, unsubscribe groups, analytics).

use serde::{Deserialize, Serialize};

/// Query string items (repeated keys supported).
pub type QueryItems = Vec<(String, String)>;

fn query_value(key: &str, value: Option<&str>) -> QueryItems {
    match value {
        Some(v) if !v.is_empty() => vec![(key.to_string(), v.to_string())],
        _ => vec![],
    }
}

fn query_limit(limit: Option<u32>) -> QueryItems {
    match limit {
        Some(limit) if limit > 0 => vec![("limit".to_string(), limit.to_string())],
        _ => vec![],
    }
}

/// An unsubscribe group in a workspace.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UnsubscribeGroup {
    pub unsubscribe_group_id: u64,
    pub name: String,
    #[serde(default)]
    pub created_at: u64,
}

/// Paginated list of unsubscribe groups.
#[derive(Debug, Clone, Deserialize)]
pub struct ListUnsubscribeGroupsResponse {
    #[serde(default)]
    pub unsubscribe_groups: Vec<UnsubscribeGroup>,
    pub next_cursor: Option<String>,
}

/// Query parameters for listing unsubscribe groups.
#[derive(Debug, Clone, Default)]
pub struct ListUnsubscribeGroupsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub search: Option<String>,
}

impl ListUnsubscribeGroupsParams {
    /// Build query string items.
    pub fn to_query(&self) -> QueryItems {
        let mut items = query_value("cursor", self.cursor.as_deref());
        items.extend(query_limit(self.limit));
        items.extend(query_value("search", self.search.as_deref()));
        items
    }
}

/// A sending domain and its verification state.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Domain {
    pub domain: String,
    #[serde(default)]
    pub tracking: String,
    #[serde(default)]
    pub return_path: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub tracking_verified: bool,
    #[serde(default)]
    pub return_path_verified: bool,
    #[serde(default)]
    pub dkim1_verified: bool,
    #[serde(default)]
    pub dkim2_verified: bool,
    #[serde(default)]
    pub dmarc_verified: bool,
    #[serde(default)]
    pub require_tls: bool,
    #[serde(default)]
    pub email_track_id: String,
}

/// Paginated list of sending domains.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDomainsResponse {
    #[serde(default)]
    pub domains: Vec<Domain>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ListDomainsRaw {
    #[serde(default)]
    domains: Vec<serde_json::Value>,
    pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Deserialize)]
struct Pagination {
    next_cursor: Option<String>,
}

/// Query parameters for listing domains.
#[derive(Debug, Clone, Default)]
pub struct ListDomainsParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub filter_domain: Option<String>,
}

impl ListDomainsParams {
    /// Build query string items.
    pub fn to_query(&self) -> QueryItems {
        let mut items = query_value("cursor", self.cursor.as_deref());
        items.extend(query_limit(self.limit));
        items.extend(query_value("filter[domain]", self.filter_domain.as_deref()));
        items
    }
}

/// Request body for creating a sending domain.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDomainRequest {
    pub domain: String,
    pub tracking: String,
    pub return_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_tls: Option<bool>,
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub email_track_id: Option<String>,
}

fn is_none_or_empty(value: &Option<String>) -> bool {
    value.as_ref().map(|s| s.is_empty()).unwrap_or(true)
}

/// Request body for updating a domain's email track.
///
/// Pass a track ID to set the email track, an empty string to clear it,
/// or `None` to leave it unchanged.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateDomainRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_track_id: Option<String>,
}

impl UpdateDomainRequest {
    /// Leave the email track unchanged.
    pub fn unchanged() -> Self {
        Self {
            email_track_id: None,
        }
    }

    /// Clear the email track.
    pub fn clear() -> Self {
        Self {
            email_track_id: Some(String::new()),
        }
    }

    /// Set the email track to the given ID.
    pub fn set(email_track_id: impl Into<String>) -> Self {
        Self {
            email_track_id: Some(email_track_id.into()),
        }
    }
}

/// Generic success message from mutating endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct SuccessResponse {
    #[serde(default)]
    pub message: String,
}

/// A sending domain whose spam complaint ratio reached a critical level.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainSpamRatioRadar {
    #[serde(default)]
    pub workspace_id: u64,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub esp: String,
    #[serde(default)]
    pub spam_ratio: f64,
    #[serde(default)]
    pub date: String,
}

/// Paginated list of domain spam-ratio radar entries.
#[derive(Debug, Clone, Deserialize)]
pub struct ListDomainSpamRatioRadarResponse {
    #[serde(default)]
    pub radar: Vec<DomainSpamRatioRadar>,
    pub next_cursor: Option<String>,
}

/// Query parameters for listing domain spam-ratio radar entries.
#[derive(Debug, Clone, Default)]
pub struct ListDomainSpamRatioRadarParams {
    pub workspace_ids: Vec<u64>,
    pub domain: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

impl ListDomainSpamRatioRadarParams {
    /// Build query string items (repeated `workspace_ids` keys).
    pub fn to_query(&self) -> QueryItems {
        let mut items: QueryItems = self
            .workspace_ids
            .iter()
            .map(|id| ("workspace_ids".to_string(), id.to_string()))
            .collect();
        items.extend(query_value("domain", self.domain.as_deref()));
        items.extend(query_value("start_date", self.start_date.as_deref()));
        items.extend(query_value("end_date", self.end_date.as_deref()));
        items.extend(query_value("cursor", self.cursor.as_deref()));
        items.extend(query_limit(self.limit));
        items
    }
}

/// A daily Gmail spam-rate report from Google Postmaster Tools.
#[derive(Debug, Clone, Deserialize)]
pub struct GooglePostmasterSpamReport {
    #[serde(default)]
    pub workspace_id: u64,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub spam_ratio: f64,
}

/// Paginated list of Google Postmaster spam reports.
#[derive(Debug, Clone, Deserialize)]
pub struct ListGooglePostmasterSpamReportsResponse {
    #[serde(default)]
    pub spam_reports: Vec<GooglePostmasterSpamReport>,
    pub next_cursor: Option<String>,
}

/// Query parameters for listing Google Postmaster spam reports.
#[derive(Debug, Clone, Default)]
pub struct ListGooglePostmasterSpamReportsParams {
    pub workspace_ids: Vec<u64>,
    pub domain: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

impl ListGooglePostmasterSpamReportsParams {
    /// Build query string items (repeated `workspace_ids` keys).
    pub fn to_query(&self) -> QueryItems {
        let mut items: QueryItems = self
            .workspace_ids
            .iter()
            .map(|id| ("workspace_ids".to_string(), id.to_string()))
            .collect();
        items.extend(query_value("domain", self.domain.as_deref()));
        items.extend(query_value("start_date", self.start_date.as_deref()));
        items.extend(query_value("end_date", self.end_date.as_deref()));
        items.extend(query_value("cursor", self.cursor.as_deref()));
        items.extend(query_limit(self.limit));
        items
    }
}

/// Unknown SNDS filter result.
pub const SNDS_FILTER_UNKNOWN: &str = "";
/// Green SNDS filter result.
pub const SNDS_FILTER_GREEN: &str = "GREEN";
/// Yellow SNDS filter result.
pub const SNDS_FILTER_YELLOW: &str = "YELLOW";
/// Red SNDS filter result.
pub const SNDS_FILTER_RED: &str = "RED";

/// A daily Microsoft SNDS report for a sending IP.
#[derive(Debug, Clone, Deserialize)]
pub struct SndsReport {
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub rcpt_commands: u64,
    #[serde(default)]
    pub data_commands: u64,
    #[serde(default)]
    pub message_recipients: u64,
    #[serde(default)]
    pub filter_result: String,
    #[serde(default)]
    pub complaint_rate: f64,
    #[serde(default)]
    pub trap_hits: u64,
}

/// Paginated list of Microsoft SNDS reports.
#[derive(Debug, Clone, Deserialize)]
pub struct ListSndsReportsResponse {
    #[serde(default)]
    pub snds_reports: Vec<SndsReport>,
    pub next_cursor: Option<String>,
}

/// Query parameters for listing Microsoft SNDS reports.
#[derive(Debug, Clone, Default)]
pub struct ListSndsReportsParams {
    pub ip: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

impl ListSndsReportsParams {
    /// Build query string items.
    pub fn to_query(&self) -> QueryItems {
        let mut items = query_value("ip", self.ip.as_deref());
        items.extend(query_value("start_date", self.start_date.as_deref()));
        items.extend(query_value("end_date", self.end_date.as_deref()));
        items.extend(query_value("cursor", self.cursor.as_deref()));
        items.extend(query_limit(self.limit));
        items
    }
}

pub(crate) fn parse_unsubscribe_group(
    value: serde_json::Value,
) -> crate::error::Result<UnsubscribeGroup> {
    let payload = match value.get("unsubscribe_group") {
        Some(inner) if inner.is_object() => inner.clone(),
        _ => value,
    };
    Ok(serde_json::from_value(payload)?)
}

pub(crate) fn parse_domain(value: serde_json::Value) -> crate::error::Result<Domain> {
    let payload = match value.get("domain") {
        Some(inner) if inner.is_object() => inner.clone(),
        _ => value,
    };
    Ok(serde_json::from_value(payload)?)
}

pub(crate) fn parse_list_domains(
    value: serde_json::Value,
) -> crate::error::Result<ListDomainsResponse> {
    let raw: ListDomainsRaw = serde_json::from_value(value)?;
    let mut domains = Vec::with_capacity(raw.domains.len());
    for item in raw.domains {
        domains.push(parse_domain(item)?);
    }
    Ok(ListDomainsResponse {
        next_cursor: raw.pagination.and_then(|p| p.next_cursor),
        domains,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_domain_three_way() {
        assert_eq!(
            serde_json::to_value(UpdateDomainRequest::unchanged()).unwrap(),
            serde_json::json!({})
        );
        assert_eq!(
            serde_json::to_value(UpdateDomainRequest::clear()).unwrap(),
            serde_json::json!({"email_track_id": ""})
        );
        assert_eq!(
            serde_json::to_value(UpdateDomainRequest::set("track-id")).unwrap(),
            serde_json::json!({"email_track_id": "track-id"})
        );
    }

    #[test]
    fn create_domain_omits_empty_track() {
        let request = CreateDomainRequest {
            domain: "example.com".into(),
            tracking: "tracking".into(),
            return_path: "return-path".into(),
            require_tls: Some(true),
            email_track_id: None,
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            serde_json::json!({
                "domain": "example.com",
                "tracking": "tracking",
                "return_path": "return-path",
                "require_tls": true,
            })
        );
    }

    #[test]
    fn list_domains_filter_query() {
        let params = ListDomainsParams {
            cursor: Some("abc".into()),
            limit: Some(10),
            filter_domain: Some("ex.com".into()),
        };
        assert_eq!(
            params.to_query(),
            vec![
                ("cursor".into(), "abc".into()),
                ("limit".into(), "10".into()),
                ("filter[domain]".into(), "ex.com".into()),
            ]
        );
    }

    #[test]
    fn radar_repeats_workspace_ids() {
        let params = ListDomainSpamRatioRadarParams {
            workspace_ids: vec![1, 2],
            domain: Some("example.com".into()),
            start_date: Some("2026-09-01".into()),
            end_date: Some("2026-09-08".into()),
            ..Default::default()
        };
        let query = params.to_query();
        assert_eq!(
            &query[..2],
            &[
                ("workspace_ids".into(), "1".into()),
                ("workspace_ids".into(), "2".into()),
            ]
        );
        assert!(query.contains(&("domain".into(), "example.com".into())));
    }

    #[test]
    fn domain_from_wrapped_payload() {
        let domain = parse_domain(serde_json::json!({
            "domain": { "domain": "example.com", "verified": true }
        }))
        .unwrap();
        assert_eq!(domain.domain, "example.com");
        assert!(domain.verified);
    }
}
