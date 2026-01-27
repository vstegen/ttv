use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn config_dir() -> Result<PathBuf> {
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

pub fn data_dir() -> Result<PathBuf> {
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
