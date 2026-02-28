use crate::models::Codex;
use std::fs;

pub struct Archivist;

impl Archivist {
    pub fn load(path: &str) -> Result<Codex, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let codex = serde_json::from_str(&contents)?;
        Ok(codex)
    }

    pub fn save(codex: &Codex, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(codex)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
