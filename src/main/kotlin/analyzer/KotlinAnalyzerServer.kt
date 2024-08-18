package analyzer

import internal.KotlinTextDocumentService
import internal.KotlinWorkspaceService
import internal.analyzer.configuration.ServerConfig
import org.eclipse.lsp4j.*
import org.eclipse.lsp4j.services.*
import java.util.concurrent.CompletableFuture
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference

data class KotlinAnalyzerServer(
    val textDocumentService: KotlinTextDocumentService,
    val workspaceService: KotlinWorkspaceService,
    val serverConfig: ServerConfig
) : LanguageServer, LanguageClientAware {

    private val client: AtomicReference<LanguageClient?> = AtomicReference(null)

    private val isLanguageClientSet: AtomicBoolean = AtomicBoolean(false)

    override fun initialize(p0: InitializeParams?): CompletableFuture<InitializeResult> {
        val capabilities = ServerCapabilities().apply {
            setTextDocumentSync(TextDocumentSyncKind.Full)
        }
        client.get()?.logMessage(MessageParams(MessageType.Info, "Kotlin Analyzer Server has been initialized"))
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

    override fun connect(client: LanguageClient?) {
        if (isLanguageClientSet.compareAndSet(false, true)) {
            this.client.set(client)
        }
    }
}
