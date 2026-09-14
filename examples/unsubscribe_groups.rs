use laneful_rs::{LanefulClient, LanefulError, ListUnsubscribeGroupsParams, Result};

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
    let created = client.create_unsubscribe_group(workspace_id, "Newsletters")?;
    println!(
        "Created group {}: {}",
        created.unsubscribe_group_id, created.name
    );

    let updated = client.update_unsubscribe_group(
        workspace_id,
        created.unsubscribe_group_id,
        "Weekly Newsletters",
    )?;
    println!("Updated name: {}", updated.name);

    let listing = client.list_unsubscribe_groups(
        workspace_id,
        Some(&ListUnsubscribeGroupsParams {
            limit: Some(50),
            ..Default::default()
        }),
    )?;
    for group in listing.unsubscribe_groups {
        println!("- {} {}", group.unsubscribe_group_id, group.name);
    }
    Ok(())
}
