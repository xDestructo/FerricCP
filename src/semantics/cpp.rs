use crate::symbol_table::{SymbolTable};
use tree_sitter::{Node, TreeCursor};


pub fn walk_ast(root_node: Node, source: &[u8], table: &mut SymbolTable) {
    let mut cursor = root_node.walk();
    traverse(&mut cursor, source, table);
}

fn traverse(cursor: &mut TreeCursor, source: &[u8], table: &mut SymbolTable) {
    loop {
        let node = cursor.node();
        let kind = node.kind();

        // '{'
        let is_scope = kind == "compound_statement";
        if is_scope {
            table.enter_scope();
        }

        // semantic extraction 
        match kind {
            // macros #define ll long long
            "preproc_def" => {
                if let (Some(name_node), Some(value_node)) = (node.child_by_field_name("name"), node.child_by_field_name("value")) {
                    if let (Ok(name), Ok(value)) = (name_node.utf8_text(source), value_node.utf8_text(source)) {
                        table.declare_alias(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
            
            // type aliases (using vi = std::vector<int>)
            "alias_declaration" => {
                if let (Some(name_node), Some(type_node)) = (node.child_by_field_name("name"), node.child_by_field_name("type")) {
                    if let (Ok(name), Ok(actual_type)) = (name_node.utf8_text(source), type_node.utf8_text(source)) {
                        table.declare_alias(name.trim().to_string(), actual_type.trim().to_string());
                    }
                }
            }
            
            // variable declaration
            "declaration" => {}
            
            _ => {}
        }

        if cursor.goto_first_child() {
            traverse(cursor, source, table);
            cursor.goto_parent(); 
        }

        // '}'
        if is_scope {
            table.exit_scope();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}