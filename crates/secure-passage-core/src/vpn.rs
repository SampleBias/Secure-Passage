//! Lightweight VPN / public-IP check for the startup dialog.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IpResponse {
    ip: Option<String>,
    org: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
}

/// Returns (vpn_likely, detail). Best-effort heuristic using ipapi.co.
pub async fn check_vpn_status() -> (bool, String) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("Could not build HTTP client: {e}")),
    };

    match client.get("https://ipapi.co/json/").send().await {
        Ok(resp) => match resp.json::<IpResponse>().await {
            Ok(body) => {
                let ip = body.ip.unwrap_or_else(|| "unknown".into());
                let org = body.org.unwrap_or_else(|| "unknown".into()).to_lowercase();
                let host = body
                    .hostname
                    .unwrap_or_default()
                    .to_lowercase();
                let keywords = [
                    "vpn", "proxy", "hosting", "cloud", "datacenter", "digitalocean",
                    "amazon", "google", "microsoft", "ovh", "hetzner", "linode",
                    "mullvad", "nordvpn", "expressvpn", "proton",
                ];
                let hay = format!("{org} {host}");
                let detected = keywords.iter().any(|k| hay.contains(k));
                let detail = format!("Public IP: {ip} — Org: {org}");
                (detected, detail)
            }
            Err(e) => (false, format!("Could not parse IP lookup: {e}")),
        },
        Err(e) => (false, format!("IP lookup failed: {e}")),
    }
}
