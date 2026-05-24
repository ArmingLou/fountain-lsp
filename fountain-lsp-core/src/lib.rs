pub mod lsp;
pub mod parser;
pub mod completion;

pub use lsp::Server;
pub use parser::FountainDocument;

mod logging {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    pub fn init() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

pub use logging::init;