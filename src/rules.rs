use tree_sitter::Node;
use crate::dispatcher::SemanticRule;
use crate::symbol_table::SymbolTable;
use crate::diagnostics::Diagnostic;

// id uninitialized_arr_bound
pub struct ArrayBoundRule;

impl SemanticRule for ArrayBoundRule {
    fn check(&self, _node: Node, _source: &[u8], _table: &SymbolTable) -> Option<Diagnostic> {
        // later
        None
    }
}

// id binary_op_type_mismatch
pub struct BinaryMismatchRule;

impl SemanticRule for BinaryMismatchRule {
    fn check(&self, _node: Node, _source: &[u8], _table: &SymbolTable) -> Option<Diagnostic> {
        // same
        None
    }
}

pub fn load_semantic_rules(dispatcher: &mut crate::dispatcher::RuleDispatcher) {
    dispatcher.register("uninitialized_arr_bound", Box::new(ArrayBoundRule));
    dispatcher.register("binary_op_type_mismatch", Box::new(BinaryMismatchRule));
}