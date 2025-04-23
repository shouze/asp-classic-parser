use env_logger::init;
use log::{error, info};
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::TcpListener,
};
use tower_lsp::{LspService, Server};

use asp_classic_parser::lsp::AspLspServer;

#[tokio::main]
async fn main() {
    // Initialize the logger
    init();

    // Log startup message
    info!("Starting ASP Classic Language Server...");

    // Get connection type from environment or command line arguments
    if cfg!(windows) && std::env::args().any(|arg| arg == "--stdio") {
        // Windows with explicit --stdio arg
        info!("Using stdio connection");
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        start_server(stdin, stdout).await;
    } else if let Ok(port) = std::env::var("ASP_LSP_PORT") {
        // TCP connection on specified port
        match port.parse::<u16>() {
            Ok(port_num) => {
                info!("Using TCP connection on port {}", port_num);
                let listener = TcpListener::bind(format!("127.0.0.1:{}", port_num))
                    .await
                    .expect("Failed to bind to port");
                let (stream, _) = listener
                    .accept()
                    .await
                    .expect("Failed to accept connection");
                let (read, write) = io::split(stream);
                start_server(read, write).await;
            }
            Err(_) => {
                error!("Invalid port number: {}", port);
                std::process::exit(1);
            }
        }
    } else {
        // Default to stdio
        info!("Using stdio connection (default)");
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        start_server(stdin, stdout).await;
    };
}

async fn start_server<I, O>(stdin: I, stdout: O)
where
    I: AsyncRead + Unpin + 'static,
    O: AsyncWrite + Unpin + 'static,
{
    // Create the language server instance
    let (service, socket) = LspService::new(|client| {
        let server = AspLspServer::new(client);
        info!("LSP server instance created");
        server
    });

    // Start the server
    info!("ASP Classic Language Server ready, handling messages...");
    Server::new(stdin, stdout, socket).serve(service).await;
}
