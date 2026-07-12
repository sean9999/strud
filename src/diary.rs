use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

/// Expand a leading `~` to $HOME. Env vars are not expanded.
#[allow(dead_code)]
pub fn expand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    if path == "~" {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
}

/// Resolve the diary directory. Order: `--dir` flag, then $STRUD_DIR, then ~/.strud.
///
/// The `config_dir` parameter (strud.toml's `dir` key) is accepted but not used
/// for resolution: since strud.toml lives inside the diary dir, reading it
/// already requires knowing the dir. The field is kept in the schema for
/// forward compatibility.
pub fn resolve_dir(flag: Option<PathBuf>, _config_dir: Option<&str>) -> Result<PathBuf> {
    if let Some(d) = flag {
        return Ok(d);
    }
    if let Ok(d) = env::var("STRUD_DIR") {
        return Ok(PathBuf::from(d));
    }
    let home = env::var("HOME").context("HOME not set; pass --dir or set $STRUD_DIR")?;
    Ok(PathBuf::from(home).join(".strud"))
}