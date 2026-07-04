use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Replacement {
    pub original: String,
    pub model: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UnresolvedRef {
    pub original: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConflictRef {
    pub original: String,
    pub candidates: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FileReport {
    pub file: String,
    pub replacements: Vec<Replacement>,
    pub warnings: Vec<UnresolvedRef>,
    pub conflicts: Vec<ConflictRef>,
    pub error: Option<String>,
}

impl FileReport {
    pub fn new(file: &Path) -> Self {
        Self {
            file: file.to_string_lossy().to_string(),
            replacements: Vec::new(),
            warnings: Vec::new(),
            conflicts: Vec::new(),
            error: None,
        }
    }

    pub fn error(file: &Path, message: impl Into<String>) -> Self {
        let mut report = Self::new(file);
        report.error = Some(message.into());
        report
    }

    pub fn has_issues(&self) -> bool {
        self.error.is_some() || !self.warnings.is_empty() || !self.conflicts.is_empty()
    }
}

/// Aggregated Report who could be exported as JSON or CSV file.
#[derive(Debug, Serialize, Default)]
pub struct MigrationReport {
    pub files: Vec<FileReport>,
}

impl MigrationReport {
    pub fn total_replacements(&self) -> usize {
        self.files.iter().map(|f| f.replacements.len()).sum()
    }

    pub fn total_warnings(&self) -> usize {
        self.files.iter().map(|f| f.warnings.len()).sum()
    }

    pub fn total_conflicts(&self) -> usize {
        self.files.iter().map(|f| f.conflicts.len()).sum()
    }

    pub fn write_json(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).expect("JSON serialization issue");
        std::fs::write(path, json)
    }

    pub fn write_csv(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_path(path)?;
        writer.write_record(["File", "Type", "Original_Table", "Target_Or_Candidates"])?;

        for file in &self.files {
            for r in &file.replacements {
                writer.write_record([&file.file, "Replacement", &r.original, &r.model])?;
            }
            for w in &file.warnings {
                writer.write_record([&file.file, "Warning", &w.original, ""])?;
            }
            for c in &file.conflicts {
                writer.write_record([
                    &file.file,
                    "conflict",
                    &c.original,
                    &c.candidates.join(" | "),
                ])?;
            }
            if let Some(err) = &file.error {
                writer.write_record([&file.file, "Error", err, ""])?;
            }
        }
        writer.flush()?;
        Ok(())
    }
}

pub fn default_report_paths(out_dir: &Path) -> (PathBuf, PathBuf) {
    (
        out_dir.join("migration_report.json"),
        out_dir.join("migration_report.csv"),
    )
}
