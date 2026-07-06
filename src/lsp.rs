use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

#[derive(Debug)]
pub struct FerricLsp {
    pub client: Client,
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
        let file_path = uri.path();
        
        self.client.log_message(MessageType::INFO, format!("Analyzing {}", file_path)).await;

        // fake violation, implement this later fr fr
        let diagnostic = Diagnostic::new(
            Range::new(Position::new(0, 0), Position::new(0, 5)),
            Some(DiagnosticSeverity::ERROR),
            Some(NumberOrString::String("ferric_001".to_string())),
            Some("FerricCP".to_string()),
            "Uninitialized variable detected.".to_string(),
            None,
            None,
        );

        self.client.publish_diagnostics(uri, vec![diagnostic], None).await;
    }
}