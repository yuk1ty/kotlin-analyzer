package analyzer

import com.github.michaelbull.result.binding
import com.github.michaelbull.result.getOrThrow
import org.eclipse.lsp4j.launch.LSPLauncher
import java.util.concurrent.Executors
import com.github.michaelbull.result.runCatching

fun main(args: Array<String>) {
    val systemIn = System.`in`
    val systemOut = System.out
    val ec = Executors.newCachedThreadPool()
    val server = KotlinAnalyzerServer()
    // TODO: Need to implement text document service
    binding {
        val launcher = runCatching { LSPLauncher.createServerLauncher(server, systemIn, systemOut, ec) { it } }.bind()
        runCatching { launcher.startListening().get() }
    }.getOrThrow()
}