use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

const STARTER_TOML: &str = "# strud schema configuration. Edit freely.\n\
\n\
# Diary directory override. Optional; --dir flag and $STRUD_DIR take priority.\n\
# dir = \"~/.strud\"\n\
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

pub fn run(dir: &Path, force: bool) -> Result<()> {
    let dir = if dir.as_os_str().is_empty() {
        crate::diary::resolve_dir(None, None)?
    } else {
        dir.to_path_buf()
    };
    fs::create_dir_all(&dir)?;

    let toml_path = dir.join("strud.toml");
    let tpl_path = dir.join("default.template.md");

    if toml_path.exists() {
        if force {
            fs::write(&toml_path, STARTER_TOML)?;
        } else {
            bail!("{} already exists; pass --force to overwrite", toml_path.display());
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