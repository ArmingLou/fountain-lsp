use std::sync::Arc;
use tower_lsp::{LanguageServer, jsonrpc::Result, lsp_types::*};
use tokio::sync::Mutex;

mod handler;

pub use handler::FountainLanguageHandler;

pub struct Server {
    handler: Arc<Mutex<FountainLanguageHandler>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            handler: Arc::new(Mutex::new(FountainLanguageHandler::new())),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut handler = self.handler.lock().await;
        Ok(handler.initialize(params))
    }

    async fn shutdown(&self) -> Result<()> {
        let handler = self.handler.lock().await;
        handler.shutdown().await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut handler = self.handler.lock().await;
        handler.did_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut handler = self.handler.lock().await;
        handler.did_change(params).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut handler = self.handler.lock().await;
        handler.did_close(params).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let handler = self.handler.lock().await;
        Ok(handler.completion(params).await.map(CompletionResponse::List))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let handler = self.handler.lock().await;
        Ok(handler.hover(params).await)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let handler = self.handler.lock().await;
        Ok(handler.document_symbol(params).await.map(DocumentSymbolResponse::Nested))
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let handler = self.handler.lock().await;
        Ok(handler.semantic_tokens_full(params).await)
    }
}