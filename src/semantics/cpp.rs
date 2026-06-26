use crate::symbol_table::{SymbolTable, VariableState};
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
            "declaration" => {extract_variables(node, source, table);}
            
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

fn extract_variables(node: Node, source: &[u8], table: &mut SymbolTable) {
    let Some(type_node) = node.child_by_field_name("type") else { return; };
    let Ok(declared_type) = type_node.utf8_text(source) else { return; };
    let declared_type = declared_type.trim().to_string();

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let kind = child.kind();
        
        // uninitialized var eg: int x, y, *z;
        if matches!(kind, "identifier" | "array_declarator" | "pointer_declarator" | "reference_declarator") {
            if let Some(name) = extract_identifier_name(child, source) {
                table.declare_var(name, VariableState {
                    declared_type: declared_type.clone(),
                    is_initialized: false, 
                    is_mutated: false,
                    read_count: 0,
                    line_declared: node.start_position().row + 1, 
                });
            }
        } 
        // initialized var: int x = 5;
        else if kind == "init_declarator" {
            if let Some(decl_node) = child.child_by_field_name("declarator") {
                if let Some(name) = extract_identifier_name(decl_node, source) {
                    table.declare_var(name, VariableState {
                        declared_type: declared_type.clone(),
                        is_initialized: true, 
                        is_mutated: false,
                        read_count: 0,
                        line_declared: node.start_position().row + 1,
                    });
                }
            }
        }
    }
}

/// get var name from ast recursively
fn extract_identifier_name(node: Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => node.utf8_text(source).ok().map(|s| s.trim().to_string()),
        "array_declarator" | "pointer_declarator" | "reference_declarator" => {
            node.child_by_field_name("declarator")
                .and_then(|n| extract_identifier_name(n, source))
        },
        _ => None
    }
}