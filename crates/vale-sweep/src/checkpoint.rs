use crate::result::SweepResult;
use std::path::Path;
use vale_core::error::{ValeError, ValeResult};

pub fn save_checkpoint(path: &Path, results: &[SweepResult]) -> ValeResult<()> {
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| ValeError::Parse(e.to_string()))?;
    std::fs::write(path, json).map_err(ValeError::Io)
}

pub fn load_checkpoint(path: &Path) -> ValeResult<Vec<SweepResult>> {
    let text = std::fs::read_to_string(path).map_err(ValeError::Io)?;
    serde_json::from_str(&text).map_err(|e| ValeError::Parse(e.to_string()))
}

pub fn append_checkpoint(path: &Path, result: &SweepResult) -> ValeResult<()> {
    let mut existing = if path.exists() {
        load_checkpoint(path)?
    } else {
        Vec::new()
    };
    existing.push(result.clone());
    save_checkpoint(path, &existing)
}
