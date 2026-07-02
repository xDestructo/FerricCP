use std::collections::HashMap;
use tree_sitter::Node;
use crate::config::RuleConfig;
use crate::symbol_table::SymbolTable;
use crate::diagnostics::Diagnostic;

pub trait SemanticRule {
    fn check(&self, node: Node, source: &[u8], table: &SymbolTable, rule: &RuleConfig) -> Option<Diagnostic>;
}

pub struct RuleDispatcher {
    pub checkers: HashMap<String, Box<dyn SemanticRule>>,
}

impl RuleDispatcher {
    pub fn new() -> Self {
        Self { checkers: HashMap::new() }
    }

    /// hashmaps a YAML string ID to a Rust struct
    pub fn register(&mut self, yaml_id: &str, rule: Box<dyn SemanticRule>) {
        self.checkers.insert(yaml_id.to_string(), rule);
    }

    /// Forwards to corresponding rule struct's checker func if exists
    pub fn run_semantic_check(
        &self, 
        yaml_id: &str, 
        node: Node, 
        source: &[u8], 
        table: &SymbolTable,
        rule: &RuleConfig
    ) -> Option<Diagnostic> {
        if let Some(checker) = self.checkers.get(yaml_id) {
            checker.check(node, source, table, rule)
        } else {
            None
        }
    }
}