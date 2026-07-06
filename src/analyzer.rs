use crate::symbol_table::SymbolTable;
use crate::config::RuleConfig;
use crate::diagnostics::{Diagnostic, build_diagnostic};
use tree_sitter::{Node, QueryCursor, StreamingIterator};
use crate::dispatcher::RuleDispatcher;
use crate::rules::load_semantic_rules;

pub fn analyze(
    root_node: Node,
    source_code: &[u8],
    rules: &[RuleConfig],
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
        let mut matches = cursor.matches(&rule.compiled_query, root_node, source_code);

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = rule.compiled_query.capture_names()[capture.index as usize];
                
                if capture_name != "violation" {
                    continue;
                }

                let node = capture.node;

                if rule.rule_type == "semantic" {
                    if let Some(diagnostic) = dispatcher.run_semantic_check(
                        &rule.id, node, source_code, symbol_table, rule
                    ) {
                        details_arr.push(diagnostic);
                    }
                } 
                else if rule.rule_type == "syntactic" {
                    details_arr.push(build_diagnostic(node, source_code, rule, None));
                }
            }
        }
    }

    details_arr
}