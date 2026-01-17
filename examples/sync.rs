use laneful_rs::{Email, LanefulClient, LanefulError, Result};

fn env_var(name: &str) -> Result<String> {
    std::env::var(name)
        .map_err(|_| LanefulError::ConfigError(format!("{name} is required (set it in your env)")))
}

fn arg_var(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == name {
            return args.next();
        }
    }
    None
}

fn required_arg(name: &str) -> Result<String> {
    arg_var(name).ok_or_else(|| {
        LanefulError::ConfigError(format!("{name} is required (pass it as an argument)"))
    })
}

fn main() -> Result<()> {
    let endpoint = env_var("LANEFUL_ENDPOINT")?;
    let api_key = env_var("LANEFUL_API_KEY")?;
    let from = required_arg("--from")?;
    let to = required_arg("--to")?;

    let client = LanefulClient::new(endpoint, api_key)?;

    let email = Email::builder()
        .from(from, Some("Sender"))
        .to(to, Some("Recipient"))
        .subject("Hello from Laneful (sync)")
        .text_content("This is a sync example.")
        .build()?;

    let response = client.send_one(email)?;
    println!("Sent: {:?}", response);

    Ok(())
}
