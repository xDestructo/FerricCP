use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    pub language: String,
    pub rule_type: String,
    pub severity: String,
    pub message: String,
    pub tip: Option<String>, 
    pub query: String,
}

pub fn load_rules<P: AsRef<Path>>(dir_path: P) -> Result<Vec<RuleConfig>> {
    let mut rules = Vec::new();
    let path = dir_path.as_ref();

    if !path.exists() || !path.is_dir() {
        return Ok(rules); // Return empty if the directory doesn't exist
    }

    for entry in fs::read_dir(path).context("Failed to read rules directory")? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let file_contents = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {:?}", file_path))?;
            
            let rule: RuleConfig = serde_yaml::from_str(&file_contents)
                .with_context(|| format!("Failed to parse YAML in {:?}", file_path))?;
            
            rules.push(rule);
        }
    }

    Ok(rules)
}