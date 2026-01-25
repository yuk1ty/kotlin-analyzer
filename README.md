# kotlin-analyzer

This is an experimental project to build a language server for Kotlin in Rust.

## Build & Test

```bash
cargo test
```

## Run (stdio)

```bash
cargo run
```

Set logs if needed:

```bash
RUST_LOG=kotlin_analyzer=info,tower_lsp=info cargo run
```

## Roadmap

See `PLAN.md` for the detailed implementation plan and current progress.
