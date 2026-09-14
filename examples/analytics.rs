use laneful_rs::{
    LanefulClient, LanefulError, ListDomainSpamRatioRadarParams,
    ListGooglePostmasterSpamReportsParams, ListSndsReportsParams, Result,
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

    let client = LanefulClient::new(endpoint, api_key)?;
    let radar = client.list_domain_spam_ratio_radar(Some(&ListDomainSpamRatioRadarParams {
        start_date: Some("2026-09-01".into()),
        end_date: Some("2026-09-08".into()),
        ..Default::default()
    }))?;
    for entry in radar.radar {
        println!(
            "{} {} @{}: {}%",
            entry.date, entry.domain, entry.esp, entry.spam_ratio
        );
    }

    let postmaster = client.list_google_postmaster_spam_reports(Some(
        &ListGooglePostmasterSpamReportsParams {
            domain: Some("example.com".into()),
            ..Default::default()
        },
    ))?;
    for report in postmaster.spam_reports {
        println!("{} {}: {}%", report.date, report.domain, report.spam_ratio);
    }

    let snds = client.list_snds_reports(Some(&ListSndsReportsParams::default()))?;
    for report in snds.snds_reports {
        println!(
            "{} {}: filter={} complaint={}%",
            report.date, report.ip, report.filter_result, report.complaint_rate
        );
    }
    Ok(())
}
