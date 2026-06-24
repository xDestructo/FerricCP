use crate::symbol_table::SymbolTable;
use tree_sitter::Node;

pub fn walk_ast(root_node: Node, _source: &[u8], _table: &mut SymbolTable) {
    if root_node.kind() == "translation_unit" {
        println!("fnaf");
    }
}