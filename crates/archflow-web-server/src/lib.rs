// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web Server - Axum-based HTTP/WebSocket server
//
// This crate provides the web server for ArchFlow Engine:
// - Serves static React UI files from archflow-web-ui/dist/
// - REST API for diagram CRUD operations
// - WebSocket for real-time collaboration
// - CORS support for development
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod collaboration;
pub mod error;
pub mod websocket;

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing::Level;

pub use error::{Error, Result};

/// Application state shared across all handlers
#[derive(Clone, Debug)]
pub struct AppState {
    /// Path to the React UI build directory (archflow-web-ui/dist/)
    pub ui_dist_path: PathBuf,
}

impl AppState {
    /// Create a new AppState with the given UI dist path
    pub fn new(ui_dist_path: PathBuf) -> Self {
        Self { ui_dist_path }
    }

    /// Create AppState with default path (relative to crate root)
    pub fn with_default_path() -> Result<Self> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../archflow-web-ui/dist");

        if !path
            .try_exists()
            .map_err(|_| Error::DistPathNotFound(path.clone()))?
        {
            return Err(Error::DistPathNotFound(path));
        }

        Ok(Self { ui_dist_path: path })
    }
}

/// Create the main router with all routes
pub fn create_router(state: AppState) -> Router {
    // CORS layer for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // API routes
        .route("/api/diagrams", get(list_diagrams).post(create_diagram))
        .route(
            "/api/diagrams/:id",
            get(get_diagram).put(update_diagram).delete(delete_diagram),
        )
        // WebSocket route
        .route("/ws", get(websocket::websocket_handler))
        // Fallback: serve static files from React build
        .fallback(serve_static_assets)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serve static assets from React build
async fn serve_static_assets(State(state): State<AppState>, uri: Uri) -> Result<Response> {
    let path = uri.path();

    // Serve index.html for root and non-file routes (SPA routing)
    if path == "/" || !path.contains('.') {
        let index_path = state.ui_dist_path.join("index.html");
        return match tokio::fs::read(index_path).await {
            Ok(contents) => {
                Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], contents).into_response())
            }
            Err(_) => Err(Error::Internal("index.html not found".to_string())),
        };
    }

    // Serve other static files using ServeFile
    let file_path = state.ui_dist_path.join(&path[1..]); // Remove leading /
    if file_path.try_exists().unwrap_or(false) {
        return match tokio::fs::read(&file_path).await {
            Ok(contents) => {
                let mime = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();
                let mut response = contents.into_response();
                response
                    .headers_mut()
                    .insert(header::CONTENT_TYPE, mime.parse().unwrap());
                Ok(response)
            }
            Err(_) => Err(Error::Internal("Failed to read file".to_string())),
        };
    }

    Err(Error::Internal("File not found".to_string()))
}

// ═══════════════════════════════════════════════════════════════════════════════
// API Handlers
// ═══════════════════════════════════════════════════════════════════════════════

/// List all diagrams
async fn list_diagrams(State(_state): State<AppState>) -> Result<Json<Vec<DiagramData>>> {
    // TODO: Implement actual diagram storage
    Ok(Json(vec![]))
}

/// Get a specific diagram by ID
async fn get_diagram(
    State(_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<DiagramData>> {
    // TODO: Implement actual diagram retrieval
    Err(Error::DiagramNotFound(id))
}

/// Create a new diagram
async fn create_diagram(
    State(_state): State<AppState>,
    Json(diagram): Json<DiagramData>,
) -> Result<Json<DiagramData>> {
    // TODO: Implement actual diagram creation
    Ok(Json(diagram))
}

/// Update an existing diagram
async fn update_diagram(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<String>,
    Json(diagram): Json<DiagramData>,
) -> Result<Json<DiagramData>> {
    // TODO: Implement actual diagram update
    Ok(Json(diagram))
}

/// Delete a diagram
async fn delete_diagram(
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<String>,
) -> Result<StatusCode> {
    // TODO: Implement actual diagram deletion
    Ok(StatusCode::NO_CONTENT)
}

/// Diagram data structure
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DiagramData {
    pub id: String,
    pub name: String,
    pub entities: Vec<EntityData>,
    pub connections: Vec<ConnectionData>,
}

/// Entity data structure
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityData {
    pub id: u32,
    pub position: PositionData,
    pub size: SizeData,
    #[serde(default)]
    pub logic_bricks: Option<LogicBricksData>,
}

/// Position data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

/// Size data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SizeData {
    pub width: f32,
    pub height: f32,
}

/// Logic Bricks data for an entity
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogicBricksData {
    pub rules: Vec<LogicRuleData>,
}

/// Logic rule data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogicRuleData {
    pub id: String,
    pub sensor: SensorData,
    pub actuators: Vec<ActuatorData>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Sensor data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SensorData {
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub config: serde_json::Value,
}

/// Actuator data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActuatorData {
    #[serde(rename = "type")]
    pub actuator_type: String,
    pub params: serde_json::Value,
}

/// Connection data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionData {
    pub id: String,
    pub from_entity: u32,
    pub to_entity: u32,
    #[serde(default)]
    pub label: Option<String>,
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Path to React UI dist directory
    pub ui_dist_path: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ui_dist_path: PathBuf::from("crates/archflow-web-ui/dist"),
        }
    }
}

/// Run the web server
pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    let state = AppState::new(config.ui_dist_path.clone());
    let app = create_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("ArchFlow Web Server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Run the server with default configuration
pub async fn run_default() -> anyhow::Result<()> {
    run_server(ServerConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_router() {
        let state = AppState {
            ui_dist_path: PathBuf::from("/tmp/test-dist"),
        };
        let router = create_router(state);
        // Router created successfully
        drop(router);
    }

    #[tokio::test]
    async fn test_serve_index_html_requires_dist() {
        let state = AppState {
            ui_dist_path: PathBuf::from("/nonexistent/path"),
        };
        let result = serve_static_assets(State(state), Uri::from_static("/")).await;

        // Should return error
        assert!(result.is_err());
    }
}
