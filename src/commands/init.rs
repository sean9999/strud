use anyhow::{Result, bail};
use std::fs;
use std::path::PathBuf;

const STARTER_TOML: &str = "# strud schema configuration. Edit freely.\n\
# Run `strud init` to regenerate; pass --force to overwrite this file.\n\
\n\
[[metric]]\n\
name = \"mood\"\n\
type = \"int\"\n\
min = 1\n\
max = 5\n\
\n\
[[metric]]\n\
name = \"sleep_hours\"\n\
type = \"float\"\n\
min = 0\n\
max = 24\n\
\n\
[[metric]]\n\
name = \"energy\"\n\
type = \"enum\"\n\
values = [\"low\", \"medium\", \"high\"]\n\
\n\
[[metric]]\n\
name = \"exercised\"\n\
type = \"bool\"\n";

const STARTER_TEMPLATE: &str = "## Notes\n\n## Wins\n";

pub fn run(dir: Option<PathBuf>, force: bool) -> Result<()> {
    let dir = match dir {
        Some(d) => d,
        None => crate::diary::resolve_dir(None)?,
    };
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("entries"))?;

    let toml_path = dir.join("strud.toml");
    let tpl_path = dir.join("default.template.md");

    if toml_path.exists() {
        if force {
            fs::write(&toml_path, STARTER_TOML)?;
        } else {
            bail!(
                "{} already exists; pass --force to overwrite",
                toml_path.display()
            );
        }
    } else {
        fs::write(&toml_path, STARTER_TOML)?;
    }

    // The template is user content: write only if absent, never overwrite.
    if !tpl_path.exists() {
        fs::write(&tpl_path, STARTER_TEMPLATE)?;
    }

    println!("Created diary at {}", dir.display());
    Ok(())
}
