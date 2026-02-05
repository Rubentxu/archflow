# ArchFlow Development Workflow
# Justfile for automated development tasks
#
# Install: https://github.com/casey/just
# Usage: just <recipe>

# Default recipe
default: help

# =============================================================================
# CONFIGURATION
# =============================================================================
PROJECT_ROOT := "/home/rubentxu/Proyectos/rust/hodei-archFlow"
WEB_DIR := PROJECT_ROOT + "/crates/archflow-web-ui"
SDK_DIR := PROJECT_ROOT + "/crates/archflow-sdk"
PKG_DIR := SDK_DIR + "/pkg"
WEB_PKG_DIR := PROJECT_ROOT + "/crates/archflow-web/pkg"

# =============================================================================
# HELP & INFO
# =============================================================================

# Show help
[doc("Show this help message")]
help:
    @echo ""
    @echo "ArchFlow Development Workflow v0.24.0"
    @echo ""
    @echo "Usage: just <recipe>"
    @echo ""
    @echo "Recipes:"
    @just --list --unsorted | sed 's/^/  /'
    @echo ""
    @echo "Examples:"
    @echo "  just install          # Install dependencies"
    @echo "  just dev              # Start development server"
    @echo "  just build            # Build everything"
    @echo "  just test             # Run all tests"
    @echo ""

# Show project status
[doc("Show development status")]
status:
    @echo "ArchFlow Status"
    @echo ""
    @rustc --version | xargs echo "  Rust:"
    @wasm-pack --version 2>/dev/null | xargs echo "  wasm-pack:" || echo "  wasm-pack: not installed"
    @node --version 2>/dev/null | xargs echo "  Node.js:" || echo "  Node.js: not installed"
    @echo ""
    @if [ -d "{{PKG_DIR}}" ]; then \
        echo "✅ WASM built"; \
    else \
        echo "⚠️  WASM not built"; \
    fi
    @if [ -d "{{WEB_DIR}}/node_modules" ]; then \
        echo "✅ Node modules installed"; \
    else \
        echo "⚠️  Node modules missing"; \
    fi
    @echo ""
    @echo "Quick Links:"
    @echo "  Dev Server:   http://localhost:5173"
    @echo "  Files:        {{PROJECT_ROOT}}"

# =============================================================================
# INSTALLATION
# =============================================================================

# Install all dependencies
[doc("Install all development dependencies")]
install:
    @echo "Installing dependencies..."
    @echo ""
    # Check and install wasm-pack
    @if ! command -v wasm-pack > /dev/null 2>&1; then \
        echo "Installing wasm-pack..."; \
        cargo install wasm-pack; \
    else \
        echo "wasm-pack already installed"; \
    fi
    # Install Node dependencies
    @if [ ! -d "{{WEB_DIR}}/node_modules" ]; then \
        echo "Installing Node.js dependencies..."; \
        cd {{WEB_DIR}} && npm install; \
    else \
        echo "Node modules already installed"; \
    fi
    # Install Playwright
    @if ! command -v playwright > /dev/null 2>&1; then \
        echo "Installing Playwright browsers..."; \
        cd {{WEB_DIR}} && npx playwright install --with-deps chromium; \
    else \
        echo "Playwright already installed"; \
    fi
    @echo ""
    @echo "All dependencies installed!"

# Install wasm-pack only
install-wasm-pack:
    @echo "Installing wasm-pack..."
    @cargo install wasm-pack
    @echo "wasm-pack installed!"

# Install Node dependencies only
install-node:
    @echo "Installing Node.js dependencies..."
    @cd {{WEB_DIR}} && npm install
    @echo "Node modules installed!"

# Install Playwright browsers
install-playwright:
    @echo "Installing Playwright browsers..."
    @cd {{WEB_DIR}} && npx playwright install --with-deps chromium
    @echo "Playwright installed!"

# =============================================================================
# BUILD
# =============================================================================

