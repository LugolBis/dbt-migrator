use std::collections::HashMap;

use crate::sql::normalizer::NormalizedRef;

/// dbt model meta data extracted from `dbt_project.yml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    /// Unique name used in `ref('nom_du_model')`
    pub name: String,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub tags: Vec<String>,
    /// dbt model path - only used by tree strategy fallback.
    pub relative_path: String,
}

/// Result of a table resolution.
#[derive(Debug, Clone)]
pub enum ResolutionOutcome {
    Resolved(ModelInfo),
    Conflict(Vec<ModelInfo>),
    NotFound,
}

/// Index multi-granularity : a same reference could be resolved with different level of grnaluarity
/// (`database.schema.table`), by `schema.table`, or by `table`
/// only if there isn't conflicts. See [`ResolutionIndex::resolve`].
#[derive(Debug, Default)]
pub struct ResolutionIndex {
    by_full: HashMap<String, Vec<ModelInfo>>,
    by_schema_table: HashMap<String, Vec<ModelInfo>>,
    by_table: HashMap<String, Vec<ModelInfo>>,
}

impl ResolutionIndex {
    pub fn insert(&mut self, model: ModelInfo) {
        if let (Some(database), Some(schema)) = (&model.database, &model.schema) {
            let key = format!("{database}.{schema}.{}", model.name_key());
            self.by_full.entry(key).or_default().push(model.clone());
        }
        if let Some(schema) = &model.schema {
            let key = format!("{schema}.{}", model.name_key());
            self.by_schema_table
                .entry(key)
                .or_default()
                .push(model.clone());
        }
        self.by_table
            .entry(model.name_key())
            .or_default()
            .push(model);
    }

    /// Apply the resolution strategy with the following priority (1 = higher priority) :
    /// 1. Complete name (`database.schema.table`)
    /// 2. `schema.table`
    /// 3. `table` only if there isn't conflicts
    pub fn resolve(&self, normalized: &NormalizedRef) -> ResolutionOutcome {
        if let (Some(database), Some(schema)) = (&normalized.database, &normalized.schema) {
            let key = format!("{database}.{schema}.{}", normalized.table);
            if let Some(candidates) = self.by_full.get(&key) {
                return Self::disambiguate(candidates);
            }
        }

        if let Some(schema) = &normalized.schema {
            let key = format!("{schema}.{}", normalized.table);
            if let Some(candidates) = self.by_schema_table.get(&key) {
                return Self::disambiguate(candidates);
            }
        }

        if let Some(candidates) = self.by_table.get(&normalized.table) {
            return Self::disambiguate(candidates);
        }

        ResolutionOutcome::NotFound
    }

    fn disambiguate(candidates: &[ModelInfo]) -> ResolutionOutcome {
        match candidates.len() {
            0 => ResolutionOutcome::NotFound,
            1 => ResolutionOutcome::Resolved(candidates[0].clone()),
            _ => ResolutionOutcome::Conflict(candidates.to_vec()),
        }
    }

    pub fn len(&self) -> usize {
        self.by_table.values().map(|v| v.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.by_table.is_empty()
    }
}

impl ModelInfo {
    /// Key to search model name.
    fn name_key(&self) -> String {
        self.name.to_lowercase()
    }
}
