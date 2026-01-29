use std::path::PathBuf;

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Row, SqlitePool};

use crate::{fs_utils, paths, twitch::TwitchUser};

const DB_FILENAME: &str = "ttv.sqlite";

pub async fn connect() -> Result<SqlitePool> {
    let path = db_path()?;
    let dir = path
        .parent()
        .context("database path should have a parent directory")?;
    fs_utils::ensure_dir(dir)?;

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .with_context(|| format!("failed to open database at {}", path.display()))?;

    init_schema(&pool).await?;
    fs_utils::set_file_permissions(&path)?;
    Ok(pool)
}

pub fn db_path() -> Result<PathBuf> {
    let base = paths::data_dir()?;
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

pub struct DbStreamer {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

pub async fn list_streamers(pool: &SqlitePool) -> Result<Vec<DbStreamer>> {
    let rows = sqlx::query("SELECT id, name, display_name FROM streamers ORDER BY name")
        .fetch_all(pool)
        .await
        .context("failed to load streamers")?;

    let mut streamers = Vec::with_capacity(rows.len());
    for row in rows {
        streamers.push(DbStreamer {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            display_name: row.try_get("display_name")?,
        });
    }

    Ok(streamers)
}

pub async fn delete_streamer_by_login(pool: &SqlitePool, login: &str) -> Result<u64> {
    let result = sqlx::query("DELETE FROM streamers WHERE lower(name) = lower(?1)")
        .bind(login)
        .execute(pool)
        .await
        .context("failed to delete streamer")?;
    Ok(result.rows_affected())
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
