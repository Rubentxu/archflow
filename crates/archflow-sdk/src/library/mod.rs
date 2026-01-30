//! Component Library System
//!
//! Provides a system for managing and instantiating predefined components
//! similar to draw.io or excalidraw libraries.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod manager;

pub use manager::LibraryManager;

/// Represents a library of components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentLibrary {
    /// Unique identifier for the library
    pub id: String,
    /// Display name of the library
    pub name: String,
    /// Description of the library contents
    pub description: String,
    /// Version of the library
    pub version: String,
    /// Author or source of the library
    pub author: String,
    /// Categories within the library
    pub categories: Vec<LibraryCategory>,
    /// Metadata about the library
    pub metadata: LibraryMetadata,
}

impl ComponentLibrary {
    /// Creates a new library with the given parameters
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            version: "1.0.0".to_string(),
            author: "ArchFlow".to_string(),
            categories: Vec::new(),
            metadata: LibraryMetadata::default(),
        }
    }

    /// Adds a category to the library
    pub fn with_category(mut self, category: LibraryCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Sets the library description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the library version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the library author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Searches for items matching the query string
    pub fn search_items(&self, query: &str) -> Vec<&LibraryItem> {
        let query_lower = query.to_lowercase();
        self.categories
            .iter()
            .flat_map(|cat| &cat.items)
            .filter(|item| {
                item.name.to_lowercase().contains(&query_lower)
                    || item.description.to_lowercase().contains(&query_lower)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

/// Represents a category within a library
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryCategory {
    /// Unique identifier for the category
    pub id: String,
    /// Display name of the category
    pub name: String,
    /// Icon representing the category (emoji or icon code)
    pub icon: String,
    /// Items within this category
    pub items: Vec<LibraryItem>,
    /// Whether the category is collapsed in the UI
    pub collapsed: bool,
}

impl LibraryCategory {
    /// Creates a new category
    pub fn new(id: impl Into<String>, name: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            items: Vec::new(),
            collapsed: false,
        }
    }

    /// Adds an item to the category
    pub fn with_item(mut self, item: LibraryItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the collapsed state
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

/// Represents an item within a library category
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryItem {
    /// Unique identifier for the item
    pub id: String,
    /// Display name of the item
    pub name: String,
    /// Description of the item
    pub description: String,
    /// Preview representation of the item
    pub preview: ItemPreview,
    /// Component data for instantiation
    pub data: ComponentData,
    /// Tags for searching and categorization
    pub tags: Vec<String>,
}

impl LibraryItem {
    /// Creates a new library item
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        preview: ItemPreview,
        data: ComponentData,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            preview,
            data,
            tags: Vec::new(),
        }
    }

    /// Sets the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Preview representation types for library items
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "value")]
#[ts(export)]
pub enum ItemPreview {
    /// Unicode emoji or icon character
    Icon(String),
    /// SVG string content
    Svg(String),
    /// Path to SVG file
    SvgPath(String),
    /// Color representation
    Color(String),
    /// Custom shape definition
    CustomShape(ShapeDefinition),
}

/// Shape definition for custom previews
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ShapeDefinition {
    pub shape_type: String,
    pub width: f32,
    pub height: f32,
    pub stroke_color: String,
    pub fill_color: String,
}

/// Component data for instantiation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentData {
    /// Type of shape to create
    pub shape_type: LibraryShapeType,
    /// Geometry properties
    pub geometry: ComponentGeometry,
    /// Visual styling
    pub style: ComponentStyle,
    /// Child components (for complex shapes)
    pub children: Vec<ComponentData>,
}

impl ComponentData {
    /// Creates new component data with the given shape type
    pub fn new(shape_type: LibraryShapeType) -> Self {
        Self {
            shape_type,
            geometry: ComponentGeometry::default(),
            style: ComponentStyle::default(),
            children: Vec::new(),
        }
    }

    /// Sets the geometry
    pub fn with_geometry(mut self, geometry: ComponentGeometry) -> Self {
        self.geometry = geometry;
        self
    }

    /// Sets the style
    pub fn with_style(mut self, style: ComponentStyle) -> Self {
        self.style = style;
        self
    }

    /// Adds child components
    pub fn with_children(mut self, children: Vec<ComponentData>) -> Self {
        self.children = children;
        self
    }
}

/// Shape types available in the library
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum LibraryShapeType {
    /// Rectangle shape
    Rectangle,
    /// Rounded rectangle with radius
    RoundedRectangle { radius: f32 },
    /// Circle shape
    Circle,
    /// Ellipse shape
    Ellipse,
    /// Diamond/rhombus shape
    Diamond,
    /// Triangle shape
    Triangle,
    /// Hexagon shape
    Hexagon,
    /// Cylinder shape (for databases)
    Cylinder,
    /// Cloud shape
    Cloud,
    /// Document shape
    Document,
    /// Line shape
    Line,
    /// Custom path shape
    CustomPath { path: String },
}

/// Geometry properties for component instantiation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentGeometry {
    /// Default width
    pub width: f32,
    /// Default height
    pub height: f32,
    /// Optional default X position
    pub default_x: Option<f32>,
    /// Optional default Y position
    pub default_y: Option<f32>,
}

impl Default for ComponentGeometry {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 80.0,
            default_x: None,
            default_y: None,
        }
    }
}

