use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    set_dir_permissions(path)?;
    Ok(())
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

pub fn set_file_permissions(path: &Path) -> Result<()> {
    set_file_permissions_impl(path)
}

#[cfg(unix)]
fn set_file_permissions_impl(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_file_permissions_impl(_path: &Path) -> Result<()> {
    Ok(())
}
