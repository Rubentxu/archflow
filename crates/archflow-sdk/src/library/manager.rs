//! Library Manager
//!
//! Manages all component libraries including built-in and user-created libraries.

use super::*;
use std::collections::HashMap;

/// Error type for library operations
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
    #[error("Category not found: {0}")]
    CategoryNotFound(String),
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Manages component libraries
#[derive(Debug)]
pub struct LibraryManager {
    /// All registered libraries
    libraries: HashMap<String, ComponentLibrary>,
    /// IDs of active (visible) libraries
    active_library_ids: Vec<String>,
    /// IDs of favorited items
    favorites: Vec<String>,
    /// Recently used item IDs
    recent_items: Vec<String>,
    /// Maximum number of recent items to track
    max_recent_items: usize,
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryManager {
    /// Creates a new library manager with built-in libraries loaded
    pub fn new() -> Self {
        let mut manager = Self {
            libraries: HashMap::new(),
            active_library_ids: Vec::new(),
            favorites: Vec::new(),
            recent_items: Vec::new(),
            max_recent_items: 20,
        };

        manager.load_builtin_libraries();
        manager
    }

    /// Loads all built-in libraries
    fn load_builtin_libraries(&mut self) {
        self.register_library(Self::create_general_library());
        self.register_library(Self::create_flowchart_library());
        self.register_library(Self::create_uml_library());
        self.register_library(Self::create_c4_library());
    }

    /// Creates the General shapes library
    fn create_general_library() -> ComponentLibrary {
        let basic_category = LibraryCategory::new("basic", "Basic Shapes", "⬜")
            .with_item(
                LibraryItem::new(
                    "rect",
                    "Rectangle",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rectangle".to_string(),
                        width: 24.0,
                        height: 18.0,
                        stroke_color: "#ffffff".to_string(),
                        fill_color: "#3366cc".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::Rectangle)
                        .with_geometry(ComponentGeometry::new(120.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#3366cc")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Simple rectangle shape")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "rounded-rect",
                    "Rounded Rectangle",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rounded-rect".to_string(),
                        width: 24.0,
                        height: 18.0,
                        stroke_color: "#ffffff".to_string(),
                        fill_color: "#33aa66".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::RoundedRectangle { radius: 8.0 })
                        .with_geometry(ComponentGeometry::new(120.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#33aa66")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Rectangle with rounded corners")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "circle",
                    "Circle",
                    ItemPreview::Icon("●".to_string()),
                    ComponentData::new(LibraryShapeType::Circle)
                        .with_geometry(ComponentGeometry::new(80.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#ff8800")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Perfect circle")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "ellipse",
                    "Ellipse",
                    ItemPreview::Icon("⬭".to_string()),
                    ComponentData::new(LibraryShapeType::Ellipse)
                        .with_geometry(ComponentGeometry::new(120.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#9933cc")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Oval shape")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "diamond",
                    "Diamond",
                    ItemPreview::Icon("◆".to_string()),
                    ComponentData::new(LibraryShapeType::Diamond)
                        .with_geometry(ComponentGeometry::new(100.0, 100.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#ff3366")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Diamond/rhombus shape")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "triangle",
                    "Triangle",
                    ItemPreview::Icon("▲".to_string()),
                    ComponentData::new(LibraryShapeType::Triangle)
                        .with_geometry(ComponentGeometry::new(100.0, 87.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#ffcc00")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Triangle shape")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "hexagon",
                    "Hexagon",
                    ItemPreview::Icon("⬡".to_string()),
                    ComponentData::new(LibraryShapeType::Hexagon)
                        .with_geometry(ComponentGeometry::new(100.0, 87.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#00ccff")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(1.0),
                        ),
                )
                .with_description("Hexagon shape")
                .with_tags(vec!["basic".to_string(), "shape".to_string()]),
            );

        ComponentLibrary::new("general", "General")
            .with_description("Basic geometric shapes")
            .with_category(basic_category)
    }

    /// Creates the Flowchart library
    fn create_flowchart_library() -> ComponentLibrary {
        let symbols_category = LibraryCategory::new("symbols", "Flowchart Symbols", "📊")
            .with_item(
                LibraryItem::new(
                    "start-end",
                    "Start/End",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rounded-rect".to_string(),
                        width: 24.0,
                        height: 14.0,
                        stroke_color: "#ffffff".to_string(),
                        fill_color: "#33aa66".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::RoundedRectangle { radius: 20.0 })
                        .with_geometry(ComponentGeometry::new(120.0, 60.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#33aa66")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("Start or end of process")
                .with_tags(vec!["flowchart".to_string(), "terminal".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "process",
                    "Process",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rectangle".to_string(),
                        width: 24.0,
                        height: 16.0,
                        stroke_color: "#ffffff".to_string(),
                        fill_color: "#3366cc".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::Rectangle)
                        .with_geometry(ComponentGeometry::new(120.0, 60.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#3366cc")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("Process or action step")
                .with_tags(vec!["flowchart".to_string(), "process".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "decision",
                    "Decision",
                    ItemPreview::Icon("◆".to_string()),
                    ComponentData::new(LibraryShapeType::Diamond)
                        .with_geometry(ComponentGeometry::new(100.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#ffcc00")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("Decision point with yes/no branches")
                .with_tags(vec!["flowchart".to_string(), "decision".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "database",
                    "Database",
                    ItemPreview::Icon("🛢️".to_string()),
                    ComponentData::new(LibraryShapeType::Cylinder)
                        .with_geometry(ComponentGeometry::new(100.0, 100.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#9933cc")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("Database storage")
                .with_tags(vec!["flowchart".to_string(), "data".to_string()]),
            );

        ComponentLibrary::new("flowchart", "Flowchart")
            .with_description("Standard flowchart symbols")
            .with_category(symbols_category)
    }

    /// Creates the UML library
    fn create_uml_library() -> ComponentLibrary {
        let uml_category = LibraryCategory::new("diagrams", "UML Elements", "🏗️")
            .with_item(
                LibraryItem::new(
                    "class",
                    "Class",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "class-box".to_string(),
                        width: 24.0,
                        height: 20.0,
                        stroke_color: "#ffffff".to_string(),
                        fill_color: "#3366cc".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::Rectangle)
                        .with_geometry(ComponentGeometry::new(140.0, 100.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#3366cc")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("UML Class")
                .with_tags(vec!["uml".to_string(), "class".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "actor",
                    "Actor",
                    ItemPreview::Icon("👤".to_string()),
                    ComponentData::new(LibraryShapeType::Circle)
                        .with_geometry(ComponentGeometry::new(60.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#a0a0a0")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("UML Actor (user or external system)")
                .with_tags(vec!["uml".to_string(), "actor".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "use-case",
                    "Use Case",
                    ItemPreview::Icon("⬭".to_string()),
                    ComponentData::new(LibraryShapeType::Ellipse)
                        .with_geometry(ComponentGeometry::new(140.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#ffcc00")
                                .with_stroke_color("#ffffff")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("UML Use Case")
                .with_tags(vec!["uml".to_string(), "use-case".to_string()]),
            );

        ComponentLibrary::new("uml", "UML")
            .with_description("UML diagram elements")
            .with_category(uml_category)
    }

    /// Creates the C4 Model library
    fn create_c4_library() -> ComponentLibrary {
        let c4_category = LibraryCategory::new("c4-elements", "C4 Model Elements", "🏛️")
            .with_item(
                LibraryItem::new(
                    "person",
                    "Person",
                    ItemPreview::Icon("👤".to_string()),
                    ComponentData::new(LibraryShapeType::Circle)
                        .with_geometry(ComponentGeometry::new(80.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#08427b")
                                .with_stroke_color("#052e56")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("C4 Model: Person (user or role)")
                .with_tags(vec![
                    "c4".to_string(),
                    "context".to_string(),
                    "person".to_string(),
                ]),
            )
            .with_item(
                LibraryItem::new(
                    "system",
                    "Software System",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rounded-rect".to_string(),
                        width: 26.0,
                        height: 18.0,
                        stroke_color: "#0b4884".to_string(),
                        fill_color: "#1168bd".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::RoundedRectangle { radius: 8.0 })
                        .with_geometry(ComponentGeometry::new(200.0, 100.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#1168bd")
                                .with_stroke_color("#0b4884")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("C4 Model: Software System")
                .with_tags(vec![
                    "c4".to_string(),
                    "context".to_string(),
                    "system".to_string(),
                ]),
            )
            .with_item(
                LibraryItem::new(
                    "container",
                    "Container",
                    ItemPreview::CustomShape(ShapeDefinition {
                        shape_type: "rounded-rect".to_string(),
                        width: 22.0,
                        height: 16.0,
                        stroke_color: "#2e6299".to_string(),
                        fill_color: "#438dd5".to_string(),
                    }),
                    ComponentData::new(LibraryShapeType::RoundedRectangle { radius: 4.0 })
                        .with_geometry(ComponentGeometry::new(160.0, 80.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#438dd5")
                                .with_stroke_color("#2e6299")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("C4 Model: Container (application or data store)")
                .with_tags(vec!["c4".to_string(), "container".to_string()]),
            )
            .with_item(
                LibraryItem::new(
                    "component",
                    "Component",
                    ItemPreview::Icon("⬡".to_string()),
                    ComponentData::new(LibraryShapeType::Hexagon)
                        .with_geometry(ComponentGeometry::new(120.0, 100.0))
                        .with_style(
                            ComponentStyle::default()
                                .with_fill_color("#85bbf0")
                                .with_stroke_color("#5a8fc4")
                                .with_stroke_width(2.0),
                        ),
                )
                .with_description("C4 Model: Component (group of related functionality)")
                .with_tags(vec!["c4".to_string(), "component".to_string()]),
            );

        ComponentLibrary::new("c4-model", "C4 Model")
            .with_description("C4 Model architecture diagrams")
            .with_category(c4_category)
    }

    /// Registers a library
    pub fn register_library(&mut self, library: ComponentLibrary) {
        let id = library.id.clone();
        self.active_library_ids.push(id.clone());
        self.libraries.insert(id, library);
    }

    /// Gets a library by ID
    pub fn get_library(&self, id: &str) -> Option<&ComponentLibrary> {
        self.libraries.get(id)
    }

    /// Gets all libraries
    pub fn get_all_libraries(&self) -> Vec<&ComponentLibrary> {
        self.libraries.values().collect()
    }

    /// Gets active libraries
    pub fn get_active_libraries(&self) -> Vec<&ComponentLibrary> {
        self.active_library_ids
            .iter()
            .filter_map(|id| self.libraries.get(id))
            .collect()
    }

    /// Removes a library
    pub fn remove_library(&mut self, id: &str) -> Result<(), LibraryError> {
        let library = self
            .libraries
            .get(id)
            .ok_or_else(|| LibraryError::LibraryNotFound(id.to_string()))?;

        if library.metadata.is_builtin {
            return Err(LibraryError::LibraryNotFound(
                "Cannot remove built-in library".to_string(),
            ));
        }

        self.libraries.remove(id);
        self.active_library_ids.retain(|lid| lid != id);
        Ok(())
    }

    /// Searches for items across all libraries
    pub fn search_items(&self, query: &str) -> Vec<(&ComponentLibrary, &LibraryItem)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for library in self.libraries.values() {
            for category in &library.categories {
                for item in &category.items {
                    if item.name.to_lowercase().contains(&query_lower)
                        || item.description.to_lowercase().contains(&query_lower)
                        || item
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query_lower))
                    {
                        results.push((library, item));
                    }
                }
            }
        }

        results
    }

    /// Gets an item from a specific library
    pub fn get_item(&self, library_id: &str, item_id: &str) -> Result<&LibraryItem, LibraryError> {
        let library = self
            .get_library(library_id)
            .ok_or_else(|| LibraryError::LibraryNotFound(library_id.to_string()))?;

        for category in &library.categories {
            if let Some(item) = category.items.iter().find(|i| i.id == item_id) {
                return Ok(item);
            }
        }

        Err(LibraryError::ItemNotFound(item_id.to_string()))
    }

    /// Adds an item to recent items
    pub fn add_to_recent(&mut self, library_id: &str, item_id: &str) {
        let key = format!("{}:{}", library_id, item_id);

        // Remove if already exists
        self.recent_items.retain(|k| k != &key);

        // Add to front
        self.recent_items.insert(0, key);

        // Trim to max size
        if self.recent_items.len() > self.max_recent_items {
            self.recent_items.truncate(self.max_recent_items);
        }
    }

    /// Gets recent items
    pub fn get_recent_items(&self) -> Vec<(&ComponentLibrary, &LibraryItem)> {
        self.recent_items
            .iter()
            .filter_map(|key| {
                let parts: Vec<&str> = key.splitn(2, ':').collect();
                if parts.len() == 2 {
                    self.get_item(parts[0], parts[1]).ok().map(|item| {
                        let lib = self.get_library(parts[0]).unwrap();
                        (lib, item)
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Adds an item to favorites
    pub fn add_to_favorites(&mut self, library_id: &str, item_id: &str) {
        let key = format!("{}:{}", library_id, item_id);
        if !self.favorites.contains(&key) {
            self.favorites.push(key);
        }
    }

    /// Removes an item from favorites
    pub fn remove_from_favorites(&mut self, library_id: &str, item_id: &str) {
        let key = format!("{}:{}", library_id, item_id);
        self.favorites.retain(|k| k != &key);
    }

    /// Gets favorited items
    pub fn get_favorites(&self) -> Vec<(&ComponentLibrary, &LibraryItem)> {
        self.favorites
            .iter()
            .filter_map(|key| {
                let parts: Vec<&str> = key.splitn(2, ':').collect();
                if parts.len() == 2 {
                    self.get_item(parts[0], parts[1]).ok().map(|item| {
                        let lib = self.get_library(parts[0]).unwrap();
                        (lib, item)
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Imports a library from JSON string
    pub fn import_library(&mut self, json: &str) -> Result<(), LibraryError> {
        let library: ComponentLibrary = serde_json::from_str(json)?;
        self.register_library(library);
        Ok(())
    }

    /// Exports a library to JSON string
    pub fn export_library(&self, library_id: &str) -> Result<String, LibraryError> {
        let library = self
            .get_library(library_id)
            .ok_or_else(|| LibraryError::LibraryNotFound(library_id.to_string()))?;

        Ok(serde_json::to_string_pretty(library)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_manager_new() {
        let manager = LibraryManager::new();
        assert_eq!(manager.get_all_libraries().len(), 4); // Built-in libraries
        assert_eq!(manager.get_active_libraries().len(), 4);
    }

    #[test]
    fn test_get_library() {
        let manager = LibraryManager::new();
        let library = manager.get_library("general");
        assert!(library.is_some());
        assert_eq!(library.unwrap().id, "general");
    }

    #[test]
    fn test_get_library_not_found() {
        let manager = LibraryManager::new();
        assert!(manager.get_library("nonexistent").is_none());
    }

    #[test]
    fn test_search_items() {
        let manager = LibraryManager::new();
        let results = manager.search_items("rect");
        assert!(!results.is_empty());

        let results = manager.search_items("circle");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_item() {
        let manager = LibraryManager::new();
        let item = manager.get_item("general", "rect");
        assert!(item.is_ok());
        assert_eq!(item.unwrap().id, "rect");
    }

    #[test]
    fn test_get_item_not_found() {
        let manager = LibraryManager::new();
        let result = manager.get_item("general", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_recent_items() {
        let mut manager = LibraryManager::new();

        manager.add_to_recent("general", "rect");
        manager.add_to_recent("general", "circle");
        manager.add_to_recent("general", "rect"); // Duplicate should move to front

        let recent = manager.get_recent_items();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].1.id, "rect"); // Most recent first
    }

    #[test]
    fn test_favorites() {
        let mut manager = LibraryManager::new();

        manager.add_to_favorites("general", "rect");
        manager.add_to_favorites("general", "circle");
        manager.add_to_favorites("general", "rect"); // Duplicate should be ignored

        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 2);

        manager.remove_from_favorites("general", "rect");
        let favorites = manager.get_favorites();
        assert_eq!(favorites.len(), 1);
    }

    #[test]
    fn test_import_export_library() {
        let mut manager = LibraryManager::new();

        let library = ComponentLibrary::new("test", "Test Library");
        let json = serde_json::to_string(&library).unwrap();

        manager.import_library(&json).unwrap();
        assert!(manager.get_library("test").is_some());

        let exported = manager.export_library("test").unwrap();
        assert!(exported.contains("Test Library"));
    }

    #[test]
    fn test_c4_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("c4-model");
        assert!(library.is_some());

        let item = manager.get_item("c4-model", "person");
        assert!(item.is_ok());
        assert_eq!(item.unwrap().name, "Person");
    }

    #[test]
    fn test_flowchart_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("flowchart");
        assert!(library.is_some());

        let item = manager.get_item("flowchart", "decision");
        assert!(item.is_ok());
        assert_eq!(item.unwrap().name, "Decision");
    }
}
