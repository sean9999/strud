use anyhow::{Context, Result};
use std::path::PathBuf;

/// Resolve the diary directory: the `--dir` flag if given, otherwise `~/.strud`.
pub fn resolve_dir(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = flag {
        return Ok(d);
    }
    let home = std::env::var("HOME").context("HOME not set; pass --dir")?;
    Ok(PathBuf::from(home).join(".strud"))
}
