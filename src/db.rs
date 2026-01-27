use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;

use crate::twitch::TwitchUser;

const DB_FILENAME: &str = "ttv.sqlite";

pub async fn connect() -> Result<SqlitePool> {
    let path = db_path()?;
    let dir = path
        .parent()
        .context("database path should have a parent directory")?;
    fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    set_dir_permissions(dir)?;

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .with_context(|| format!("failed to open database at {}", path.display()))?;

    init_schema(&pool).await?;
    set_file_permissions(&path)?;
    Ok(pool)
}

pub fn db_path() -> Result<PathBuf> {
    let base = data_base_dir()?;
    Ok(base.join(DB_FILENAME))
}

pub async fn upsert_streamer(pool: &SqlitePool, streamer: &TwitchUser) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO streamers (id, name, display_name)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            display_name = excluded.display_name,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&streamer.id)
    .bind(&streamer.login)
    .bind(&streamer.display_name)
    .execute(pool)
    .await
    .context("failed to upsert streamer")?;
    Ok(())
}

async fn init_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS streamers (
            uid INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .context("failed to initialize database schema")?;
    Ok(())
}

fn data_base_dir() -> Result<PathBuf> {
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
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
    Ok(PathBuf::from(home).join(".local").join("share").join("ttv"))
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
