use crate::config::Config;
use crate::entry::Entry;

/// Validate an entry against the schema: presence of `date`, and type/range of
/// any declared metrics that are actually present. Unknown keys are ignored
/// (lenient — see SPEC.md §5). Returns a list of human-readable issues.
pub fn validate_entry(e: &Entry, cfg: &Config) -> Vec<String> {
    let mut issues = Vec::new();
    if e.date.is_none() {
        issues.push("missing or invalid 'date'".to_string());
    }
    for m in &cfg.metric {
        if let Some(v) = e.frontmatter.get(&serde_yaml::Value::String(m.name.clone())) {
            if let Err(msg) = m.validate_value(v) {
                issues.push(format!("{}: {}", m.name, msg));
            }
        }
    }
    issues
}