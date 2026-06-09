use anyhow::{Context, Result};
use std::fs;
use tree_sitter::{Node, Parser};
use tree_sitter_cpp;
                             
pub mod config;
pub mod analyzer;

fn main() -> Result<()> {
    println!("Init");

    let mut parser = Parser::new();
    let cpp = tree_sitter_cpp::LANGUAGE.into();
    parser.set_language(&cpp)
        .context("Failed to load tree-sitter C++ grammar")?;

    let rule_path = "rules/cpp";
    let rules_arr = config::load_rules(rule_path)?;
    println!("Loaded {} rules from {}.", rules_arr.len(), rule_path);

    let path = "test.cpp";
    let source_code = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    println!("Read {} ({} bytes).", path, source_code.len());

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();
    
    analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &cpp);

    Ok(())
}