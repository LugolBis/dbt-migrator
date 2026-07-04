pub mod dialects;
pub mod normalizer;
pub mod report;
pub mod rewriter;

pub use normalizer::{DefaultNormalizer, NormalizedRef, Normalizer};
pub use report::{FileReport, MigrationReport};
