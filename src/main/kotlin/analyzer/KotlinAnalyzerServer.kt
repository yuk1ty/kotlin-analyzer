package analyzer

import internal.KotlinTextDocumentService
import internal.KotlinWorkspaceService
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.services.LanguageServer
import org.eclipse.lsp4j.services.TextDocumentService
import org.eclipse.lsp4j.services.WorkspaceService
import java.util.concurrent.CompletableFuture

data class KotlinAnalyzerServer(
    val textDocumentService: KotlinTextDocumentService,
    val workspaceService: KotlinWorkspaceService
) : LanguageServer {
    override fun initialize(p0: InitializeParams?): CompletableFuture<InitializeResult> {
        val capabilities = ServerCapabilities().apply {
            setTextDocumentSync(TextDocumentSyncKind.Full)
        }
        return CompletableFuture.completedFuture(InitializeResult(capabilities, ServerInfo("kotlin-analyzer", "0.1.0")))
    }

    override fun shutdown(): CompletableFuture<Any> {
        TODO("Not yet implemented")
    }

    override fun exit() {
        TODO("Not yet implemented")
    }

    override fun getTextDocumentService(): TextDocumentService {
        return textDocumentService
    }

    override fun getWorkspaceService(): WorkspaceService {
        return workspaceService
    }
}
