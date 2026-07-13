//! Body-template interpolation for `strud new`.
//!
//! Templates are [Handlebars](https://crates.io/crates/handlebars) with one
//! custom helper, `format`, that renders a timestamp source with a chrono
//! strftime pattern:
//!
//! ```text
//! ##  {{format now "%H:%M"}}
//! # {{format date "%Y-%m-%d"}}
//! ```
//!
//! Available sources are `now` (the current local timestamp) and `date` (the
//! entry's date — equal to `now` unless `--date` overrides it). New sources
//! and helpers can be added below without touching the call sites.

use anyhow::{Context as _, Result};
use chrono::NaiveDateTime;
use handlebars::{
    Context as HbsContext, Handlebars, Helper, Output, RenderContext, RenderErrorReason,
    HelperResult,
};
use serde::Serialize;

use crate::parse::parse_dt;

#[derive(Serialize)]
struct TemplateData {
    now: String,
    date: String,
}

/// Render `template`, substituting `{{format <source> "<strftime>"}}` with the
/// formatted timestamp. Unknown variables render empty (Handlebars default).
pub fn render(template: &str, now: NaiveDateTime, date: NaiveDateTime) -> Result<String> {
    let mut hbs = Handlebars::new();
    hbs.register_helper("format", Box::new(format_helper));
    let data = TemplateData {
        now: now.format("%Y-%m-%dT%H:%M").to_string(),
        date: date.format("%Y-%m-%dT%H:%M").to_string(),
    };
    hbs.render_template(template, &data)
        .with_context(|| "template render failed")
}

/// `{{format <source> "<strftime>"}}` — formats the first param (a datetime
/// string) with the second (a chrono strftime pattern).
fn format_helper(
    h: &Helper,
    _: &Handlebars,
    _: &HbsContext,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let val = h
        .param(0)
        .and_then(|p| p.value().as_str())
        .ok_or(RenderErrorReason::ParamTypeMismatchForName(
            "format",
            0.to_string(),
            "datetime".to_string(),
        ))?;
    let fmt = h
        .param(1)
        .and_then(|p| p.value().as_str())
        .ok_or(RenderErrorReason::ParamTypeMismatchForName(
            "format",
            1.to_string(),
            "strftime pattern".to_string(),
        ))?;
    let dt = parse_dt(val)
        .ok_or_else(|| RenderErrorReason::Other(format!("not a datetime: {}", val)))?;
    out.write(&dt.format(fmt).to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::render;
    use chrono::NaiveDate;

    fn dt() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 7, 13)
            .unwrap()
            .and_hms_opt(19, 17, 0)
            .unwrap()
    }

    #[test]
    fn formats_now_and_date() {
        let t = render(
            "## {{format now \"%H:%M\"}}\nDate: {{format date \"%Y-%m-%d\"}}",
            dt(),
            dt(),
        )
        .unwrap();
        assert_eq!(t, "## 19:17\nDate: 2026-07-13");
    }

    #[test]
    fn passthrough_without_tags() {
        let t = render("# Today, I\n# Am Thankful For", dt(), dt()).unwrap();
        assert_eq!(t, "# Today, I\n# Am Thankful For");
    }

    #[test]
    fn unknown_source_errors() {
        let t = render("## {{format bogus \"%H:%M\"}}", dt(), dt());
        assert!(t.is_err());
    }
}