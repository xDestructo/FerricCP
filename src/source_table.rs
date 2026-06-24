use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VariableState {
    pub declared_type: String, 
    
    pub is_initialized: bool,  
    
    pub is_mutated: bool,      
    
    pub read_count: u32,       
    
    pub line_declared: usize,  
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, VariableState>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], 
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    pub fn declare(&mut self, name: String, state: VariableState) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, state);
        }
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut VariableState> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.get_mut(name) {
                return Some(state);
            }
        }
        None // undeclared var
    }

}