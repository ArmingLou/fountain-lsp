use tower_lsp::lsp_types::*;
use crate::parser::{FountainDocument, DocumentStore};
use crate::completion::CompletionProvider;

pub struct FountainLanguageHandler {
    capabilities: ServerCapabilities,
    documents: DocumentStore,
    completion_provider: CompletionProvider,
}

impl FountainLanguageHandler {
    pub fn new() -> Self {
        let documents = DocumentStore::new();
        let completion_provider = CompletionProvider::new(documents.documents.clone());
        FountainLanguageHandler {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "@".to_string(),
                        "＞".to_string(),
                        ">".to_string(),
                        "。".to_string(),
                        ".".to_string(),
                        "》".to_string(),
                        "——".to_string(),
                        "【".to_string(),
                        "[".to_string(),
                        "-".to_string(),
                        "（".to_string(),
                        "(".to_string(),
                        "#".to_string(),
                        " ".to_string(),
                        "i".to_string(),
                        "I".to_string(),
                        "e".to_string(),
                        "E".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            documents,
            completion_provider,
        }
    }

    pub fn initialize(&mut self, _params: InitializeParams) -> InitializeResult {
        InitializeResult {
            capabilities: self.capabilities.clone(),
            server_info: Some(ServerInfo {
                name: "fountain-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            ..InitializeResult::default()
        }
    }

    pub async fn shutdown(&self) {
    }

    pub async fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        let version = params.text_document.version;

        let mut doc = FountainDocument::new(uri.clone(), text, version);
        doc.parse();

        self.documents.insert(uri, doc).await;
    }

    pub async fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let mut doc = self.documents.get(&uri).await.unwrap();
            doc.text = change.text;
            doc.version = version;
            doc.parse();
            self.documents.update(uri, doc).await;
        }
    }

    pub async fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri).await;
    }

    pub async fn completion(&self, params: CompletionParams) -> Option<CompletionList> {
        self.completion_provider.provide_completion(params).await
    }

    pub async fn hover(&self, params: HoverParams) -> Option<Hover> {
        let doc = self.documents.get(&params.text_document_position_params.text_document.uri.to_string()).await?;
        let parsed = doc.parsed.as_ref()?;
        let line = params.text_document_position_params.position.line;
        let hover_text = parsed.hover_at_line(line)?;
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_text,
            }),
            range: None,
        })
    }

    pub async fn document_symbol(&self, params: DocumentSymbolParams) -> Option<Vec<DocumentSymbol>> {
        let doc = self.documents.get(&params.text_document.uri.to_string()).await?;
        let parsed = doc.parsed.as_ref()?;
        Some(parsed.to_document_symbols())
    }

    pub async fn semantic_tokens_full(&self, _params: SemanticTokensParams) -> Option<SemanticTokensResult> {
        None
    }
}

impl Default for FountainLanguageHandler {
    fn default() -> Self {
        Self::new()
    }
}