use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use std::fs;
use tree_sitter::Parser;

use crate::{analyzer, config, semantics};

#[derive(Debug)]
pub struct FerricLsp {
    pub client: Client,
    pub rules_dir: String,
}

#[tower_lsp::async_trait]
impl LanguageServer for FerricLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        ..Default::default()
                    }
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "FerricCP LSP init.").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let file_path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => return, 
        };
        
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let (language, lang_dir) = match extension {
            "cpp" | "cc" | "cxx" | "h" | "hpp" => (tree_sitter_cpp::LANGUAGE.into(), "cpp"),
            // "py" => (tree_sitter_python::LANGUAGE.into(), "python"), 
            _ => return, 
        };

        let source_code = match fs::read_to_string(&file_path) {
            Ok(code) => code,
            Err(_) => return,
        };

        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
            return;
        }
        
        let tree = match parser.parse(&source_code, None) {
            Some(t) => t,
            None => return,
        };
        let root_node = tree.root_node();

        let rules_path = format!("{}/{}", self.rules_dir, lang_dir);
        let rules_arr = config::load_rules(&rules_path, language).unwrap_or_default();

        let symbol_table = semantics::build_symbol_table(root_node, source_code.as_bytes(), extension);
        let violations = analyzer::analyze(root_node, source_code.as_bytes(), &rules_arr, &symbol_table);

        let mut vs_code_diagnostics = Vec::new();

        for violation in violations {
            let start_line = violation.line as u32;
            let start_col = violation.column as u32;
            let end_line = violation.end_line as u32;
            let end_col = violation.end_column as u32;

            
            let severity_lower = violation.severity.to_lowercase();
            let diag_severity = match severity_lower.as_str() {
                "error" => DiagnosticSeverity::ERROR,
                "warning" => DiagnosticSeverity::WARNING,
                _ => DiagnosticSeverity::INFORMATION,
            };

            let diagnostic = Diagnostic {
                range: Range::new(
                    Position::new(start_line, start_col),
                    Position::new(end_line, end_col), 
                ),
                severity: Some(diag_severity),
                code: Some(NumberOrString::String(violation.id)),
                source: Some("FerricCP".to_string()),
                message: violation.message,
                ..Default::default()
            };
            vs_code_diagnostics.push(diagnostic);
        }

        self.client.publish_diagnostics(uri, vs_code_diagnostics, None).await;
    }
}