# Build everything (WASM + Rust)
[doc("Build all components (WASM + Rust)")]
build:
    @echo "Building ArchFlow..."
    @echo ""
    @echo "Building Rust workspace..."
    @cargo build -p archflow-sdk -p archflow-core -p archflow-geometry -p archflow-spatial -p archflow-primitives -p archflow-renderers -p archflow-records -p archflow-collab -p archflow-workspace
    @echo ""
    @echo "Building and syncing WASM..."
    @just build-wasm
    @echo ""
    @echo "Build complete!"
    @echo ""
    @echo "Run 'just dev' to start development server"

# Build in release mode
[doc("Build everything in release mode")]
build-release:
    @echo "Building in release mode..."
    @cargo build -p archflow-sdk -p archflow-core -p archflow-geometry -p archflow-spatial -p archflow-primitives -p archflow-renderers -p archflow-records -p archflow-collab -p archflow-workspace --release
    @cd {{SDK_DIR}} && wasm-pack build --target web --release
    @echo "Release build complete!"

# Build WASM only
[doc("Build WASM module only")]
build-wasm: sync-wasm-types
    @echo "Building WASM..."
    @cd crates/archflow-web && wasm-pack build --target web --debug
    @echo "WASM built!"
    @cp crates/archflow-web/pkg/archflow_web_bg.wasm crates/archflow-web-ui/src/wasm/


# Sync WASM types to frontend
[doc("Sync WASM types from Rust to TypeScript frontend")]
sync-wasm-types:
    @echo "Sincronizando tipos WASM..."
    @./scripts/sync-wasm-types.sh
    @echo "Tipos sincronizados!"

# Build Rust only
[doc("Build Rust workspace only")]
build-rust:
    @echo "Building Rust..."
    @cargo build -p archflow-sdk -p archflow-core -p archflow-geometry -p archflow-spatial -p archflow-primitives -p archflow-renderers -p archflow-records -p archflow-collab -p archflow-workspace
    @echo "Rust built!"

# =============================================================================
# DEVELOPMENT SERVER
# =============================================================================

# Start development server with hot reload (frontend + WASM rebuild)
[doc("Start development server with hot reload")]
dev: build-wasm
    @echo Starting development server...
    @echo ""
    @echo Frontend:  http://localhost:5173
    @echo Hot reload enabled for Rust and JS
    @echo ""
    @echo Press Ctrl+C to stop
    @echo ""
    @cd {{WEB_DIR}} && npm run dev

# Start Vite dev server only (frontend)
[doc("Start Vite frontend server only")]
frontend:
    @echo Starting frontend server...
    @cd {{WEB_DIR}} && npm run dev

# Start WASM watch mode (auto-rebuild on change)
[doc("Watch WASM changes and rebuild automatically")]
wasm-watch:
    @echo Starting WASM watch mode...
    @cd {{SDK_DIR}} && wasm-pack build --target web --debug --watch

# Start Rust watch mode with cargo-watch
[doc("Watch Rust changes and rebuild automatically")]
rust-watch:
    @echo Starting Rust watch mode...
    @if ! command -v cargo-watch > /dev/null 2>&1; then \
        echo Installing cargo-watch...; \
        cargo install cargo-watch; \
    fi
    @echo Running cargo watch...
    @cargo watch -x build -x test

# Start all watchers (Rust + WASM + Frontend) in parallel
[doc("Start all watchers in parallel (requires terminal multiplexer)")]
watch-all:
    @echo Starting full watch mode...
    @echo "Run these commands in separate terminals:"
    @echo ""
    @echo 1. Rust watch: "  just rust-watch"
    @echo 2. WASM watch: " just wasm-watch"
    @echo 3. Frontend: "  just frontend"
    @echo ""

# =============================================================================
# TESTING
# =============================================================================

# Run all tests (unit + integration + shader E2E)
[doc("Run all tests (unit + integration + shader E2E)")]
test:
    @echo "Running all tests..."
    @cargo test -p archflow-sdk -p archflow-core -p archflow-geometry -p archflow-spatial -p archflow-primitives
    @echo ""
    @echo "Running shader E2E tests..."
    @just test-shader-e2e
    @echo ""
    @echo "All tests passed!"

