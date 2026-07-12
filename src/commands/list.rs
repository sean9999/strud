use anyhow::{bail, Context, Result};
use chrono::{Days, Local, NaiveDate, NaiveDateTime};
use serde_yaml::Value;
use std::path::PathBuf;

use crate::diary;
use crate::parse::parse_file;
use crate::validate::validate_entry;

pub fn run(
    dir: Option<PathBuf>,
    date: Option<String>,
    since: Option<String>,
    until: Option<String>,
    last: Option<usize>,
    raw: bool,
) -> Result<()> {
    let dir = diary::resolve_dir(dir, None)?;
    let cfg = crate::config::load(&dir)?;

    let date_f = parse_date_arg(date, "date")?;
    let since_f = parse_date_arg(since, "since")?;
    let until_f = parse_date_arg(until, "until")?;

    let today = Local::now().date_naive();
    let default_range = date_f.is_none() && since_f.is_none() && until_f.is_none() && last.is_none();
    let since_eff = if default_range {
        Some(today - Days::new(13))
    } else {
        since_f
    };

    let mut files: Vec<PathBuf> = Vec::new();
    for e in std::fs::read_dir(&dir)? {
        let p = e?.path();
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if is_day_file(name) {
                files.push(p);
            }
        }
    }
    files.sort();

    if raw {
        let mut matching: Vec<(NaiveDate, PathBuf)> = files
            .into_iter()
            .filter_map(|p| {
                let name = p.file_name().unwrap().to_str().unwrap();
                let d = NaiveDate::parse_from_str(&name[..10], "%Y-%m-%d").ok()?;
                if let Some(s) = since_eff {
                    if d < s {
                        return None;
                    }
                }
                if let Some(u) = until_f {
                    if d > u {
                        return None;
                    }
                }
                if let Some(df) = date_f {
                    if d != df {
                        return None;
                    }
                }
                Some((d, p))
            })
            .collect();
        if let Some(n) = last {
            matching.sort_by_key(|(d, _)| *d);
            let start = matching.len().saturating_sub(n);
            matching = matching[start..].to_vec();
        }
        if matching.is_empty() {
            bail!("no entries match");
        }
        for (_, p) in matching {
            let content = std::fs::read_to_string(&p)?;
            println!("{}", content.trim_end_matches('\n'));
            println!();
        }
        return Ok(());
    }

    let mut rows: Vec<(NaiveDateTime, Vec<String>)> = Vec::new();
    for p in &files {
        let fname = p.file_name().unwrap().to_str().unwrap();
        let file_date = NaiveDate::parse_from_str(&fname[..10], "%Y-%m-%d").ok();
        let text = std::fs::read_to_string(p)?;
        let entries = parse_file(&text)?;
        for e in &entries {
            let dt = match e.date {
                Some(d) => d,
                None => {
                    eprintln!(
                        "warning: {} has an entry without a valid date; skipping",
                        p.display()
                    );
                    continue;
                }
            };
            let ed = dt.date();
            if let Some(s) = since_eff {
                if ed < s {
                    continue;
                }
            }
            if let Some(u) = until_f {
                if ed > u {
                    continue;
                }
            }
            if let Some(df) = date_f {
                if ed != df {
                    continue;
                }
            }
            if let Some(fd) = file_date {
                if fd != ed {
                    eprintln!(
                        "warning: {} entry {} date does not match filename",
                        p.display(),
                        dt.format("%Y-%m-%dT%H:%M")
                    );
                }
            }
            let issues = validate_entry(e, &cfg);
            if !issues.is_empty() {
                eprintln!(
                    "warning: {} entry {}: {}",
                    p.display(),
                    dt.format("%Y-%m-%dT%H:%M"),
                    issues.join("; ")
                );
            }
            let mut row = vec![dt.format("%Y-%m-%dT%H:%M").to_string()];
            for m in &cfg.metric {
                let cell = e
                    .frontmatter
                    .get(&Value::String(m.name.clone()))
                    .map(value_to_cell)
                    .unwrap_or_default();
                row.push(cell);
            }
            rows.push((dt, row));
        }
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    if let Some(n) = last {
        let start = rows.len().saturating_sub(n);
        rows = rows[start..].to_vec();
    }
    if rows.is_empty() {
        bail!("no entries match");
    }

    let mut headers = vec!["date".to_string()];
    for m in &cfg.metric {
        headers.push(m.name.clone());
    }
    let table_rows: Vec<Vec<String>> = rows.into_iter().map(|(_, r)| r).collect();
    print_table(&headers, &table_rows);
    Ok(())
}

fn parse_date_arg(s: Option<String>, flag: &str) -> Result<Option<NaiveDate>> {
    match s {
        None => Ok(None),
        Some(v) => {
            let d = NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                .with_context(|| format!("invalid --{} '{}'", flag, v))?;
            Ok(Some(d))
        }
    }
}

fn is_day_file(name: &str) -> bool {
    name.len() == 13
        && name.ends_with(".md")
        && NaiveDate::parse_from_str(&name[..10], "%Y-%m-%d").is_ok()
}

fn value_to_cell(v: &Value) -> String {
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => serde_yaml::to_string(v)
            .map(|s| s.trim_start_matches("---\n").trim().to_string())
            .unwrap_or_default(),
    }
}

fn print_table(headers: &[String], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, c) in row.iter().enumerate() {
            if i < widths.len() && widths[i] < c.len() {
                widths[i] = c.len();
            }
        }
    }
    let line = |cells: &[String]| {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
            .collect::<Vec<_>>()
            .join("  ")
    };
    println!("{}", line(headers));
    for row in rows {
        println!("{}", line(row));
    }
}