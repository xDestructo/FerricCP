use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Query};

#[derive(Debug, Deserialize)]
struct YamlRule {
    pub id: String,
    pub language: String,
    pub rule_type: String,
    pub severity: String,
    pub message: String,
    pub tip: Option<String>, 
    pub query: String,
}

pub struct RuleConfig {
    pub id: String,
    pub language: String,
    pub rule_type: String,
    pub severity: String,
    pub message: String,
    pub tip: Option<String>, 
    pub compiled_query: Query,
}

pub fn load_rules<P: AsRef<Path>>(dir_path: P, language: Language) -> Result<Vec<RuleConfig>> {
    let mut rules = Vec::new();
    let path = dir_path.as_ref();

    if !path.exists() || !path.is_dir() {
        return Ok(rules);
    }

    for entry in fs::read_dir(path).context("Failed to read rules directory")? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let file_contents = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {:?}", file_path))?;
            
            let raw_rule: YamlRule = serde_yaml::from_str(&file_contents)
                .with_context(|| format!("Failed to parse YAML in {:?}", file_path))?;
            
            let compiled = Query::new(&language, &raw_rule.query)
                .with_context(|| format!("Failed to compile query for rule '{}'", raw_rule.id))?;

            rules.push(RuleConfig {
                id: raw_rule.id,
                language: raw_rule.language,
                rule_type: raw_rule.rule_type,
                severity: raw_rule.severity,
                message: raw_rule.message,
                tip: raw_rule.tip,
                compiled_query: compiled,
            });
        }
    }

    Ok(rules)
}