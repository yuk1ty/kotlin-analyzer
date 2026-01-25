#[tokio::main]
async fn main() {
    kotlin_analyzer::lsp::init_logging();

    if let Err(err) = kotlin_analyzer::lsp::run().await {
        eprintln!("LSP server failed: {err}");
    }
}
