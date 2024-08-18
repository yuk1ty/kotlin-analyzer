package internal.analyzer.configuration

import java.io.InputStream
import java.io.OutputStream
import java.net.Socket

/**
 * Generic configuration for the server.
 */
data class ServerConfig(val tcpClient: TcpClientConfig?) {
    companion object {
        fun from(host: String?, port: Int?): ServerConfig {
            val tcpClient = if (host != null && port != null) {
                TcpClientConfig(host, port)
            } else null
            return ServerConfig(tcpClient)
        }
    }
}

/**
 * Configuration for a TCP client. If we set the configuration, the server will
 * connect to the specified host and port. This is an optional connection for
 * VSCode.
 */
data class TcpClientConfig(val host: String, val port: Int) {
    internal fun asSocket(): Socket = Socket(host, port)
}

/**
 * Open a stream for the server. If the TCP client is set, the server will
 * connect to the specified host and port. Otherwise, the server will use the
 * standard input and output.
 */
fun openStream(tcpClient: TcpClientConfig?): Pair<InputStream, OutputStream> {
    tcpClient?.let { client ->
        val socket = client.asSocket()
        return Pair(socket.getInputStream(), socket.getOutputStream())
    } ?: return Pair(System.`in`, System.out)
}