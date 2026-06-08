use anyhow::{Context, Result};
use std::fs;
use tree_sitter::Parser;
use tree_sitter_cpp;

fn main() -> Result<()> {
    println!("Init\n");

    let mut parser = Parser::new();
    let cpp = tree_sitter_cpp::LANGUAGE.into();
    parser
        .set_language(&cpp)
        .context("Failed to load tree-sitter C++ grammar")?;

    let file_path = "test.cpp";
    let source_code = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path))?;

    println!("=> Successfully read {} ({} bytes).", file_path, source_code.len());

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();

    println!("AST:\n");
    
    println!("{}", root_node.to_sexp());

    Ok(())
}