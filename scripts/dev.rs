#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
notify = "6.0"
duct = "0.13"
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }
crossterm = "0.28"
which = "5.0"
dirs = "5.0"
fs_extra = "1.3"
regex = "1.10"
colored = "2.1"
tokio-stream = "0.1"
async-trait = "0.1"

[profile.dev]
opt-level = 0
---

//! ArchFlow Dev Orchestrator - Automated development workflow
//!
//! This script manages the complete development environment for ArchFlow:
//! - Compiles Rust WASM with hot reload
//! - Runs frontend dev server with hot reload
//! - Executes tests in watch mode
//! - Provides a unified CLI for all development tasks
//!
//! Usage (cargo-script - Rust nightly required):
//!   ./scripts/dev.rs install
//!   ./scripts/dev.rs build
//!   ./scripts/dev.rs start
//!
//! Prerequisites:
//!   - Rust 1.85+ with cargo-nightly for -Zscript
//!   - Node.js 18+
//!   - wasm-pack installed
//!   - Playwright for E2E tests

use anyhow::{Result, Context, bail};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::signal;
use tokio::time::sleep;

const PROJECT_ROOT: &str = "/home/rubentxu/Proyectos/rust/hodei-archFlow";
const WASM_BRIDGE_CRATE: &str = "crates/archflow-wasm-bridge";
const WEB_UI_CRATE: &str = "crates/archflow-web-ui";
const SDK_CRATE: &str = "crates/archflow-sdk";

/// Simple ANSI colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Red, Green, Yellow, Cyan,
}

impl Color {
    fn to_str(self) -> &'static str {
        match self {
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Cyan => "\x1b[36m",
        }
    }
}

struct Style(Color);

impl Style {
    fn color(self, s: &str) -> String {
        format!("{}{}\x1b[0m", self.0.to_str(), s)
    }
}

fn print_header(s: &str) {
    println!("{}", Style(Color::Cyan).color(&format!("\n🔧 {}", s)));
}

fn print_success(s: &str) {
    println!("{}", Style(Color::Green).color(&format!("✅ {}", s)));
}

fn print_error(s: &str) {
    println!("{}", Style(Color::Red).color(&format!("❌ {}", s)));
}

fn print_info(s: &str) {
    println!("{}", Style(Color::Yellow).color(&format!("ℹ️  {}", s)));
}

/// Get project root directory
fn project_root() -> PathBuf {
    PathBuf::from(PROJECT_ROOT)
}

/// Get WASM bridge crate directory
fn wasm_bridge_dir() -> PathBuf {
    project_root().join(WASM_BRIDGE_CRATE)
}

/// Get web UI crate directory (frontend)
fn web_ui_dir() -> PathBuf {
    project_root().join(WEB_UI_CRATE)
}

/// Get SDK crate directory
fn sdk_dir() -> PathBuf {
    project_root().join(SDK_CRATE)
}

/// Check if a command is available
async fn check_command(name: &str) -> Result<bool> {
    let output = Command::new("which")
        .arg(name)
        .output()
        .await?;
    Ok(output.status.success())
}

/// Install missing dependencies (excluding Playwright)
async fn install_dependencies(verbose: bool) -> Result<()> {
    print_header("Installing dependencies...");

    // Check wasm-pack
    if !check_command("wasm-pack").await? {
        print_info("Installing wasm-pack...");
        Command::new("cargo")
            .args(["install", "wasm-pack"])
            .status()
            .await
            .context("Failed to install wasm-pack")?;
    }

    // Check Node.js dependencies
    let node_modules = web_ui_dir().join("node_modules");
    if !node_modules.exists() {
        print_info("Installing Node.js dependencies...");
        let mut cmd = Command::new("npm");
        cmd.args(["install"]);
        cmd.current_dir(&web_ui_dir());
        if !verbose {
            cmd.stdout(std::process::Stdio::null());
        }
        cmd.status().await.context("Failed to install npm dependencies")?;
    }

    print_success("Dependencies installed (Playwright excluded)");
    Ok(())
}

/// Install Playwright browsers (separate command)
async fn install_playwright() -> Result<()> {
    print_header("Installing Playwright browsers...");

    if !check_command("playwright").await? {
        print_info("Installing Playwright browsers...");
        let mut cmd = Command::new("npx");
        cmd.args(["playwright", "install", "--with-deps", "chromium"]);
        cmd.current_dir(&web_ui_dir());
        cmd.status().await.context("Failed to install Playwright")?;
    }

    print_success("Playwright installed");
    Ok(())
}

