package internal

import io.github.oshai.kotlinlogging.KotlinLogging
import org.eclipse.lsp4j.DidChangeTextDocumentParams
import org.eclipse.lsp4j.DidCloseTextDocumentParams
import org.eclipse.lsp4j.DidOpenTextDocumentParams
import org.eclipse.lsp4j.DidSaveTextDocumentParams
import org.eclipse.lsp4j.services.TextDocumentService

class KotlinTextDocumentService : TextDocumentService {

    companion object {
        private val logger = KotlinLogging.logger {}
    }

    override fun didOpen(params: DidOpenTextDocumentParams?) {
        logger.info { "hit did open!" }
    }

    override fun didChange(params: DidChangeTextDocumentParams?) {
        logger.info { "hit did change!" }
    }

    override fun didClose(params: DidCloseTextDocumentParams?) {
        logger.info { "hit did close!" }
    }

    override fun didSave(params: DidSaveTextDocumentParams?) {
        logger.info { "hit did save!" }
    }
}