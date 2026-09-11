use laneful_rs::{Email, LanefulClient, LanefulError, MailSettings, Result, Tracking};

fn env_var(name: &str) -> Result<String> {
    std::env::var(name)
        .map_err(|_| LanefulError::ConfigError(format!("{name} is required (set it in your env)")))
}

fn main() -> Result<()> {
    let endpoint = env_var("LANEFUL_ENDPOINT")?;
    let api_key = env_var("LANEFUL_API_KEY")?;
    let from = env_var("LANEFUL_FROM_EMAIL").or_else(|_| env_var("LANEFUL_FROM"))?;
    let to = env_var("LANEFUL_TO_EMAIL").or_else(|_| env_var("LANEFUL_TO"))?;

    let client = LanefulClient::new(endpoint, api_key)?;
    let email = Email::builder()
        .from(&from, Some("Your Name"))
        .to(&to, Some("Recipient Name"))
        .subject("Sandbox email")
        .text_content("This email is sent with sandbox mode and returns message IDs.")
        .from_header(&from, Some("Newsletter"))
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
    println!("Status: {}", response.status);
    println!("Message IDs: {:?}", response.message_ids);
    Ok(())
}
