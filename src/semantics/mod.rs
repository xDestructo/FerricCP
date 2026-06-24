use crate::symbol_table::SymbolTable;
use tree_sitter::Node;

pub mod cpp; 

pub fn build_symbol_table(root_node: Node, source: &[u8], extension: &str) -> SymbolTable {
    let mut table = SymbolTable::new();

    match extension {
        "cpp" | "cc" | "cxx" | "h" | "hpp" => {
            cpp::walk_ast(root_node, source, &mut table);
        }
        // "py" => python::walk_ast(root_node, source, &mut table), 
        
        _ => {
            println!("use cpp instead");
        }
    }

    table
}