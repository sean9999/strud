use anyhow::Result;
use chrono::NaiveDateTime;
use serde_yaml::{Mapping, Value};

use crate::entry::Entry;

/// Parse a day file into its constituent entries.
///
/// A day file is a sequence of `---`-delimited front-matter blocks, each
/// followed by a Markdown body. Fences are paired open/close; the body of an
/// entry runs from its close fence to the next open fence (or EOF).
/// Leading content before the first fence becomes a dateless entry.
pub fn parse_file(text: &str) -> Result<Vec<Entry>> {
    let lines: Vec<&str> = text.split('\n').collect();
    let fences: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim() == "---")
        .map(|(i, _)| i)
        .collect();

    let mut entries = Vec::new();

    if fences.is_empty() {
        let body = text.trim_matches('\n');
        if !body.is_empty() {
            entries.push(Entry {
                date: None,
                frontmatter: Mapping::new(),
                fm_text: String::new(),
                body: body.to_string(),
            });
        }
        return Ok(entries);
    }

    if fences[0] > 0 {
        let lead = lines[..fences[0]].join("\n");
        let lead = lead.trim_matches('\n');
        if !lead.is_empty() {
            entries.push(Entry {
                date: None,
                frontmatter: Mapping::new(),
                fm_text: String::new(),
                body: lead.to_string(),
            });
        }
    }

    let mut k = 0;
    while k + 1 < fences.len() {
        let open = fences[k];
        let close = fences[k + 1];
        let fm_text = lines[open + 1..close].join("\n").trim_matches('\n').to_string();
        let body_end = if k + 2 < fences.len() {
            fences[k + 2]
        } else {
            lines.len()
        };
        let body = lines[close + 1..body_end].join("\n").trim_matches('\n').to_string();
        let frontmatter: Mapping = if fm_text.is_empty() {
            Mapping::new()
        } else {
            serde_yaml::from_str(&fm_text)?
        };
        let date = extract_date(&frontmatter);
        entries.push(Entry {
            date,
            frontmatter,
            fm_text,
            body,
        });
        k += 2;
    }

    Ok(entries)
}

fn extract_date(fm: &Mapping) -> Option<NaiveDateTime> {
    let v = fm.get(&Value::String("date".into()))?;
    scalar_to_string(v).as_deref().and_then(parse_dt)
}

fn scalar_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Parse an ISO-ish datetime, accepting minute or second precision.
pub fn parse_dt(s: &str) -> Option<NaiveDateTime> {
    ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"]
        .iter()
        .find_map(|f| NaiveDateTime::parse_from_str(s, f).ok())
}

/// Re-render a day file from its entries. Existing front-matter and body text
/// are emitted verbatim; only the inter-entry blank-line separator and the
/// trailing newline are normalized.
pub fn render_file(entries: &[Entry]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let units: Vec<String> = entries.iter().map(render_unit).collect();
    let mut out = units.join("\n\n");
    out.push('\n');
    out
}

fn render_unit(e: &Entry) -> String {
    let mut u = String::from("---\n");
    u.push_str(&e.fm_text);
    if !e.fm_text.is_empty() {
        u.push('\n');
    }
    u.push_str("---");
    if !e.body.is_empty() {
        u.push('\n');
        u.push_str(&e.body);
    }
    u
}