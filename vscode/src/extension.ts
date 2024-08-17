import * as path from "path";
import { workspace, ExtensionContext } from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import * as vscode from "vscode";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const javaHome = process.env["JAVA_HOME"];
  if (!javaHome) {
    vscode.window.showErrorMessage("JAVA_HOME is not set");
    return;
  }
  vscode.window.showInformationMessage(`Detected JAVA_HOME ${javaHome}`);

  const javaExecutable: Executable = {
    command: javaHome,
    // TODO: make this configurable
    args: ["-jar", "../../build/libs/kotlin-analyzer-0.1.0.jar"],
  };
  const serverOptions: ServerOptions = {
    run: javaExecutable,
    debug: javaExecutable,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "kotlin" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };

  client = new LanguageClient(
    "vscodeKotlinAnalyzerClient",
    "Kotlin Analyzer VScode Client",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
