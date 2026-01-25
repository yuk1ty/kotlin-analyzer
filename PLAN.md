# Kotlin Language Server (Rust) Implementation Plan

## Requirements / Decisions
- Kotlin 2.3.0 (JVM only)
- Parser: chumsky
- Diagnostics: syntax errors only (initial phase)
- Build integration: Gradle first, classpath output task required
- Clients: VSCode (custom extension) + Neovim

## Architecture Outline
- LSP framework: tower-lsp
- Text model: ropey + UTF-16 position conversion
- Modules (crate-level):
  - `lsp`: server + routing + logging
  - `text`: document store + position mapping
  - `parser`: lexer + grammar + diagnostics conversion + AST
  - `analysis`: symbols
  - `index`: workspace indexing
  - `gradle`: classpath integration

## Milestones
1) Foundation (LSP skeleton + document model + diagnostics pipe)
2) chumsky lexer + minimal grammar with syntax diagnostics
3) Goto Definition (single file)
4) Gradle classpath output task + module support
5) Workspace indexing + VSCode/Neovim integration

## Detailed Plan (Week-based)

### Week 1 — Foundation (Done)
- [x] LSP server skeleton (`initialize`, `didOpen/didChange/didClose`, `definition` stub)
- [x] Document store (open/apply changes/close)
- [x] UTF-16 ↔︎ char position conversion
- [x] Diagnostics model + LSP conversion
- [x] Minimal E2E (server runs, diagnostics pipeline wired)
- [x] Unit tests for text positions + change application

### Week 2 — chumsky Parser (Done)
- [x] Lexer (keywords/idents/literals/punct/operators/comments)
- [x] Minimal grammar (package/import/class/fun/val/var + balanced groups)
- [x] Syntax diagnostics from lexer/parser errors
- [x] Parser smoke tests
- [x] Error recovery improvements (nested delimiters + lexer recovery)
- [ ] Expand Kotlin 2.3.0 subset as needed (types/params/expressions)

### Week 3 — Goto Definition (Single File) (Done)
- [x] AST shape for declarations (class/fun/val/var)
- [x] Symbol table from AST
- [x] Cursor → identifier extraction
- [x] `textDocument/definition` implementation (same file)
- [x] AST/symbol index tests

### Week 4 — Gradle Integration (Classpath Task) (In Progress)
- [x] Gradle init script to output JSON (classpath + source roots)
- [x] Rust-side Gradle runner (process + error handling)
- [x] Multi-module support (rootProject.allprojects aggregation)
- [x] In-memory cache per workspace root (initial run on startup)
- [ ] Cache strategy (mtime/hash) + incremental refresh
- [ ] Fallback if Gradle fails (standard source roots)

### Week 5 — Workspace Index + Client Integration (In Progress)
- [x] Workspace-wide index of symbols
- [x] Definition across workspace (multiple locations)
- [ ] VSCode extension (server launch + config + packaging)
- [ ] Neovim config snippets + README
- [ ] Basic perf check on medium/large project

## Work Log (Summary)
- Added workspace index with periodic refresh and open-file exclusion.
- Implemented AST + symbol indexing; definition jumps within file and across workspace.
- Added Gradle classpath runner (init script + JSON output) and startup cache.

## Immediate Next Steps
1) Expand Kotlin grammar subset (types/params/expressions) as needed
2) Gradle cache refresh strategy (mtime/hash + manual trigger)
3) VSCode/Neovim integration and documentation
