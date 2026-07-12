mod commands;
mod config;
mod diary;
mod entry;
mod parse;
mod validate;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "strud", version, about = "structured diary backed by Markdown")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scaffold a new diary directory with config and template
    Init {
        /// Target directory (default: $STRUD_DIR or ~/.strud)
        #[arg(default_value = "")]
        dir: PathBuf,
        /// Overwrite an existing strud.toml
        #[arg(long)]
        force: bool,
    },
    /// Create a new diary entry
    New {
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Override entry datetime, e.g. 2026-07-12T22:40
        #[arg(long)]
        date: Option<String>,
    },
    /// List entries
    List {
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Only this calendar day, e.g. 2026-07-12
        #[arg(long)]
        date: Option<String>,
        /// Inclusive lower date bound, e.g. 2026-07-01
        #[arg(long)]
        since: Option<String>,
        /// Inclusive upper date bound, e.g. 2026-07-31
        #[arg(long)]
        until: Option<String>,
        /// Show only the last N matches
        #[arg(long)]
        last: Option<usize>,
        /// Dump underlying Markdown instead of the table
        #[arg(long)]
        raw: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init { dir, force } => commands::init::run(&dir, force),
        Cmd::New { dir, date } => commands::new::run(dir, date),
        Cmd::List {
            dir,
            date,
            since,
            until,
            last,
            raw,
        } => commands::list::run(dir, date, since, until, last, raw),
    }
}