# Run SDK tests only
[doc("Run SDK unit tests only")]
test-sdk:
    @echo Running SDK tests...
    @cargo test -p archflow-sdk
    @echo SDK tests passed!

# Run workspace tests
[doc("Run workspace tests only")]
test-workspace:
    @echo Running workspace tests...
    @cargo test -p archflow-core \
        -p archflow-geometry \
        -p archflow-spatial \
        -p archflow-primitives

# Run tests in watch mode
[doc("Run tests in watch mode (requires cargo-watch)")]
test-watch:
    @echo Starting test watch mode...
    @cargo watch -x "test --workspace"

# Run shader rendering E2E tests (WASM + WebGL2)
[doc("Run shader rendering E2E tests in headless browser")]
test-shader-e2e:
    @echo "Running shader rendering E2E tests..."
    @echo ""
    @cd crates/archflow-render && wasm-pack test --headless --firefox
    @echo ""
    @echo "Shader E2E tests passed!"

# Run shader E2E tests in Chrome
[doc("Run shader E2E tests in Chrome")]
test-shader-e2e-chrome:
    @echo "Running shader rendering E2E tests (Chrome)..."
    @cd crates/archflow-render && wasm-pack test --headless --chrome

# Run shader E2E tests with browser visible
[doc("Run shader E2E tests with visible browser (debugging)")]
test-shader-e2e-headed:
    @echo "Running shader rendering E2E tests (headed mode)..."
    @cd crates/archflow-render && wasm-pack test --firefox

# Run E2E tests with Playwright
[doc("Run E2E tests with Playwright")]
test-e2e:
    @echo Running E2E tests with Playwright...
    @cd {{WEB_DIR}} && npm test

# Run E2E tests with headed mode (visible browser)
[doc("Run E2E tests with headed mode (show browser)")]
test-e2e-headed:
    @echo Running E2E tests (headed mode)...
    @cd {{WEB_DIR}} && npm run test:headed

# Run E2E tests and show report
[doc("Run E2E tests and show HTML report")]
test-e2e-report:
    @echo Running E2E tests with report...
    @cd {{WEB_DIR}} && npm run test:report

# Generate test coverage report
[doc("Generate test coverage report")]
coverage:
    @echo Generating coverage report...
    @cargo install cargo-tarpaulin 2>/dev/null || true
    @cargo tarpaulin --workspace --out Html
    @echo Coverage report generated!
    @echo "Open: target/tarpaulin-report/index.html"

# =============================================================================
# LINTING & FORMATTING
# =============================================================================

# Format Rust code
[doc("Format Rust code with rustfmt")]
fmt:
    @echo Formatting Rust code...
    @cargo fmt --all
    @echo Code formatted!

# Check formatting
[doc("Check Rust code formatting")]
fmt-check:
    @echo Checking formatting...
    @cargo fmt --all --check
    @echo Formatting OK!

# Run clippy lints
[doc("Run Clippy lints")]
clippy:
    @echo Running Clippy...
    @cargo clippy --workspace -- -D warnings
    @echo Clippy passed!

# Check for dead code
[doc("Check for dead code")]
deadcode:
    @echo Checking for dead code...
    @cargo check --all-targets
    @echo No dead code issues!

# =============================================================================
# CLEANUP
# =============================================================================

# Clean all build artifacts
[doc("Clean all build artifacts")]
clean:
    @echo Cleaning build artifacts...
    @cargo clean
    @rm -rf {{PKG_DIR}}
    @rm -rf {{WEB_DIR}}/node_modules
    @echo Cleaned!

# Clean WASM only
clean-wasm:
    @echo Cleaning WASM artifacts...
    @rm -rf {{PKG_DIR}}
    @echo WASM cleaned!

# Clean Node modules only
clean-node:
    @echo Cleaning Node modules...
    @rm -rf {{WEB_DIR}}/node_modules
    @echo Node modules cleaned!

# =============================================================================
# TYPE GENERATION
# =============================================================================