/// Build WASM module
async fn build_wasm(debug: bool, verbose: bool) -> Result<()> {
    print_header("Building WASM module...");

    let target = if debug { "debug" } else { "release" };
    let flags = if debug { ["--debug"] } else { ["--release"] };

    let mut cmd = Command::new("wasm-pack");
    cmd.args(&["build", "--target", "web"]);
    cmd.args(&flags);
    cmd.current_dir(&sdk_dir());

    if !verbose {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    let status = cmd.status().await.context("Failed to build WASM")?;

    if !status.success() {
        bail!("WASM build failed");
    }

    print_success(&format!("WASM built ({})", target));
    Ok(())
}

/// Build Rust workspace (specific packages to avoid cache issues)
async fn build_rust(debug: bool, verbose: bool) -> Result<()> {
    print_header("Building Rust workspace...");

    let mut cmd = Command::new("cargo");
    // Build specific packages to avoid workspace cache issues
    cmd.args([
        "build",
        "-p", "archflow-sdk",
        "-p", "archflow-core",
        "-p", "archflow-geometry",
        "-p", "archflow-spatial",
        "-p", "archflow-primitives",
        "-p", "archflow-renderers",
        "-p", "archflow-records",
        "-p", "archflow-collab",
        "-p", "archflow-workspace",
    ]);

    if !debug {
        cmd.arg("--release");
    }

    if verbose {
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
    }

    let status = cmd.status().await.context("Failed to build Rust")?;

    if !status.success() {
        bail!("Rust build failed");
    }

    print_success("Rust workspace built");
    Ok(())
}

/// Run Rust tests
async fn run_tests(watch: bool, verbose: bool) -> Result<()> {
    print_header("Running tests...");

    let mut cmd = Command::new("cargo");
    cmd.args(["test", "--workspace"]);

    if !verbose {
        cmd.stdout(std::process::Stdio::null());
    }

    if watch {
        // In watch mode, we'd use cargo-watch
        print_info("For continuous testing, use: cargo watch -x test");
    }

    let status = cmd.status().await.context("Tests failed")?;

    if !status.success() {
        bail!("Tests failed");
    }

    print_success("All tests passed");
    Ok(())
}

/// Start development servers
async fn start_dev_server(
    with_wasm: bool,
    with_frontend: bool,
    _with_tests: bool,
    verbose: bool,
) -> Result<()> {
    print_header("Starting development servers...");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    // Handle Ctrl+C
    tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Build WASM first if needed
    if with_wasm {
        build_wasm(true, verbose).await?;
    }

    // Start frontend dev server
    if with_frontend {
        print_info("Starting frontend dev server on http://localhost:5173");

        let mut vite = Command::new("npm");
        vite.args(["run", "dev"]);
        vite.current_dir(&web_ui_dir());
        vite.stdout(std::process::Stdio::inherit());
        vite.stderr(std::process::Stdio::inherit());

        let mut vite_child = vite.spawn().context("Failed to start Vite")?;

        // Wait for Vite to be ready
        sleep(Duration::from_secs(3)).await;
        print_success("Frontend server started");

        // Keep running until shutdown
        while !shutdown.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(500)).await;

            // Check if process is still running
            if let Ok(Some(_)) = vite_child.try_wait() {
                break;
            }
        }

        // Cleanup
        print_info("Stopping frontend server...");
        vite_child.kill().await.ok();
    }

    print_success("Development server stopped");
    Ok(())
}

/// Watch mode for Rust development
async fn start_watch_mode(_verbose: bool) -> Result<()> {
    print_header("Starting watch mode...");

    // Check if cargo-watch is installed
    if !check_command("cargo-watch").await? {
        print_info("Installing cargo-watch...");
        Command::new("cargo")
            .args(["install", "cargo-watch"])
            .status()
            .await
            .context("Failed to install cargo-watch")?;
    }

    print_info("cargo-watch installed. Run:");
    println!();
    println!("  {}", Style(Color::Cyan).color("cargo watch -x build -x test"));
    println!();
    print_info("For WASM rebuild on change:");
    println!();
    println!("  {}", Style(Color::Cyan).color("wasm-pack build --target web --watch"));

    Ok(())
}

/// Clean build artifacts
async fn clean() -> Result<()> {
    print_header("Cleaning build artifacts...");

    // Clean Rust target
    Command::new("cargo")
        .args(["clean"])
        .status()
        .await?;

    // Remove WASM pkg
    let pkg_dir = sdk_dir().join("pkg");
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir)?;
    }

    // Remove node_modules
    let node_modules = web_ui_dir().join("node_modules");
    if node_modules.exists() {
        std::fs::remove_dir_all(&node_modules)?;
    }

    print_success("Cleaned all artifacts");
    Ok(())
}

