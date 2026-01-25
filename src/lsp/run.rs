use std::error::Error;

use tokio::io::{stdin, stdout};
use tower_lsp::{LspService, Server};

use crate::lsp::server::Backend;

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
