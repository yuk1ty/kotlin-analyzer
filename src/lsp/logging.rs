pub fn init_logging() {
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "kotlin_analyzer=info,tower_lsp=info".to_string());

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init();
}
