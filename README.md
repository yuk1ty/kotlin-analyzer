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

## Editor integration

### VSCode (custom extension)

This repo does not ship a VSCode extension. If you build one, point it at the stdio
server and wire the Gradle refresh command.

Minimal command wiring (TypeScript):

```ts
import { workspace, commands, Uri } from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

function registerGradleRefresh(client: LanguageClient) {
  return commands.registerCommand("kotlin-analyzer.refreshGradle", async () => {
    const root = workspace.workspaceFolders?.[0]?.uri.toString();
    await client.sendRequest("workspace/executeCommand", {
      command: "kotlin-analyzer.refreshGradle",
      arguments: root ? [root] : [],
    });
  });
}
```

### Neovim (nvim-lspconfig)

```lua
local lspconfig = require("lspconfig")

lspconfig.kotlin_analyzer = {
  default_config = {
    cmd = { "kotlin-analyzer" },
    filetypes = { "kotlin" },
    root_dir = lspconfig.util.root_pattern("settings.gradle", "settings.gradle.kts", ".git"),
  },
}

lspconfig.kotlin_analyzer.setup({})

-- manual Gradle refresh
vim.api.nvim_create_user_command("KotlinGradleRefresh", function()
  local root = vim.fn.getcwd()
  vim.lsp.buf.execute_command({
    command = "kotlin-analyzer.refreshGradle",
    arguments = { vim.uri_from_fname(root) },
  })
end, {})
```

## Gradle cache refresh

Gradle classpath caching is refreshed on startup and periodically if Gradle files
change. You can also trigger it manually via the LSP command:

```
command: kotlin-analyzer.refreshGradle
arguments: [workspace root URI or path]
```

## Roadmap

See `PLAN.md` for the detailed implementation plan and current progress.
