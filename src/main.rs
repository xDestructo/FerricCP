use anyhow::{Context, Result};
use std::{fs};
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

    /// What format the output is expected
    #[arg(short = 'F', long, default_value_t = diagnostics::OutputFormat::Text)]
    format: diagnostics::OutputFormat
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log = |msg: &str| {
        eprintln!("{}", msg); // stderr only now
    };

    log("init");

    let mut parser = Parser::new();
    let cpp = tree_sitter_cpp::LANGUAGE.into();
    parser.set_language(&cpp)
        .context("Failed to load tree-sitter C++ grammar")?;

    let rules_arr = config::load_rules(&args.rules)?;
    log(&format!("Loaded {} rules from {}.", rules_arr.len(), args.rules));

    let source_code = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", &args.file))?;

    log(&format!("Read {} ({} bytes).", &args.file, source_code.len()));

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();

    let mut violations = analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &cpp);
    
    diagnostics::output(&mut violations, args.format);

    Ok(())
}