use anyhow::{Context, Result};
use std::fs;
use tree_sitter::Parser;
use clap::{Parser as ClapParser, Subcommand}; 
use std::path::Path;
use lsp::FerricLsp;
use tower_lsp::{LspService, Server};

pub mod config;
pub mod analyzer;
pub mod diagnostics;
pub mod symbol_table;
pub mod semantics;
pub mod dispatcher;
pub mod rules;
pub mod lsp; 

/// FerricCP: A C++ static analyzer for Competitive Programming, built in Rust.
#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze a file in the terminal
    Analyze {
        /// The source file to analyze
        #[arg(short, long)]
        file: String,

        /// [Optional] Path to the rules directory
        #[arg(short, long, default_value = "rules")]
        rules: String,

        /// [Optional ]What format the output is expected
        #[arg(short = 'F', long, default_value_t = diagnostics::OutputFormat::Text)]
        format: diagnostics::OutputFormat,

        /// [Optional] No logs printed to stderr
        #[arg(short, long)]
        quiet: bool,
    },
    /// Boot the Language Server Protocol (LSP) for VS Code integration
    Lsp,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { file, rules, format, quiet } => {
            let log = |msg: &str| {
                if !quiet {
                    eprintln!("{}", msg);
                }
            };

            log("init");

            let extension = Path::new(&file)
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

            let rules_path = format!("{}/{}", rules, lang_dir);
            let rules_arr = config::load_rules(&rules_path, language)?;
            log(&format!("Loaded {} rules from {}.", rules_arr.len(), rules));

            let source_code = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read {}", &file))?;

            log(&format!("Read {} ({} bytes).", &file, source_code.len()));

            let tree = parser
                .parse(&source_code, None)
                .context("Failed to parse source code")?;

            let root_node = tree.root_node();
            let symbol_table = semantics::build_symbol_table(root_node, source_code.as_bytes(), extension);

            let mut violations = analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &symbol_table);
            diagnostics::output(&mut violations, format);
        }
        Commands::Lsp => {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(|client| FerricLsp { client });
            Server::new(stdin, stdout, socket).serve(service).await;
        }
    }

    Ok(())
}