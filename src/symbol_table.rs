use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VariableState {
    pub declared_type: String, 
    
    pub is_initialized: bool,  
    
    pub is_mutated: bool,      
    
    pub read_count: u32,       
    
    pub line_declared: usize,  
}

#[derive(Debug, Default)]
pub struct Scope {
    pub variables: HashMap<String, VariableState>,
    pub aliases: HashMap<String, String>, // Tracks things like ll -> long long
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()], 
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare_var(&mut self, name: String, state: VariableState) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.variables.insert(name, state);
        }
    }

    pub fn declare_alias(&mut self, alias: String, actual_type: String) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.aliases.insert(alias, actual_type);
        }
    }

    pub fn lookup_var(&mut self, name: &str) -> Option<&mut VariableState> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.variables.get_mut(name) {
                return Some(state);
            }
        }
        None // undeclared var
    }

    pub fn resolve_type(&self, type_name: &str) -> String {
        let mut current_type = type_name.to_string();
        
        for scope in self.scopes.iter().rev() {
            if let Some(actual_type) = scope.aliases.get(&current_type) {
                current_type = actual_type.clone();
            }
        }
        current_type
    }
}