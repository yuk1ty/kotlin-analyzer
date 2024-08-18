package analyzer

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.help
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.int
import com.github.michaelbull.result.binding
import com.github.michaelbull.result.getOrThrow
import org.eclipse.lsp4j.launch.LSPLauncher
import java.util.concurrent.Executors
import com.github.michaelbull.result.runCatching
import internal.KotlinTextDocumentService
import internal.KotlinWorkspaceService
import internal.analyzer.configuration.ServerConfig
import internal.analyzer.configuration.openStream
import io.github.oshai.kotlinlogging.KotlinLogging

val logger = KotlinLogging.logger {}

class KotlinAnalyzer : CliktCommand("ktanalyzer") {
    /** See more details in [internal.analyzer.configuration.TcpClientConfig] */
    private val tcpClientHost: String? by option(
        "--tcp-host",
        "-h"
    ).help("The host of the TCP client (mainly for VSCode)")

    /** See more details in [internal.analyzer.configuration.TcpClientConfig] */
    private val tcpClientPort: Int? by option("--tcp-port", "-p").int()
        .help("The port of the TCP client (mainly for VSCode)")

    override fun run() {
        val serverConfig = ServerConfig.from(tcpClientHost, tcpClientPort)
        val (systemIn, systemOut) = openStream(serverConfig.tcpClient)
        val ec = Executors.newCachedThreadPool()

        val textDocumentService = KotlinTextDocumentService()
        val workspaceService = KotlinWorkspaceService()
        val server = KotlinAnalyzerServer(textDocumentService, workspaceService, serverConfig)

        binding {
            val launcher =
                runCatching { LSPLauncher.createServerLauncher(server, systemIn, systemOut, ec) { it } }.bind()
            logger.info { "Server is starting" }
            server.connect(launcher.remoteProxy)
            runCatching { launcher.startListening().get() }
        }.getOrThrow()
    }

}

fun main(args: Array<String>) {
    KotlinAnalyzer().main(args)
}