/// Status check
async fn status_check() -> Result<()> {
    print_header("ArchFlow Development Status");

    // Check Rust version
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .await?;
    let rust_version = String::from_utf8_lossy(&rustc.stdout);
    println!("  Rust: {}", rust_version.trim());

    // Check wasm-pack
    let wasm_pack = Command::new("wasm-pack")
        .arg("--version")
        .output()
        .await?;
    println!("  wasm-pack: {}", String::from_utf8_lossy(&wasm_pack.stdout).trim());

    // Check Node.js
    let node = Command::new("node")
        .arg("--version")
        .output()
        .await?;
    println!("  Node.js: {}", String::from_utf8_lossy(&node.stdout).trim());

    // Check wasm build
    let pkg_dir = sdk_dir().join("pkg");
    if pkg_dir.exists() {
        print_success("WASM built");
    } else {
        print_info("WASM not built (run: dev.rs build)");
    }

    // Check node_modules
    let node_modules = web_ui_dir().join("node_modules");
    if node_modules.exists() {
        print_success("Node modules installed");
    } else {
        print_info("Node modules not installed (run: dev.rs install)");
    }

    // Check tests
    print_header("Running quick test...");
    let status = Command::new("cargo")
        .args(["test", "-p", "archflow-sdk", "--", "--test-threads=1"])
        .status()
        .await?;

    if status.success() {
        print_success("SDK tests passing");
    } else {
        print_error("Some tests failing");
    }

    Ok(())
}

/// Development workflow orchestrator
#[derive(Parser, Debug)]
#[command(name = "dev")]
#[command(author = "ArchFlow Team")]
#[command(version = "0.24.0")]
#[command(about = "ArchFlow Development Orchestrator", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Install all dependencies
    Install {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build everything (WASM + Rust + Frontend)
    Build {
        /// Release build
        #[arg(short, long)]
        release: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start development server with hot reload
    Start {
        /// Include WASM compilation
        #[arg(long)]
        wasm: bool,
        /// Include frontend server
        #[arg(long)]
        frontend: bool,
        /// Run tests in background
        #[arg(long)]
        test: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run all tests
    Test {
        /// Watch mode (requires cargo-watch)
        #[arg(short, long)]
        watch: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start watch mode for continuous development
    Watch {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run development server only (Vite)
    Frontend {
        /// Port to bind
        #[arg(short, long, default_value = "5173")]
        port: u16,
    },

    /// Run backend/SDK tests only
    Unit {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run E2E tests with Playwright
    E2e {
        /// Headed mode (show browser)
        #[arg(long)]
        headed: bool,
        /// Generate report
        #[arg(long)]
        report: bool,
    },

    /// Install Playwright browsers
    Playwright,

    /// Clean all build artifacts
    Clean,

    /// Show development status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Install { verbose } => {
            install_dependencies(verbose).await?;
        }

        Commands::Build { release, verbose } => {
            install_dependencies(false).await?;
            build_rust(!release, verbose).await?;
            build_wasm(!release, verbose).await?;
            print_success("Build complete!");
            print_info("Run 'dev.rs start' to start development server");
        }

        Commands::Start { wasm, frontend, test: _, verbose } => {
            // If neither flag is provided, run both.
            // If one is provided, run only that one.
            let (run_wasm, run_frontend) = if !wasm && !frontend {
                (true, true)
            } else {
                (wasm, frontend)
            };

            start_dev_server(run_wasm, run_frontend, false, verbose).await?;
        }

        Commands::Test { watch, verbose } => {
            if watch {
                start_watch_mode(verbose).await?;
            } else {
                run_tests(false, verbose).await?;
            }
        }

        Commands::Watch { verbose } => {
            start_watch_mode(verbose).await?;
        }

        Commands::Frontend { port } => {
            let port_info = format!("Starting Vite on port {}...", port);
            print_info(&port_info);
            let mut cmd = Command::new("npm");
            cmd.args(["run", "dev", "--", "--port", &port.to_string()]);
            cmd.current_dir(&web_ui_dir());
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            cmd.spawn()?.wait().await?;
        }

        Commands::Unit { verbose } => {
            run_tests(false, verbose).await?;
        }

        Commands::E2e { headed, report } => {
            print_header("Running E2E tests with Playwright...");

            let mut args = vec!["test"];
            if headed {
                args.push("--headed");
            }
            if report {
                args.push("--reporter=list");
            }

            let mut cmd = Command::new("npm");
            cmd.args(&args);
            cmd.current_dir(&web_ui_dir());
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());
            cmd.status().await?;
        }

        Commands::Playwright => {
            install_playwright().await?;
        }

        Commands::Clean => {
            clean().await?;
        }

        Commands::Status => {
            status_check().await?;
        }
    }

    Ok(())
}
