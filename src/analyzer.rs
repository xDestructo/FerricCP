use crate::symbol_table::SymbolTable;
use crate::config::RuleConfig;
use crate::diagnostics::Diagnostic;
use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator};

use crate::dispatcher::RuleDispatcher;
use crate::rules::load_semantic_rules;

pub fn analyze(
    root_node: Node,
    source_code: &[u8],
    rules: &[RuleConfig],
    language: &Language,
    symbol_table: &SymbolTable
) -> Vec<Diagnostic> 
{
    let mut cursor = QueryCursor::new();
    let mut details_arr: Vec<Diagnostic> = Vec::new();

    let mut dispatcher = RuleDispatcher::new();
    load_semantic_rules(&mut dispatcher);

    // yam rule has corresponding rule struct
    for rule in rules {
        if rule.rule_type == "semantic" && !dispatcher.checkers.contains_key(&rule.id) {
            panic!("YAML need semantic rule '{}', but no Rust struct registered :(", rule.id);
        }
    }

    for rule in rules {
        let query = match Query::new(language, &rule.query) {
            Ok(q) => q,
            Err(err) => {
                eprintln!("bruh Failed to compile rule '{}'\n -> {}", rule.id, err);
                continue; 
            }
        };

        let mut matches = cursor.matches(&query, root_node, source_code);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                
                if capture_name != "violation" {
                    continue;
                }

                let node = capture.node;

                if rule.rule_type == "semantic" {
                    if let Some(diagnostic) = 
                            dispatcher.run_semantic_check(&rule.id, 
                                node, source_code, symbol_table, rule) {
                        details_arr.push(diagnostic);
                    }
                } 
                else if rule.rule_type == "syntactic" {
                    let start = node.start_position();
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
    }

    details_arr
}