use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use dialoguer::Input;
use serde_yaml::Mapping;
use std::path::{Path, PathBuf};

use crate::config::{Metric, MetricType};
use crate::diary;
use crate::entry::Entry;
use crate::parse::{parse_dt, render_file};

#[derive(Debug, Clone)]
enum MetricValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Enum(String),
}

pub fn run(dir: Option<PathBuf>, date: Option<String>) -> Result<()> {
    let dir = diary::resolve_dir(dir)?;
    let cfg = crate::config::load(&dir)?;

    let now = Local::now().naive_local();
    let date = match date {
        Some(s) => parse_dt(&s).with_context(|| format!("invalid --date '{}'", s))?,
        None => now.with_second(0).unwrap().with_nanosecond(0).unwrap(),
    };

    let file_date = date.date();
    let path = diary::entries_dir(&dir).join(format!("{}.md", file_date.format("%Y-%m-%d")));

    // If an entry already exists for this day, skip the prompts entirely and
    // just open the whole file for editing.
    if path.exists() {
        edit_file(&path)?;
        println!("Editing entry {}", path.display());
        return Ok(());
    }

    let filled = prompt_metrics(&cfg.metric)?;
    let template = std::fs::read_to_string(dir.join("template.md")).unwrap_or_default();

    // `strud new` always creates a fresh file: compose the front matter and
    // body template into their final form, write them exactly where they
    // belong, then hand the file itself to $EDITOR.
    let fm_text = render_frontmatter(&date, &cfg.metric, &filled);
    let frontmatter: Mapping = serde_yaml::from_str(&fm_text).unwrap_or_default();
    let entry = Entry {
        date: Some(date),
        frontmatter,
        fm_text,
        body: template.trim_end_matches('\n').to_string(),
    };

    let out = render_file(std::slice::from_ref(&entry));
    std::fs::create_dir_all(diary::entries_dir(&dir))?;
    std::fs::write(&path, out)?;

    edit_file(&path)?;

    println!(
        "Created entry {} at {}",
        date.format("%Y-%m-%dT%H:%M"),
        path.display()
    );
    Ok(())
}

fn prompt_metrics(metrics: &[Metric]) -> Result<Vec<(String, MetricValue)>> {
    let mut out = Vec::new();
    for m in metrics {
        let prompt = format!("{} ({}) [blank to skip]", m.name, m.type_hint());
        loop {
            let input: String = Input::new()
                .with_prompt(&prompt)
                .allow_empty(true)
                .interact_text()
                .context("prompt failed")?;
            let trimmed = input.trim();
            if trimmed.is_empty() {
                break;
            }
            match parse_metric_value(m, trimmed) {
                Ok(v) => {
                    out.push((m.name.clone(), v));
                    break;
                }
                Err(e) => eprintln!("  invalid: {}", e),
            }
        }
    }
    Ok(out)
}

fn parse_metric_value(m: &Metric, s: &str) -> Result<MetricValue, String> {
    match m.ty {
        MetricType::Int => {
            let n: i64 = s.parse().map_err(|_| "expected integer".to_string())?;
            if let Some(min) = m.min
                && (n as f64) < min
            {
                return Err(format!("min {}", min));
            }
            if let Some(max) = m.max
                && (n as f64) > max
            {
                return Err(format!("max {}", max));
            }
            Ok(MetricValue::Int(n))
        }
        MetricType::Float => {
            let n: f64 = s.parse().map_err(|_| "expected number".to_string())?;
            if let Some(min) = m.min
                && n < min
            {
                return Err(format!("min {}", min));
            }
            if let Some(max) = m.max
                && n > max
            {
                return Err(format!("max {}", max));
            }
            Ok(MetricValue::Float(n))
        }
        MetricType::Bool => match s.to_lowercase().as_str() {
            "y" | "yes" | "true" | "t" => Ok(MetricValue::Bool(true)),
            "n" | "no" | "false" | "f" => Ok(MetricValue::Bool(false)),
            _ => Err("expected y/n".to_string()),
        },
        MetricType::Enum => {
            let vals = m.values.as_ref().ok_or("enum has no values")?;
            if vals.iter().any(|v| v == s) {
                Ok(MetricValue::Enum(s.to_string()))
            } else {
                Err(format!("expected one of {}", vals.join("|")))
            }
        }
    }
}

fn fmt_float(f: f64) -> String {
    let s = format!("{}", f);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{}.0", s)
    }
}

fn render_frontmatter(
    date: &chrono::NaiveDateTime,
    metrics: &[Metric],
    filled: &[(String, MetricValue)],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("date: {}", date.format("%Y-%m-%dT%H:%M")));
    for m in metrics {
        if let Some((_, v)) = filled.iter().find(|(n, _)| n == &m.name) {
            let cell = match v {
                MetricValue::Int(i) => i.to_string(),
                MetricValue::Float(f) => fmt_float(*f),
                MetricValue::Bool(b) => b.to_string(),
                MetricValue::Enum(s) => s.clone(),
            };
            lines.push(format!("{}: {}", m.name, cell));
        }
    }
    lines.join("\n")
}

fn edit_file(path: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let mut parts: Vec<String> = editor.split_whitespace().map(String::from).collect();
    if parts.is_empty() {
        return Ok(());
    }
    let prog = parts.remove(0);
    let status = std::process::Command::new(prog)
        .args(parts)
        .arg(path)
        .status()
        .with_context(|| format!("failed to run editor '{}'", editor))?;
    if !status.success() {
        anyhow::bail!("editor '{}' exited with status {}", editor, status);
    }
    Ok(())
}
