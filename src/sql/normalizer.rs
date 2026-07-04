use sqlparser::ast::ObjectName;

/// Normalized table reference
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedRef {
    pub database: Option<String>,
    pub schema: Option<String>,
    pub table: String,
}

impl NormalizedRef {
    pub fn canonical_string(&self) -> String {
        match (&self.database, &self.schema) {
            (Some(db), Some(schema)) => format!("{db}.{schema}.{}", self.table),
            (None, Some(schema)) => format!("{schema}.{}", self.table),
            _ => self.table.clone(),
        }
    }
}

/// Normalize a Legacy table reference
pub trait Normalizer: Send + Sync {
    fn normalize(&self, raw: &ObjectName) -> NormalizedRef;
}

/// The default normalizer :
/// - 1 segment  -> table
/// - 2 segments -> schema.table
/// - 3+ segments -> the last 3 segments are used to construct database.schema.table
///   (deepest segment ares skipped, it could need to be adapted for Dremio)
pub struct DefaultNormalizer;

impl Normalizer for DefaultNormalizer {
    fn normalize(&self, raw: &ObjectName) -> NormalizedRef {
        let parts: Vec<String> = raw.0.iter().map(|ident| clean_part(&ident.value)).collect();

        match parts.len() {
            0 => NormalizedRef::default(),
            1 => NormalizedRef {
                database: None,
                schema: None,
                table: parts[0].clone(),
            },
            2 => NormalizedRef {
                database: None,
                schema: Some(parts[0].clone()),
                table: parts[1].clone(),
            },
            n => NormalizedRef {
                database: Some(parts[n - 3].clone()),
                schema: Some(parts[n - 2].clone()),
                table: parts[n - 1].clone(),
            },
        }
    }
}

/// CLean and normalize to lowercase segment
fn clean_part(raw: &str) -> String {
    raw.trim_matches(|c| c == '[' || c == ']' || c == '"' || c == '`')
        .to_lowercase()
}

/// Normalizer base on a REGEX that could be defined using `normalizer: "regex:..."`
/// in `migrate_config.yml`.
pub struct RegexNormalizer {
    regex: regex::Regex,
}

impl RegexNormalizer {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: regex::Regex::new(pattern)?,
        })
    }
}

impl Normalizer for RegexNormalizer {
    fn normalize(&self, raw: &ObjectName) -> NormalizedRef {
        let joined = raw
            .0
            .iter()
            .map(|i| i.value.clone())
            .collect::<Vec<_>>()
            .join(".");

        let Some(caps) = self.regex.captures(&joined) else {
            return DefaultNormalizer.normalize(raw);
        };

        NormalizedRef {
            database: caps.name("database").map(|m| m.as_str().to_lowercase()),
            schema: caps.name("schema").map(|m| m.as_str().to_lowercase()),
            table: caps
                .name("table")
                .map(|m| m.as_str().to_lowercase())
                .unwrap_or_default(),
        }
    }
}
