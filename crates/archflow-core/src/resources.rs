//! External Resources - Images, videos, and HTML overlays

use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ResourceError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Failed to load resource: {0}")]
    LoadError(String),
    #[error("Invalid resource format: {0}")]
    InvalidFormat(String),
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    #[error("Resource not loaded: {0}")]
    NotLoaded(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Image,
    Svg,
    Video,
    Html,
    Custom(String),
}

impl ResourceType {
    pub fn is_custom(&self) -> bool {
        matches!(self, ResourceType::Custom(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResourceState {
    Pending,
    Loading,
    Loaded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceId {
    pub id: EntityId,
    pub resource_type: ResourceType,
    pub source: String,
    pub name: String,
}

impl ResourceId {
    pub fn new(resource_type: ResourceType, source: String, name: String) -> Self {
        Self {
            id: EntityId::new(),
            resource_type,
            source,
            name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub id: ResourceId,
    pub state: ResourceState,
    pub size_bytes: u64,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
    pub ref_count: u32,
}

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub metadata: ResourceMetadata,
    #[cfg(feature = "wasm")]
    pub image_data: Option<web_sys::HtmlImageElement>,
    #[cfg(not(feature = "wasm"))]
    pub image_data: Option<Vec<u8>>,
}

impl ImageResource {
    pub fn new(id: ResourceId, width: u32, height: u32, mime_type: String) -> Self {
        Self {
            metadata: ResourceMetadata {
                id,
                state: ResourceState::Pending,
                size_bytes: 0,
                mime_type,
                width: Some(width),
                height: Some(height),
                duration_ms: None,
                error: None,
                ref_count: 0,
            },
            image_data: None,
        }
    }
    pub fn is_ready(&self) -> bool {
        self.metadata.state == ResourceState::Loaded && self.image_data.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VideoPlayback {
    Playing,
    Paused,
    Ended,
}

#[derive(Debug, Clone)]
pub struct VideoResource {
    pub metadata: ResourceMetadata,
    #[cfg(feature = "wasm")]
    pub video_data: Option<web_sys::HtmlVideoElement>,
    #[cfg(not(feature = "wasm"))]
    pub video_data: Option<PathBuf>,
    pub playback: VideoPlayback,
    pub volume: f32,
    pub muted: bool,
    pub looped: bool,
    pub position_ms: u64,
    pub playback_rate: f32,
}

impl VideoResource {
    pub fn new(
        id: ResourceId,
        width: u32,
        height: u32,
        duration_ms: u64,
        mime_type: String,
    ) -> Self {
        Self {
            metadata: ResourceMetadata {
                id,
                state: ResourceState::Pending,
                size_bytes: 0,
                mime_type,
                width: Some(width),
                height: Some(height),
                duration_ms: Some(duration_ms),
                error: None,
                ref_count: 0,
            },
            video_data: None,
            playback: VideoPlayback::Paused,
            volume: 1.0,
            muted: false,
            looped: false,
            position_ms: 0,
            playback_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssStyle {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HtmlBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for HtmlBounds {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlOverlay {
    pub metadata: ResourceMetadata,
    pub html: String,
    pub styles: Vec<CssStyle>,
    pub bounds: HtmlBounds,
    pub interactive: bool,
    pub pointer_events: bool,
    pub z_index: i32,
    pub visible: bool,
    pub opacity: f32,
    pub transform: String,
    pub on_click: Option<String>,
    pub on_mouse_enter: Option<String>,
    pub on_mouse_leave: Option<String>,
}

impl HtmlOverlay {
    pub fn new(id: ResourceId, html: String) -> Self {
        Self {
            metadata: ResourceMetadata {
                id,
                state: ResourceState::Loaded,
                size_bytes: html.len() as u64,
                mime_type: "text/html".to_string(),
                width: None,
                height: None,
                duration_ms: None,
                error: None,
                ref_count: 0,
            },
            html,
            styles: Vec::new(),
            bounds: HtmlBounds::default(),
            interactive: true,
            pointer_events: true,
            z_index: 0,
            visible: true,
            opacity: 1.0,
            transform: String::new(),
            on_click: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.bounds.x = x;
        self.bounds.y = y;
        self
    }
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.bounds.width = width;
        self.bounds.height = height;
        self
    }
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

pub trait ResourceLoader: Send + Sync {
    fn supported_types(&self) -> Vec<ResourceType>;
    fn load(&self, source: &str) -> Result<Vec<u8>, ResourceError>;
    fn mime_type(&self, extension: &str) -> String;
}

pub struct FileResourceLoader {
    base_path: PathBuf,
}

impl FileResourceLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl ResourceLoader for FileResourceLoader {
    fn supported_types(&self) -> Vec<ResourceType> {
        vec![ResourceType::Image, ResourceType::Svg, ResourceType::Video]
    }
    fn load(&self, source: &str) -> Result<Vec<u8>, ResourceError> {
        let path = self.base_path.join(source);
        std::fs::read(&path).map_err(|e| ResourceError::LoadError(e.to_string()))
    }
    fn mime_type(&self, extension: &str) -> String {
        match extension {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "ogg" => "video/ogg",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

pub struct ResourceManager {
    images: Arc<RwLock<HashMap<EntityId, ImageResource>>>,
    videos: Arc<RwLock<HashMap<EntityId, VideoResource>>>,
    html_overlays: Arc<RwLock<HashMap<EntityId, HtmlOverlay>>>,
    loader: Arc<RwLock<Option<Box<dyn ResourceLoader>>>>,
    ref_counts: Arc<RwLock<HashMap<EntityId, u32>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            images: Arc::new(RwLock::new(HashMap::new())),
            videos: Arc::new(RwLock::new(HashMap::new())),
            html_overlays: Arc::new(RwLock::new(HashMap::new())),
            loader: Arc::new(RwLock::new(None)),
            ref_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn set_loader(&mut self, loader: Box<dyn ResourceLoader>) {
        *self.loader.write().unwrap() = Some(loader);
    }
    pub fn create_html_overlay(&self, html: String, name: &str) -> Result<EntityId, ResourceError> {
        let resource_id = ResourceId::new(
            ResourceType::Html,
            format!("inline:{}", name),
            name.to_string(),
        );
        let id = resource_id.id;
        self.html_overlays
            .write()
            .unwrap()
            .insert(id, HtmlOverlay::new(resource_id, html));
        Ok(id)
    }
    pub fn get_html_overlay(&self, id: EntityId) -> Option<HtmlOverlay> {
        self.html_overlays.read().unwrap().get(&id).cloned()
    }
    pub fn get_all_html_overlays(&self) -> Vec<HtmlOverlay> {
        self.html_overlays
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    pub fn remove_html_overlay(&self, id: EntityId) -> bool {
        self.html_overlays.write().unwrap().remove(&id).is_some()
    }
    pub fn clear(&self) {
        self.html_overlays.write().unwrap().clear();
        self.ref_counts.write().unwrap().clear();
    }
    pub fn len(&self) -> usize {
        self.images.read().unwrap().len()
            + self.videos.read().unwrap().len()
            + self.html_overlays.read().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resource_id_creation() {
        let id = ResourceId::new(
            ResourceType::Image,
            "test.png".to_string(),
            "Test Image".to_string(),
        );
        assert_eq!(id.resource_type, ResourceType::Image);
    }

    #[test]
    fn test_image_resource() {
        let id = ResourceId::new(
            ResourceType::Image,
            "test.png".to_string(),
            "Test".to_string(),
        );
        let resource = ImageResource::new(id.clone(), 100, 200, "image/png".to_string());
        assert_eq!(resource.metadata.width, Some(100));
        assert_eq!(resource.metadata.height, Some(200));
        assert!(!resource.is_ready());
    }

    #[test]
    fn test_html_overlay() {
        let id = ResourceId::new(
            ResourceType::Html,
            "inline:test".to_string(),
            "Test".to_string(),
        );
        let overlay = HtmlOverlay::new(id.clone(), "<div>Hello</div>".to_string())
            .with_position(10.0, 20.0)
            .with_size(100.0, 50.0)
            .with_z_index(100);
        assert_eq!(overlay.bounds.x, 10.0);
        assert_eq!(overlay.z_index, 100);
    }

    #[test]
    fn test_resource_manager() {
        let manager = ResourceManager::new();
        assert!(manager.is_empty());
        let id = manager
            .create_html_overlay("<div>Test</div>".to_string(), "test")
            .unwrap();
        assert!(!manager.is_empty());
        let overlay = manager.get_html_overlay(id);
        assert!(overlay.is_some());
        let removed = manager.remove_html_overlay(id);
        assert!(removed);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_file_resource_loader() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("archflow_test.png");
        let _ = fs::write(&test_file, b"fake png data");

        let loader = FileResourceLoader::new(temp_dir.clone());
        assert!(loader.supported_types().contains(&ResourceType::Image));
        assert_eq!(loader.mime_type("png"), "image/png");
        assert_eq!(loader.mime_type("mp4"), "video/mp4");

        // Test loading
        let result = loader.load("archflow_test.png");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"fake png data");

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_video_resource() {
        let id = ResourceId::new(
            ResourceType::Video,
            "test.mp4".to_string(),
            "Test Video".to_string(),
        );
        let resource = VideoResource::new(id.clone(), 1920, 1080, 60000, "video/mp4".to_string());
        assert_eq!(resource.metadata.width, Some(1920));
        assert_eq!(resource.playback, VideoPlayback::Paused);
    }

    #[test]
    fn test_resource_type_custom() {
        let custom = ResourceType::Custom("custom_type".to_string());
        assert!(custom.is_custom());
        let image = ResourceType::Image;
        assert!(!image.is_custom());
    }
}
