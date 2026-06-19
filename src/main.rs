use anyhow::{Context, Result};
use std::fs;
use tree_sitter::{Parser};
use tree_sitter_cpp;
use clap::Parser as ClapParser; 

pub mod config;
pub mod analyzer;
pub mod diagnostics;

/// FerricCP: A C++ static analyzer for Competitive Programming, built in Rust.
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The C++ source file to analyze
    #[arg(short, long)]
    file: String,

    /// Path to the rules directory
    #[arg(short, long, default_value = "rules/cpp")]
    rules: String,
}

fn main() -> Result<()> {
    println!("Init");

    let args = Args::parse();


    let mut parser = Parser::new();
    let cpp = tree_sitter_cpp::LANGUAGE.into();
    parser.set_language(&cpp)
        .context("Failed to load tree-sitter C++ grammar")?;

    let rules_arr = config::load_rules(&args.rules)?;
    println!("Loaded {} rules from {}.", rules_arr.len(), &args.rules);

    let source_code = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", &args.file))?;

    println!("Read {} ({} bytes).", &args.file, source_code.len());

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();
    
    diagnostics::print_cli(&mut analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &cpp));

    Ok(())
}