use laneful_rs::{
    CreateDomainRequest, LanefulClient, LanefulError, ListDomainsParams, Result,
    UpdateDomainRequest,
};

fn env_var(name: &str) -> Result<String> {
    std::env::var(name)
        .map_err(|_| LanefulError::ConfigError(format!("{name} is required (set it in your env)")))
}

fn main() -> Result<()> {
    let endpoint = std::env::var("LANEFUL_ORG_BASE_URL")
        .or_else(|_| std::env::var("LANEFUL_ENDPOINT"))
        .unwrap_or_else(|_| "https://api.laneful.net".into());
    let api_key = env_var("LANEFUL_API_KEY")?;
    let workspace_id = env_var("LANEFUL_WORKSPACE_ID")?
        .parse::<u64>()
        .map_err(|_| LanefulError::ConfigError("LANEFUL_WORKSPACE_ID must be a number".into()))?;

    let client = LanefulClient::new(endpoint, api_key)?;
    let listing = client.list_domains(
        workspace_id,
        Some(&ListDomainsParams {
            limit: Some(50),
            ..Default::default()
        }),
    )?;
    println!("Domains: {}", listing.domains.len());

    let domain = client.create_domain(
        workspace_id,
        &CreateDomainRequest {
            domain: "mydomain.com".into(),
            tracking: "tracking".into(),
            return_path: "return-path".into(),
            require_tls: None,
            email_track_id: None,
        },
    )?;
    println!("Created {}, verified={}", domain.domain, domain.verified);

    let verified = client.verify_domain(workspace_id, "mydomain.com")?;
    println!("Verification: dmarc={}", verified.dmarc_verified);

    let updated = client.update_domain(
        workspace_id,
        "mydomain.com",
        &UpdateDomainRequest::set("e59f0a35-05bc-4516-b585-c06f69c3e67e"),
    )?;
    println!("Email track: {}", updated.email_track_id);
    Ok(())
}
