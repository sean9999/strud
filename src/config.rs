use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Int,
    Float,
    Enum,
    Bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metric {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: MetricType,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    #[allow(dead_code)]
    pub dir: Option<String>,
    #[serde(default)]
    pub metric: Vec<Metric>,
}

pub fn load(diary_dir: &Path) -> Result<Config> {
    let path = diary_dir.join("strud.toml");
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse error in {}", path.display()))
}

impl Metric {
    pub fn type_hint(&self) -> String {
        match self.ty {
            MetricType::Int => format!("int{}", range_str(self)),
            MetricType::Float => format!("float{}", range_str(self)),
            MetricType::Enum => self
                .values
                .as_ref()
                .map(|v| v.join("|"))
                .unwrap_or_else(|| "enum".into()),
            MetricType::Bool => "y/n".into(),
        }
    }

    pub fn validate_value(&self, v: &Value) -> Result<(), String> {
        match self.ty {
            MetricType::Int => match v.as_i64() {
                Some(n) => {
                    self.check_range(n as f64)?;
                    Ok(())
                }
                None => Err("expected integer".into()),
            },
            MetricType::Float => match v.as_f64() {
                Some(n) => {
                    self.check_range(n)?;
                    Ok(())
                }
                None => Err("expected number".into()),
            },
            MetricType::Bool => match v {
                Value::Bool(_) => Ok(()),
                _ => Err("expected y/n".into()),
            },
            MetricType::Enum => match v {
                Value::String(s) => {
                    let allowed = self.values.as_ref();
                    if allowed.map(|v| v.iter().any(|x| x == s)).unwrap_or(false) {
                        Ok(())
                    } else {
                        Err(format!(
                            "expected one of {}",
                            allowed.map(|v| v.join("|")).unwrap_or_default()
                        ))
                    }
                }
                _ => Err("expected enum value".into()),
            },
        }
    }

    fn check_range(&self, n: f64) -> Result<(), String> {
        if let Some(min) = self.min {
            if n < min {
                return Err(format!("min {}", min));
            }
        }
        if let Some(max) = self.max {
            if n > max {
                return Err(format!("max {}", max));
            }
        }
        Ok(())
    }
}

fn range_str(m: &Metric) -> String {
    match (m.min, m.max) {
        (Some(a), Some(b)) => format!(" {}-{}", a, b),
        (Some(a), None) => format!(" >={}", a),
        (None, Some(b)) => format!(" <={}", b),
        (None, None) => String::new(),
    }
}