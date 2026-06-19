pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub message: String,
    pub severity: String,
    pub tip: Option<String>,
    pub snippet: String,
}

pub fn print_cli(details_arr: &mut Vec<Diagnostic>) {
    if details_arr.is_empty() {
        println!("No issues found, wow awesome code.");
        return;
    }
    
    details_arr.sort_by_key(|x| (x.line, x.column));

    for elem in details_arr {
        println!("WARNING: {} ({})", elem.message, elem.severity.to_uppercase());
        println!("Rule: {}", elem.id);
        
        if let Some(tip) = &elem.tip {
            println!("Tip: {}", tip);
        }
        
        println!("Location: Line {}, Column {}", elem.line, elem.column);
        println!("Code: `{}`\n", elem.snippet);
    }
}