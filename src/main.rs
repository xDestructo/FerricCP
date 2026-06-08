use anyhow::{Context, Result};
use std::fs;
use tree_sitter::{Node, Parser};
use tree_sitter_cpp;

fn main() -> Result<()> {
    println!("Init");

    let mut parser = Parser::new();
    let cpp = tree_sitter_cpp::LANGUAGE.into();
    parser.set_language(&cpp)
        .context("Failed to load tree-sitter C++ grammar")?;

    let path = "test.cpp";
    let source_code = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;

    println!("Read {} ({} bytes).", path, source_code.len());

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();
    
    check_for_goto(root_node, source_code.as_bytes());

    Ok(())
}

fn check_for_goto(node: Node, source: &[u8]) {
    if node.kind() == "goto_statement" {
        let start = node.start_position();
        
        let line = start.row + 1;
        let column = start.column + 1;

        let snippet = std::str::from_utf8(&source[node.start_byte()..node.end_byte()])
            .unwrap_or("<unreadable source>");

        println!("WARNING: Goto detected");
        println!("Rule: ban-goto");
        println!("Location: Line {}, Column {}", line, column);
        println!("Code: `{}`\n", snippet);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        check_for_goto(child, source);
    }
}