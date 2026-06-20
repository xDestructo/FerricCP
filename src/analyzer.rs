use crate::config::RuleConfig;
use crate::diagnostics::Diagnostic;
use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator};

pub fn analyze(
    root_node: Node,
    source_code: &[u8],
    rules: &[RuleConfig],
    language: &Language,
) -> Vec<Diagnostic>
{
    let mut cursor = QueryCursor::new();
    let mut details_arr: Vec<Diagnostic> = Vec::new();

    for rule in rules {
        let query = match Query::new(language, &rule.query) {
            Ok(q) => q,
            Err(err) => {
                eprintln!("Failed to compile rule '{}'\n -> {}", rule.id, err);
                continue; 
            }
        };

        // mut for streaming iterator instead of regular iterator
        let mut matches = cursor.matches(&query, root_node, source_code);

        // using tree sitter's streaming iterator 
        while let Some(m) = matches.next() {
            
            for capture in m.captures {

                let capture_name = query.capture_names()[capture.index as usize];
                
                if capture_name != "violation" {
                    continue;
                }

                let node = capture.node;
                
                let start = node.start_position();
                // zero based indexing
                let line = start.row + 1;
                let column = start.column + 1;

                let snippet = std::str::from_utf8(&source_code[node.start_byte()..node.end_byte()])
                    .unwrap_or("<unreadable source>");

                details_arr.push(Diagnostic { 
                    line, 
                    column, 
                    id: rule.id.clone(), 
                    message: rule.message.clone(), 
                    severity: rule.severity.clone(), 
                    tip: rule.tip.clone(), 
                    snippet: snippet.to_string()
                });
            }
        }
    }

    details_arr
}