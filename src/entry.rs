use chrono::NaiveDateTime;
use serde_yaml::Mapping;

/// One parsed diary entry: a front-matter block plus a Markdown body.
///
/// `fm_text` holds the raw YAML text between the `---` fences (verbatim, for
/// lossless re-emission); `frontmatter` is the parsed mapping for reading.
#[derive(Debug, Clone)]
pub struct Entry {
    pub date: Option<NaiveDateTime>,
    pub frontmatter: Mapping,
    pub fm_text: String,
    pub body: String,
}
