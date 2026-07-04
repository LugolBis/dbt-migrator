use std::path::Path;

use sqlparser::ast::{Ident, ObjectName, Statement, VisitMut, VisitorMut};
use sqlparser::dialect::Dialect;
use sqlparser::parser::Parser;

use crate::config::schema::{ConflictRule, MigrateConfig};
use crate::dbt::index::{ResolutionIndex, ResolutionOutcome};
use crate::error::{DbtMigratorError, Result};
use crate::sql::normalizer::Normalizer;
use crate::sql::report::{ConflictRef, FileReport, Replacement, UnresolvedRef};

/// Mutable Visitor who traverses the AST to migrate table references using the dbt index.
struct RefRewriter<'a> {
    index: &'a ResolutionIndex,
    normalizer: &'a dyn Normalizer,
    conflict_rules: &'a [ConflictRule],
    script_id: &'a str,
    report: FileReport,
}

impl<'a> VisitorMut for RefRewriter<'a> {
    type Break = ();

    fn pre_visit_relation(
        &mut self,
        relation: &mut ObjectName,
    ) -> std::ops::ControlFlow<Self::Break> {
        self.try_rewrite(relation);
        std::ops::ControlFlow::Continue(())
    }
}

impl<'a> RefRewriter<'a> {
    fn try_rewrite(&mut self, relation: &mut ObjectName) {
        let original = relation.to_string();
        let normalized = self.normalizer.normalize(relation);
        let canonical = normalized.canonical_string();

        // 1. A conflict rule has the absolute priority.
        if let Some(rule) = self
            .conflict_rules
            .iter()
            .find(|r| r.script == self.script_id && r.table == canonical)
        {
            self.apply_replacement(relation, &original, &rule.model);
            return;
        }

        match self.index.resolve(&normalized) {
            ResolutionOutcome::Resolved(model) => {
                self.apply_replacement(relation, &original, &model.name);
            }
            ResolutionOutcome::Conflict(candidates) => {
                log::warn!(
                    "{}: ambiguous reference '{}' -> {} candidates models - reference conserved.",
                    self.script_id,
                    original,
                    candidates.len()
                );
                self.report.conflicts.push(ConflictRef {
                    original,
                    candidates: candidates
                        .into_iter()
                        .map(|c| format!("{} ({})", c.name, c.relative_path))
                        .collect(),
                });
            }
            ResolutionOutcome::NotFound => {
                log::debug!(
                    "{}: There isn't any dbt model found for '{}', reference conserved.",
                    self.script_id,
                    original
                );
                self.report.warnings.push(UnresolvedRef { original });
            }
        }
    }

    fn apply_replacement(&mut self, relation: &mut ObjectName, original: &str, model_name: &str) {
        let macro_text = format!("{{{{ ref('{model_name}') }}}}");
        relation.0 = vec![Ident {
            value: macro_text,
            quote_style: None,
        }];
        self.report.replacements.push(Replacement {
            original: original.to_string(),
            model: model_name.to_string(),
        });
    }
}

pub fn rewrite_sql(
    script_id: &str,
    sql: &str,
    dialect: &dyn Dialect,
    index: &ResolutionIndex,
    normalizer: &dyn Normalizer,
    config: &MigrateConfig,
) -> Result<(String, FileReport)> {
    let mut statements =
        Parser::parse_sql(dialect, sql).map_err(|source| DbtMigratorError::SqlParse {
            path: script_id.into(),
            source,
        })?;

    let mut rewriter = RefRewriter {
        index,
        normalizer,
        conflict_rules: &config.conflicts,
        script_id,
        report: FileReport::new(Path::new(script_id)),
    };

    for statement in statements.iter_mut() {
        let _ = VisitMut::visit(statement, &mut rewriter);
    }

    let rewritten = statements
        .iter()
        .map(Statement::to_string)
        .collect::<Vec<_>>()
        .join(";\n")
        + ";\n";

    Ok((rewritten, rewriter.report))
}
