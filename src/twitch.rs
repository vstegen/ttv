use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Duration;

const TWITCH_API_ENDPOINT: &str = "https://api.twitch.tv/helix";

#[derive(Debug, Deserialize)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
struct UsersResponse {
    data: Vec<TwitchUser>,
}

pub async fn fetch_users_by_login(
    client_id: &str,
    access_token: &str,
    logins: &[String],
) -> Result<Vec<TwitchUser>> {
    if logins.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build Twitch API client")?;

    let mut users = Vec::new();
    for batch in logins.chunks(100) {
        let url = build_users_url(batch)?;
        let res = client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header("Client-ID", client_id)
            .send()
            .await
            .context("failed to send Twitch request")?;

        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(map_api_error(status, body));
        }

        let response: UsersResponse = res
            .json()
            .await
            .context("failed to parse Twitch response")?;
        users.extend(response.data);
    }

    Ok(users)
}

fn build_users_url(logins: &[String]) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(&format!("{}/users", TWITCH_API_ENDPOINT))
        .context("failed to build Twitch users URL")?;
    {
        let mut pairs = url.query_pairs_mut();
        for login in logins {
            pairs.append_pair("login", login);
        }
    }
    Ok(url)
}

fn map_api_error(status: StatusCode, body: String) -> anyhow::Error {
    match status {
        StatusCode::UNAUTHORIZED => anyhow::anyhow!(
            "Unauthorized Twitch API request. Run `ttv auth` to refresh your token."
        ),
        StatusCode::FORBIDDEN => anyhow::anyhow!(
            "Forbidden Twitch API request. Check your client ID and token."
        ),
        StatusCode::TOO_MANY_REQUESTS => {
            anyhow::anyhow!("Twitch API rate limit exceeded. Try again later.")
        }
        _ => anyhow::anyhow!(
            "Unexpected Twitch API response ({}). {}",
            status,
            body
        ),
    }
}
