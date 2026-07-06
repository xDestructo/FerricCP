use std::fmt;
use serde::Serialize;
use clap::ValueEnum;
use tree_sitter::Node;
use crate::config::RuleConfig;

#[derive(Serialize)]
pub struct Diagnostic {
    pub id: String,
    pub message: String,
    pub severity: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub tip: Option<String>,
    pub snippet: String,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
pub fn build_diagnostic(
    node: Node,
    source: &[u8],
    rule: &RuleConfig,
    var_name: Option<&str>,
) -> Diagnostic 
{
    let start = node.start_position();
    let end = node.end_position(); 

    let message = match var_name {
        Some(v) => rule.message.replace("{var}", v),
        None => rule.message.clone(),
    };

    let tip = rule.tip.as_ref().map(|t| match var_name {
        Some(v) => t.replace("{var}", v),
        None => t.clone(),
    });

    Diagnostic {
        id: rule.id.clone(),
        message,
        severity: rule.severity.clone(),
        line: start.row,
        column: start.column,
        end_line: end.row,      
        end_column: end.column,  
        tip,
        snippet: node.utf8_text(source).unwrap_or("<unreadable>").to_string(),
    }
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
        
        println!("Location: Line {}, Column {} to Line {}, Column {}", 
            elem.line + 1, 
            elem.column + 1, 
            elem.end_line + 1, 
            elem.end_column + 1
        );
        
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