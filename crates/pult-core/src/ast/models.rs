use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Arquivo inspecionado e seu sumário de assinaturas compactadas extraídas do AST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedAstFile {
    pub file_path: PathBuf,
    pub relative_path: String,
    pub extracted_signatures: String,
    pub raw_byte_count: usize,
    pub reduced_byte_count: usize,
}

impl ExtractedAstFile {
    /// Taxa de redução percentual em bytes obtida pela extração do AST.
    pub fn reduction_ratio(&self) -> f32 {
        if self.raw_byte_count == 0 {
            return 0.0;
        }
        (1.0 - (self.reduced_byte_count as f32 / self.raw_byte_count as f32)) * 100.0
    }
}
