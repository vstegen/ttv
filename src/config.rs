use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub twitch: TwitchConfig,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TwitchConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Args)]
#[command(about = "Manage Twitch API credentials and tokens")]
pub struct ConfigArgs {
    #[arg(long, help = "Twitch application client ID")]
    pub client_id: Option<String>,
    #[arg(long, help = "Twitch application client secret")]
    pub client_secret: Option<String>,
    #[arg(long, help = "App access token for Twitch API calls")]
    pub access_token: Option<String>,
    #[arg(
        long,
        help = "Token expiry as an RFC3339 timestamp (e.g. 2026-01-26T12:34:56Z)"
    )]
    pub expires_at: Option<String>,
    #[arg(long, help = "Print the current configuration (secrets masked)")]
    pub show: bool,
}

pub fn run(args: ConfigArgs) -> Result<()> {
    let has_updates = args.client_id.is_some()
        || args.client_secret.is_some()
        || args.access_token.is_some()
        || args.expires_at.is_some();

    if !args.show && !has_updates {
        bail!(
            "at least one flag is required; use --client-id, --client-secret, --access-token, --expires-at, or --show"
        );
    }

    if args.show && !has_updates {
        let config = load_config()?;
        print_config(&config)?;
        return Ok(());
    }

    let mut config = load_config()?;

    if let Some(value) = args.client_id {
        config.twitch.client_id = Some(value);
    }

    if let Some(value) = args.client_secret {
        config.twitch.client_secret = Some(value);
    }

    if let Some(value) = args.access_token {
        config.twitch.access_token = Some(value);
    }

    if let Some(value) = args.expires_at {
        let parsed = DateTime::parse_from_rfc3339(&value)
            .with_context(|| "expires-at must be an RFC3339 timestamp")?;
        config.twitch.expires_at = Some(parsed.with_timezone(&Utc));
    }

    let path = config_path()?;
    save_config(&path, &config)?;
    println!("Config updated at {}", path.display());
    if args.show {
        print_config(&config)?;
    }
    Ok(())
}

fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config at {}", path.display()))?;
    let config: Config = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse config at {}", path.display()))?;
    Ok(config)
}

#[derive(Serialize)]
struct DisplayConfig {
    twitch: DisplayTwitchConfig,
}

#[derive(Serialize)]
struct DisplayTwitchConfig {
    client_id: Option<String>,
    client_secret: Option<String>,
    access_token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

fn print_config(config: &Config) -> Result<()> {
    let display = DisplayConfig {
        twitch: DisplayTwitchConfig {
            client_id: config.twitch.client_id.clone(),
            client_secret: mask_value(&config.twitch.client_secret),
            access_token: mask_value(&config.twitch.access_token),
            expires_at: config.twitch.expires_at,
        },
    };
    let json = serde_json::to_string_pretty(&display).context("failed to format config")?;
    println!("{json}");
    Ok(())
}

fn mask_value(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|_| "********".to_string())
}

fn save_config(path: &Path, config: &Config) -> Result<()> {
    let dir = path
        .parent()
        .context("config path should have a parent directory")?;
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    set_dir_permissions(dir)?;

    let json = serde_json::to_string_pretty(config).context("failed to serialize config")?;
    let tmp_path = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp_path)
            .with_context(|| format!("failed to write {}", tmp_path.display()))?;
        file.write_all(json.as_bytes())
            .context("failed to write config contents")?;
        file.sync_all().context("failed to flush config")?;
    }
    if let Err(err) = fs::rename(&tmp_path, path) {
        if path.exists() {
            fs::remove_file(path)
                .with_context(|| format!("failed to remove {}", path.display()))?;
            fs::rename(&tmp_path, path)
                .with_context(|| format!("failed to move config to {}", path.display()))?;
        } else {
            return Err(err)
                .with_context(|| format!("failed to move config to {}", path.display()));
        }
    }
    set_file_permissions(path)?;
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let base = config_base_dir()?;
    Ok(base.join("config.json"))
}

fn config_base_dir() -> Result<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("ttv"));
    }

    #[cfg(windows)]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("ttv"));
        }
    }

    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .context("could not determine home directory")?;
    Ok(PathBuf::from(home).join(".config").join("ttv"))
}

#[cfg(unix)]
fn set_dir_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_dir_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}
