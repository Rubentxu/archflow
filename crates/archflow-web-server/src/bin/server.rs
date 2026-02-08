// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web Server - Binary entrypoint
//
// Run the ArchFlow web server with:
//   cargo run -p archflow-web-server --bin server
//
// Or build and run:
//   cargo build -p archflow-web-server --release --bin server
//   ./target/release/server
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_web_server::{ServerConfig, run_server};
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let mut config = ServerConfig::default();

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" | "-h" => {
                if i + 1 < args.len() {
                    config.host = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --host requires an argument");
                    std::process::exit(1);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse().expect("Invalid port number");
                    i += 2;
                } else {
                    eprintln!("Error: --port requires an argument");
                    std::process::exit(1);
                }
            }
            "--dist" | "-d" => {
                if i + 1 < args.len() {
                    config.ui_dist_path = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --dist requires an argument");
                    std::process::exit(1);
                }
            }
            "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    // Verify dist directory exists
    if !config.ui_dist_path.try_exists().unwrap_or(false) {
        eprintln!(
            "Error: UI dist directory not found: {:?}",
            config.ui_dist_path
        );
        eprintln!("Hint: Run 'npm run build' in crates/archflow-web-ui first");
        std::process::exit(1);
    }

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!(
        "║           ArchFlow Web Server v{}                         ║",
        env!("CARGO_PKG_VERSION")
    );
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  Host: {:50} ║", config.host);
    println!("║  Port: {:50} ║", config.port);
    println!("║  UI:   {:50} ║", config.ui_dist_path.display());
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Server starting at http://{}:{}", config.host, config.port);
    println!("Press Ctrl+C to stop");
    println!();

    run_server(config).await
}

fn print_usage() {
    println!("Usage: server [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --host <ADDRESS>    Host address to bind to (default: 127.0.0.1)");
    println!("  -p, --port <PORT>       Port to listen on (default: 3000)");
    println!("  -d, --dist <PATH>       Path to React UI dist directory");
    println!("                          (default: crates/archflow-web-ui/dist)");
    println!("      --help              Show this help message");
    println!();
    println!("Examples:");
    println!("  server                          # Use defaults");
    println!("  server --port 8080              # Listen on port 8080");
    println!("  server --host 0.0.0.0 --port 3000  # Listen on all interfaces");
}