# Generate TypeScript types from Rust
[doc("Generate TypeScript bindings from Rust")]
types:
    @echo Generating TypeScript types...
    @cargo build -p archflow-sdk --features wasm
    @echo ""
    @echo TypeScript types generated!
    @ls -1 packages/archflow-sdk-types/src/generated/*.ts | wc -l | xargs echo "  Files generated:"

# Copy types to SDK package
[doc("Copy generated types to SDK package")]
types-copy:
    @echo Copying types to SDK package...
    @cp packages/archflow-sdk-types/src/generated/*.ts packages/sdk/src/generated/
    @echo Types copied!

# =============================================================================
# VERIFICATION
# =============================================================================

# Verify workspace builds correctly
[doc("Verify workspace compiles without errors")]
verify:
    @echo Verifying workspace...
    @cargo check --workspace
    @echo Workspace compiles!

# Verify WASM builds correctly
[doc("Verify WASM compiles without errors")]
verify-wasm:
    @echo Verifying WASM...
    @cd {{SDK_DIR}} && wasm-pack build --target web --dry-run
    @echo WASM compiles!

# Quick verification before commit
[doc("Quick verification before commit (check + fmt + test)")]
precommit: fmt-check verify test-sdk
    @echo ""
    @echo Ready to commit!

# =============================================================================
# DOCUMENTATION
# =============================================================================

# Generate documentation
[doc("Generate Rust documentation")]
doc:
    @echo Generating Rust documentation...
    @cargo doc --workspace --no-deps
    @echo ""
    @echo Documentation generated!
    @echo "Open: target/doc/index.html"

# Open documentation in browser
[doc("Open Rust documentation in browser")]
doc-open:
    @echo Opening documentation...
    @if command -v xdg-open > /dev/null 2>&1; then \
        xdg-open {{PROJECT_ROOT}}/target/doc/archflow_sdk/index.html; \
    elif command -v open > /dev/null 2>&1; then \
        open {{PROJECT_ROOT}}/target/doc/archflow_sdk/index.html; \
    fi

# =============================================================================
# DOCKER & DEPLOYMENT
# =============================================================================

# Build Docker image
[doc("Build Docker image for production")]
docker-build:
    @echo Building Docker image...
    @docker build -t archflow:latest .
    @echo Docker image built!

# Run Docker container
[doc("Run ArchFlow in Docker")]
docker-run:
    @echo Running ArchFlow in Docker...
    @docker run -p 8080:8080 archflow:latest

# =============================================================================
# EXPERIMENTAL / ADVANCED
# =============================================================================

# Run the Rust dev script directly (cargo-script)
[doc("Run the Rust development orchestrator script")]
dev-script:
    @./scripts/dev.rs --help

# Run dev script with specific command
dev-install:
    @./scripts/dev.rs install

dev-build:
    @./scripts/dev.rs build

dev-start:
    @./scripts/dev.rs start

dev-test:
    @./scripts/dev.rs test

dev-status:
    @./scripts/dev.rs status

# Benchmark compilation time
[doc("Benchmark compilation time")]
bench-build:
    @echo Benchmarking Rust compilation...
    @time cargo build --workspace --release 2>&1 | tail -5

# Show dependency tree
[doc("Show dependency tree for SDK")]
deps-tree:
    @echo SDK Dependency Tree...
    @cargo tree -p archflow-sdk -i

# =============================================================================
# UTILITIES
# =============================================================================

# Open project in VS Code
code:
    @echo "Opening in VS Code..."
    @if command -v code > /dev/null 2>&1; then \
        code {{PROJECT_ROOT}}; \
    else \
        echo "VS Code not found"; \
    fi

# Open project directory
open:
    @echo "Project location:"
    @echo {{PROJECT_ROOT}}
    @if command -v xdg-open > /dev/null 2>&1; then \
        xdg-open {{PROJECT_ROOT}}; \
    fi

# Print justfile recipe count
count:
    @just --list | grep -c "^[^ ]" | xargs echo "Recipes available:"

# Test color conversion in WASM
test-color-wasm:
    cd crates/archflow-web && cargo test color_conversion --lib -- --nocapture
