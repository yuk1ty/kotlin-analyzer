package analyzer

import com.github.michaelbull.result.binding
import com.github.michaelbull.result.getOrThrow
import org.eclipse.lsp4j.launch.LSPLauncher
import java.util.concurrent.Executors
import com.github.michaelbull.result.runCatching
import internal.KotlinTextDocumentService
import internal.KotlinWorkspaceService
import io.github.oshai.kotlinlogging.KotlinLogging

val logger = KotlinLogging.logger {}

fun main(args: Array<String>) {
    val systemIn = System.`in`
    val systemOut = System.out
    val ec = Executors.newCachedThreadPool()

    val textDocumentService = KotlinTextDocumentService()
    val workspaceService = KotlinWorkspaceService()
    val server = KotlinAnalyzerServer(textDocumentService, workspaceService)

    binding {
        val launcher = runCatching { LSPLauncher.createServerLauncher(server, systemIn, systemOut, ec) { it } }.bind()
        logger.info { "Server is starting" }
        server.connect(launcher.remoteProxy)
        runCatching { launcher.startListening().get() }
    }.getOrThrow()
}