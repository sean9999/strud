use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolve the diary directory: the `--dir` flag if given, otherwise `~/.strud`.
pub fn resolve_dir(flag: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(d) = flag {
        return Ok(d);
    }
    let home = std::env::var("HOME").context("HOME not set; pass --dir")?;
    Ok(PathBuf::from(home).join(".strud"))
}

/// Day files live in `<diary-dir>/entries/`. Config (`strud.toml`) and the
/// body template stay at the diary dir root.
pub fn entries_dir(dir: &Path) -> PathBuf {
    dir.join("entries")
}
