use std::fmt;

use serde::Serialize;
use clap::ValueEnum;

#[derive(Serialize)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub message: String,
    pub severity: String,
    pub tip: Option<String>,
    pub snippet: String,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

pub fn output(violations: &mut Vec<Diagnostic>, format: OutputFormat) {
    
    violations.sort_by_key(|x| (x.line, x.column));

    match format {
        OutputFormat::Text => print_cli(violations),
        OutputFormat::Json => print_json(violations),
    }
}

fn print_cli(violations: &mut Vec<Diagnostic>) {
    if violations.is_empty() {
        println!("No issues found, wow awesome code.");
        return;
    }
    
    for elem in violations {
        println!("WARNING: {} ({})", elem.message, elem.severity.to_uppercase());
        println!("Rule: {}", elem.id);
        
        if let Some(tip) = &elem.tip {
            println!("Tip: {}", tip);
        }
        
        println!("Location: Line {}, Column {}", elem.line, elem.column);
        println!("Code: `{}`\n", elem.snippet);
    }
}

fn print_json(violations: &[Diagnostic]) {
    match serde_json::to_string_pretty(violations) {
        Ok(json_output) => println!("{}", json_output),
        Err(err) => eprintln!("Failed to generate JSON output: {}", err),
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}