//! ArchFlow Demo Server - Simple HTTP server for the demo page
//!
//! Serves the demo HTML page and WASM modules.
//! Run with: cargo run -p archflow-demo-server

use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Simple MIME type mapping
fn get_mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}

// Simple router
fn handle_request(
    method: &str,
    path: &str,
    body: Option<&str>,
    base_dir: &PathBuf,
) -> (u16, String, String) {
    // CORS headers
    let cors_headers = format!(
        "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n"
    );

    // Handle OPTIONS (preflight)
    if method == "OPTIONS" {
        return (200, cors_headers, String::new());
    }

    // API endpoints
    if path.starts_with("/api/") {
        return handle_api(method, path, body, cors_headers);
    }

    // Serve static files
    let mut file_path = base_dir.clone();
    if path == "/" || path.is_empty() {
        file_path.push("index.html");
    } else {
        file_path.push(&path[1..]); // Remove leading slash
    }

    // Security: prevent directory traversal
    if !file_path.starts_with(base_dir) {
        return (403, cors_headers, "Forbidden".to_string());
    }

    match fs::read(&file_path) {
        Ok(content) => {
            let mime = get_mime_type(&file_path.to_string_lossy());
            let headers = format!(
                "{}Content-Type: {}\r\nContent-Length: {}\r\n",
                cors_headers,
                mime,
                content.len()
            );
            (200, headers, String::from_utf8_lossy(&content).into_owned())
        }
        Err(_) => {
            let not_found = "404 Not Found";
            let headers = format!(
                "{}Content-Type: text/plain\r\nContent-Length: {}\r\n",
                cors_headers,
                not_found.len()
            );
            (404, headers, not_found.to_string())
        }
    }
}

// Simple API handler
fn handle_api(
    method: &str,
    path: &str,
    _body: Option<&str>,
    cors_headers: String,
) -> (u16, String, String) {
    match path {
        "/api/health" => {
            let response = r#"{"status": "ok", "version": "2.0.0"}"#;
            let headers = format!(
                "{}Content-Type: application/json\r\nContent-Length: {}\r\n",
                cors_headers,
                response.len()
            );
            (200, headers, response.to_string())
        }
        "/api/shapes" if method == "GET" => {
            // Return demo shapes
            let response = "[{\"id\": \"rect-1\", \"type\": \"rectangle\", \"x\": 100, \"y\": 100, \"width\": 120, \"height\": 80, \"color\": \"#3498db\"},{\"id\": \"ellipse-1\", \"type\": \"ellipse\", \"x\": 300, \"y\": 150, \"width\": 80, \"height\": 80, \"color\": \"#9b59b6\"},{\"id\": \"rect-2\", \"type\": \"rectangle\", \"x\": 500, \"y\": 120, \"width\": 100, \"height\": 100, \"color\": \"#e74c3c\"}]";
            let headers = format!(
                "{}Content-Type: application/json\r\nContent-Length: {}\r\n",
                cors_headers,
                response.len()
            );
            (200, headers, response.to_string())
        }
        _ => {
            let not_found = r#"{"error": "Not found"}"#;
            let headers = format!(
                "{}Content-Type: application/json\r\nContent-Length: {}\r\n",
                cors_headers,
                not_found.len()
            );
            (404, headers, not_found.to_string())
        }
    }
}

// Parse HTTP request
fn parse_request(data: &str) -> (String, String, Option<&str>, usize) {
    let mut lines = data.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.splitn(3, ' ');

    let method = parts.next().unwrap_or("GET").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let _version = parts.next().unwrap_or("HTTP/1.1");

    // Find body
    let mut body_start = 0;
    for (i, line) in data.lines().enumerate() {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
    }

    let body = if body_start > 0 && body_start < data.lines().count() {
        Some(data.lines().nth(body_start).unwrap_or(""))
    } else {
        None
    };

    (method, path, body, body_start)
}

// Format HTTP response
fn format_response(status_code: u16, headers: &str, body: &str) -> String {
    let status_text = match status_code {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    format!(
        "HTTP/1.1 {} {}\r\n{}\r\n{}",
        status_code, status_text, headers, body
    )
}

fn handle_client(mut stream: &std::net::TcpStream, base_dir: PathBuf) -> io::Result<()> {
    let mut buffer = [0; 8192];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let (method, path, body, _) = parse_request(&request);

    // Check for proper HTTP request format
    if !request.starts_with("HTTP/")
        && !request.starts_with("GET ")
        && !request.starts_with("POST ")
        && !request.starts_with("OPTIONS ")
    {
        let response = format_response(400, "Content-Type: text/plain\r\n", "Bad Request");
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let (status, headers, body) = handle_request(&method, &path, body, &base_dir);
    let response = format_response(status, &headers, &body);

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn main() {
    // Determine base directory - serve from demo directory
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("demo");

    println!("\n🧪 ArchFlow Demo Server v2.0");
    println!("==================================================");
    println!("📁 Serving files from: {:?}", base_dir);
    println!("🌐 Server running at: http://localhost:8080");
    println!("");
    println!("Available endpoints:");
    println!("  GET  /              - Demo page");
    println!("  GET  /api/health    - Health check");
    println!("  GET  /api/shapes    - Get demo shapes");
    println!("");
    println!("Press Ctrl+C to stop the server");
    println!("==================================================\n");

    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let base_dir = base_dir.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(&stream, base_dir) {
                        eprintln!("Error handling request: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