impl ComponentGeometry {
    /// Creates geometry with specific dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            default_x: None,
            default_y: None,
        }
    }
}

/// Visual styling for components
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ComponentStyle {
    /// Fill color (hex or rgba)
    pub fill_color: Option<String>,
    /// Stroke/border color
    pub stroke_color: Option<String>,
    /// Stroke width in pixels
    pub stroke_width: Option<f32>,
    /// Opacity (0.0 to 1.0)
    pub opacity: Option<f32>,
    /// Font family for text
    pub font_family: Option<String>,
    /// Font size
    pub font_size: Option<f32>,
}

impl ComponentStyle {
    /// Sets the fill color
    pub fn with_fill_color(mut self, color: impl Into<String>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Sets the stroke color
    pub fn with_stroke_color(mut self, color: impl Into<String>) -> Self {
        self.stroke_color = Some(color.into());
        self
    }

    /// Sets the stroke width
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    /// Sets the opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }
}

/// Metadata about a library
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LibraryMetadata {
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Whether this is a built-in library
    pub is_builtin: bool,
    /// Whether the library is editable
    pub is_editable: bool,
    /// Source of the library
    pub source: LibrarySource,
}

impl Default for LibraryMetadata {
    fn default() -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            created_at: now.clone(),
            updated_at: now,
            is_builtin: true,
            is_editable: false,
            source: LibrarySource::BuiltIn,
        }
    }
}

/// Source/origin of a library
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum LibrarySource {
    /// Built-in library included with ArchFlow
    BuiltIn,
    /// Created by the user
    UserCreated,
    /// Imported from a file
    Imported { path: String },
    /// Downloaded from community/marketplace
    Community { url: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_creation() {
        let library = ComponentLibrary::new("general", "General")
            .with_description("Basic shapes and forms")
            .with_version("1.0.0")
            .with_author("ArchFlow Team");

        assert_eq!(library.id, "general");
        assert_eq!(library.name, "General");
        assert_eq!(library.description, "Basic shapes and forms");
        assert_eq!(library.version, "1.0.0");
        assert_eq!(library.author, "ArchFlow Team");
        assert!(library.categories.is_empty());
    }

    #[test]
    fn test_category_creation() {
        let category = LibraryCategory::new("basic", "Basic Shapes", "⬜").collapsed(false);

        assert_eq!(category.id, "basic");
        assert_eq!(category.name, "Basic Shapes");
        assert_eq!(category.icon, "⬜");
        assert!(!category.collapsed);
        assert!(category.items.is_empty());
    }

    #[test]
    fn test_item_creation() {
        let data = ComponentData::new(LibraryShapeType::Rectangle)
            .with_geometry(ComponentGeometry::new(120.0, 80.0))
            .with_style(
                ComponentStyle::default()
                    .with_fill_color("#3366cc")
                    .with_stroke_color("#ffffff")
                    .with_stroke_width(1.0),
            );

        let item = LibraryItem::new(
            "rect",
            "Rectangle",
            ItemPreview::Icon("⬜".to_string()),
            data,
        )
        .with_description("A simple rectangle")
        .with_tags(vec!["basic".to_string(), "shape".to_string()]);

        assert_eq!(item.id, "rect");
        assert_eq!(item.name, "Rectangle");
        assert_eq!(item.description, "A simple rectangle");
        assert_eq!(item.tags.len(), 2);
    }

    #[test]
    fn test_component_geometry_default() {
        let geom = ComponentGeometry::default();
        assert_eq!(geom.width, 100.0);
        assert_eq!(geom.height, 80.0);
        assert!(geom.default_x.is_none());
        assert!(geom.default_y.is_none());
    }

    #[test]
    fn test_library_search() {
        let category = LibraryCategory::new("basic", "Basic", "⬜")
            .with_item(
                LibraryItem::new(
                    "rect",
                    "Rectangle",
                    ItemPreview::Icon("⬜".to_string()),
                    ComponentData::new(LibraryShapeType::Rectangle),
                )
                .with_description("A rectangle shape")
                .with_tags(vec!["shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "circle",
                    "Circle",
                    ItemPreview::Icon("●".to_string()),
                    ComponentData::new(LibraryShapeType::Circle),
                )
                .with_description("A circle shape")
                .with_tags(vec!["shape".to_string()]),
            );

        let library = ComponentLibrary::new("general", "General").with_category(category);

        let results = library.search_items("rect");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "rect");

        let results = library.search_items("shape");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_style_builder() {
        let style = ComponentStyle::default()
            .with_fill_color("#3366cc")
            .with_stroke_color("#ffffff")
            .with_stroke_width(2.0)
            .with_opacity(0.8);

        assert_eq!(style.fill_color, Some("#3366cc".to_string()));
        assert_eq!(style.stroke_color, Some("#ffffff".to_string()));
        assert_eq!(style.stroke_width, Some(2.0));
        assert_eq!(style.opacity, Some(0.8));
    }

    #[test]
    fn test_style_opacity_clamping() {
        let style = ComponentStyle::default().with_opacity(1.5);
        assert_eq!(style.opacity, Some(1.0));

        let style = ComponentStyle::default().with_opacity(-0.5);
        assert_eq!(style.opacity, Some(0.0));
    }
}
