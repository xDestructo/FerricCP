use anyhow::{Context, Result};
use std::{fs};
use tree_sitter::{Parser};
use tree_sitter_cpp;
use clap::Parser as ClapParser; 
use std::path::Path;

pub mod config;
pub mod analyzer;
pub mod diagnostics;
pub mod source_table;

/// FerricCP: A C++ static analyzer for Competitive Programming, built in Rust.
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The C++ source file to analyze
    #[arg(short, long)]
    file: String,

    /// Path to the rules directory
    #[arg(short, long, default_value = "rules")]
    rules: String,

    /// What format the output is expected
    #[arg(short = 'F', long, default_value_t = diagnostics::OutputFormat::Text)]
    format: diagnostics::OutputFormat,

    /// No logs printed to stderr
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log = |msg: &str| {
        if !args.quiet {
            eprintln!("{}", msg);
        }
    };

    log("init");


    let extension = Path::new(&args.file)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");


    let (language, lang_dir) = match extension {
        "cpp" | "cc" | "cxx" | "h" | "hpp" => (tree_sitter_cpp::LANGUAGE.into(), "cpp"),
        "py" => (tree_sitter_python::LANGUAGE.into(), "python"),
        _ => anyhow::bail!("Unsupported file extension: .{}", extension),
    };
    
    let mut parser = Parser::new();
    parser.set_language(&language)
        .context("Failed to load tree-sitter C++ grammar")?;


    let rules_path = format!("{}/{}", args.rules, lang_dir);
    let rules_arr = config::load_rules(&rules_path)?;
    log(&format!("Loaded {} rules from {}.", rules_arr.len(), args.rules));


    let source_code = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {}", &args.file))?;

    log(&format!("Read {} ({} bytes).", &args.file, source_code.len()));

    let tree = parser
        .parse(&source_code, None)
        .context("Failed to parse source code")?;

    let root_node = tree.root_node();

    let mut violations = analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &language);
    
    diagnostics::output(&mut violations, args.format);

    Ok(())
}