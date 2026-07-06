use tree_sitter::Node;
use crate::dispatcher::SemanticRule;
use crate::symbol_table::SymbolTable;
use crate::diagnostics::{Diagnostic, build_diagnostic};
use crate::config::RuleConfig;

// id uninitialized_arr_bound
pub struct ArrayBoundRule;

impl SemanticRule for ArrayBoundRule {
    fn check(&self, node: Node, source: &[u8], table: &SymbolTable, rule: &RuleConfig) -> Option<Diagnostic> {
        let current_line = node.start_position().row + 1;
        let mut suspect_var = None;

        // Traverse the specific AST path from the YAML query
        // declaration -> declarator (function_declarator) -> parameters -> parameter_declaration
        // extract 'sz' from 'vector<int> arr(sz);'
        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(params) = declarator.child_by_field_name("parameters") {
                
                let mut cursor = params.walk();
                for child in params.children(&mut cursor) {
                    if child.kind() == "parameter_declaration" {
                        if let Ok(name) = child.utf8_text(source) {
                            suspect_var = Some(name.trim().to_string());
                        }
                    }
                }
            }
        }

        // query my symbol table
        if let Some(target_var) = suspect_var {
            for (key, state) in &table.registry {
                let var_name = key.split('_').next().unwrap_or(key);
                
                if var_name == target_var && state.line_declared < current_line {
                    if !state.is_initialized {
                        return Some(build_diagnostic(node, source, rule, Some(&var_name)));
                    }
                }
            }
        }
        None
    
    }
}

// id binary_op_type_mismatch
pub struct BinaryMismatchRule;

impl SemanticRule for BinaryMismatchRule {
    fn check(&self, _node: Node, _source: &[u8], _table: &SymbolTable, _rule: &RuleConfig) -> Option<Diagnostic> {
        // same
        None
    }
}

pub fn load_semantic_rules(dispatcher: &mut crate::dispatcher::RuleDispatcher) {
    dispatcher.register("uninitialized_arr_bound", Box::new(ArrayBoundRule));
    dispatcher.register("binary_op_type_mismatch", Box::new(BinaryMismatchRule));
}