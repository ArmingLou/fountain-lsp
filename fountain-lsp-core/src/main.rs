use tower_lsp::{LspService, Server as LspServer};
use fountain_lsp_core::{init, Server as FountainServer};

#[tokio::main]
async fn main() {
    init();

    tracing::info!("Starting Fountain LSP Server...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|_client| FountainServer::new())
    .finish();

    LspServer::new(stdin, stdout, socket).serve(service).await;
}