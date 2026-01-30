//! ArchFlow Demo Server
//!
//! Simple HTTP server with COOP/COEP headers required for SharedArrayBuffer support.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Handles an HTTP request
async fn handle_request(stream: &mut TcpStream, dir: &PathBuf, path: &str) -> std::io::Result<()> {
    let file_path = if path == "/" || path.is_empty() {
        dir.join("index.html")
    } else {
        dir.join(&path[1..]) // Remove leading slash
    };

    if file_path.exists() && file_path.is_file() {
        let mut file = File::open(&file_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        let content_type = match file_path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html",
            Some("js") => "application/javascript",
            Some("wasm") => "application/wasm",
            Some("css") => "text/css",
            Some("json") => "application/json",
            _ => "application/octet-stream",
        };

        let response_header = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Cross-Origin-Opener-Policy: same-origin\r\n\
             Cross-Origin-Embedder-Policy: require-corp\r\n\
             \r\n",
            content_type,
            contents.len()
        );

        stream.write_all(response_header.as_bytes()).await?;
        stream.write_all(&contents).await?;
    } else {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(())
}

/// Handles a client connection
async fn handle_client(mut stream: TcpStream, dir: Arc<PathBuf>) {
    let mut buffer = [0; 4096];
    let _ = stream.read(&mut buffer).await;

    let request = String::from_utf8_lossy(&buffer);
    let parts: Vec<&str> = request.split("\r\n").collect();

    if parts.is_empty() {
        return;
    }

    let request_line = parts[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();

    if parts.len() < 2 {
        return;
    }

    let _method = parts[0];
    let path = parts[1];

    if let Err(e) = handle_request(&mut stream, &dir, path).await {
        eprintln!("Error handling request: {}", e);
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut dir = PathBuf::from("demo");
    let mut port = 8080;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" if i + 1 < args.len() => {
                dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or(8080);
                i += 2;
            }
            _ => i += 1,
        }
    }

    if !dir.exists() {
        eprintln!("Directory not found: {:?}", dir);
        std::process::exit(1);
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let dir = Arc::new(dir);

    println!("");
    println!("  ╔════════════════════════════════════════╗");
    println!("  ║       ArchFlow Demo Server             ║");
    println!("  ╠════════════════════════════════════════╣");
    println!("  ║  Serving: {:?}", dir);
    println!("  ║  Address: http://localhost:{}", port);
    println!("  ╠════════════════════════════════════════╣");
    println!("  ║  COOP/COEP headers: ENABLED            ║");
    println!("  ║  SharedArrayBuffer: SUPPORTED          ║");
    println!("  ╠════════════════════════════════════════╣");
    println!("  ║  Press Ctrl+C to stop                  ║");
    println!("  ╚════════════════════════════════════════╝");
    println!("");

    loop {
        let (stream, _addr) = listener.accept().await?;
        let dir = Arc::clone(&dir);
        tokio::spawn(async move {
            handle_client(stream, dir).await;
        });
    }
}
