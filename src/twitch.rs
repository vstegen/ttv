use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;
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

#[derive(Debug, Deserialize)]
pub struct TwitchStream {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_name: String,
}

#[derive(Debug, Deserialize)]
struct StreamsResponse {
    data: Vec<TwitchStream>,
}

#[derive(Debug, Deserialize)]
pub struct TwitchVod {
    pub id: String,
    pub title: String,
    pub duration: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct VodsResponse {
    data: Vec<TwitchVod>,
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
        let response: UsersResponse = get_twitch(&client, client_id, access_token, url).await?;
        users.extend(response.data);
    }

    Ok(users)
}

pub async fn fetch_streams_by_user_ids(
    client_id: &str,
    access_token: &str,
    ids: &[String],
) -> Result<Vec<TwitchStream>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build Twitch API client")?;

    let mut streams = Vec::new();
    for batch in ids.chunks(100) {
        let url = build_streams_url(batch)?;
        let response: StreamsResponse = get_twitch(&client, client_id, access_token, url).await?;
        streams.extend(response.data);
    }

    Ok(streams)
}

pub async fn fetch_user_by_login(
    client_id: &str,
    access_token: &str,
    login: &str,
) -> Result<TwitchUser> {
    let users = fetch_users_by_login(client_id, access_token, &[login.to_string()]).await?;
    users
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No streamer found with login `{}`.", login))
}

pub async fn fetch_vods_by_user_id(
    client_id: &str,
    access_token: &str,
    user_id: &str,
) -> Result<Vec<TwitchVod>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build Twitch API client")?;

    let url = build_vods_url(user_id)?;
    let response: VodsResponse = get_twitch(&client, client_id, access_token, url).await?;
    Ok(response.data)
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

fn build_streams_url(ids: &[String]) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(&format!("{}/streams", TWITCH_API_ENDPOINT))
        .context("failed to build Twitch streams URL")?;
    {
        let mut pairs = url.query_pairs_mut();
        for id in ids {
            pairs.append_pair("user_id", id);
        }
    }
    Ok(url)
}

fn build_vods_url(user_id: &str) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(&format!("{}/videos", TWITCH_API_ENDPOINT))
        .context("failed to build Twitch videos URL")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("user_id", user_id);
        pairs.append_pair("type", "archive");
    }
    Ok(url)
}

async fn get_twitch<T>(
    client: &reqwest::Client,
    client_id: &str,
    access_token: &str,
    url: reqwest::Url,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let res = client
        .get(url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token),
        )
        .header("Client-ID", client_id)
        .send()
        .await
        .context("failed to send Twitch request")?;

    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(map_api_error(status, body));
    }

    let parsed = serde_json::from_str(&body).context("failed to parse Twitch response")?;
    Ok(parsed)
}

fn map_api_error(status: StatusCode, body: String) -> anyhow::Error {
    match status {
        StatusCode::UNAUTHORIZED => anyhow::anyhow!(
            "Unauthorized Twitch API request. Run `ttv auth` to refresh your token."
        ),
        StatusCode::FORBIDDEN => {
            anyhow::anyhow!("Forbidden Twitch API request. Check your client ID and token.")
        }
        StatusCode::TOO_MANY_REQUESTS => {
            anyhow::anyhow!("Twitch API rate limit exceeded. Try again later.")
        }
        _ => anyhow::anyhow!("Unexpected Twitch API response ({}). {}", status, body),
    }
}
