//! Plugin System for ArchFlow SDK
//!
//! This module provides a plugin architecture for extending the SDK with
//! custom functionality. Plugins can hook into various lifecycle events,
//! add new tools, and register custom shape renderers.

use crate::a11y::{KeyCode, KeyEvent, Modifiers};
use crate::canvas::Shape;
use crate::events::CanvasEvent;
use crate::layers::C4Level;
use crate::selection::SelectionDelta;
use crate::viewport::Viewport;
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a plugin
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(pub String);

impl PluginId {
    /// Creates a new plugin ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Version information for a plugin
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Pre-release identifier
    pub pre: String,
}

impl PluginVersion {
    /// Creates a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: String::new(),
        }
    }

    /// Creates a pre-release version
    pub fn pre_release(major: u32, minor: u32, patch: u32, pre: impl Into<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: pre.into(),
        }
    }

    /// Returns the version as a string
    pub fn to_string(&self) -> String {
        if self.pre.is_empty() {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, self.pre)
        }
    }
}

/// Metadata for a plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: PluginId,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: PluginVersion,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Compatible SDK version range
    pub sdk_version_range: String,
}

/// A plugin dependency
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Plugin ID to depend on
    pub plugin_id: PluginId,
    /// Version requirement (e.g., ">=1.0.0, <2.0.0")
    pub version_range: String,
}

/// Plugin capability tags
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Tool provider
    Tool,
    /// Shape renderer
    ShapeRenderer,
    /// Event handler
    EventHandler,
    /// Menu contributor
    Menu,
    /// Keyboard shortcut provider
    Shortcut,
    /// Custom property editor
    PropertyEditor,
    /// Export format provider
    Export,
    /// Import format provider
    Import,
    /// Layer type provider
    LayerType,
}

/// A plugin configuration option
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfigOption {
    /// Option key
    pub key: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Default value (JSON)
    pub default_value: serde_json::Value,
    /// Whether it's required
    pub required: bool,
    /// Value type
    pub value_type: ConfigValueType,
}

/// Type of configuration value
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigValueType {
    /// String value
    String,
    /// Number value
    Number,
    /// Boolean value
    Boolean,
    /// Array value
    Array,
    /// Object value
    Object,
}

/// Result of a plugin operation
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(PluginId),
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(PluginId),
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Plugin dependency missing: {0}")]
    MissingDependency(PluginId),
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Plugin disabled")]
    Disabled,
    #[error("Plugin error: {0}")]
    Other(String),
}

/// Plugin lifecycle state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginState {
    /// Not loaded
    NotLoaded,
    /// Loading
    Loading,
    /// Initialized
    Initialized,
    /// Enabled
    Enabled,
    /// Disabled
    Disabled,
    /// Error
    Error,
}

/// Context provided to plugins
#[derive(Debug)]
pub struct PluginContext {
    /// Current viewport
    pub viewport: Viewport,
    /// Current C4 level
    pub c4_level: C4Level,
    /// Mouse position in canvas coordinates
    pub mouse_position: Option<Vec2>,
    /// Current selection
    pub selected_shapes: Vec<EntityId>,
    /// Canvas dimensions
    pub canvas_width: f32,
    /// Canvas height
    pub canvas_height: f32,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            c4_level: C4Level::Context,
            mouse_position: None,
            selected_shapes: Vec::new(),
            canvas_width: 800.0,
            canvas_height: 600.0,
        }
    }
}

/// Plugin trait - the main interface for plugins
pub trait Plugin: Send + Sync {
    /// Returns the plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initializes the plugin
    fn initialize(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called when the plugin is enabled
    fn on_enable(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called when the plugin is disabled
    fn on_disable(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Cleans up the plugin
    fn shutdown(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Returns the capabilities this plugin provides
    fn capabilities(&self) -> Vec<PluginCapability> {
        Vec::new()
    }

    /// Handles canvas events
    fn on_event(&mut self, _event: &CanvasEvent, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// Updates the plugin state
    fn update(&mut self, _context: &PluginContext, _delta_time: f32) -> PluginResult<()> {
        Ok(())
    }

    /// Renders plugin UI overlay
    fn render_ui(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
}

/// Host interface for plugins
pub trait PluginHost {
    /// Gets a plugin by ID
    fn get_plugin(&self, id: &PluginId) -> Option<&dyn Plugin>;

    /// Gets a plugin mutably by ID
    fn get_plugin_mut(&mut self, id: &PluginId) -> Option<&mut Box<dyn Plugin>>;

    /// Registers a tool
    fn register_tool(&mut self, _tool: Box<dyn Tool>) -> PluginResult<()> {
        Ok(())
    }

    /// Registers a shape renderer
    fn register_shape_renderer(
        &mut self,
        _shape_type: &str,
        _renderer: Box<dyn ShapeRenderer>,
    ) -> PluginResult<()> {
        Ok(())
    }

    /// Registers a menu item
    fn register_menu_item(&mut self, _item: MenuItem) -> PluginResult<()> {
        Ok(())
    }

    /// Registers a keyboard shortcut
    fn register_shortcut(&mut self, _shortcut: Shortcut) -> PluginResult<()> {
        Ok(())
    }

    /// Gets the current context
    fn context(&self) -> &PluginContext;

    /// Gets the context mutably
    fn context_mut(&mut self) -> &mut PluginContext;

    /// Emits an event
    fn emit_event(&mut self, _event: CanvasEvent) -> PluginResult<()> {
        Ok(())
    }

    /// Logs a message
    fn log(&self, _level: LogLevel, _message: &str) {
        println!("[PLUGIN] {}", _message);
    }
}

/// Log levels
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
}

/// A tool that can be selected
pub trait Tool: Send + Sync {
    /// Returns the tool ID
    fn id(&self) -> &str;

    /// Returns the tool name
    fn name(&self) -> &str;

    /// Returns the tool icon (SVG or emoji)
    fn icon(&self) -> &str;

    /// Returns the tool category
    fn category(&self) -> ToolCategory;

    /// Called when the tool is selected
    fn on_select(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called when the tool is deselected
    fn on_deselect(&mut self, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called on mouse down
    fn on_mouse_down(&mut self, _position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called on mouse move
    fn on_mouse_move(&mut self, _position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called on mouse up
    fn on_mouse_up(&mut self, _position: Vec2, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Called on key down
    ///
    /// Returns a SelectionDelta if the key event caused selection changes
    fn on_key_down(
        &mut self,
        _event: &KeyEvent,
        _host: &mut dyn PluginHost,
    ) -> PluginResult<Option<SelectionDelta>> {
        Ok(None)
    }

    /// Called on key up
    fn on_key_up(&mut self, _event: &KeyEvent, _host: &mut dyn PluginHost) -> PluginResult<()> {
        Ok(())
    }

    /// Returns the keyboard shortcuts for this tool
    fn keyboard_shortcuts(&self) -> Vec<ToolShortcut> {
        Vec::new()
    }

    /// Renders the tool's cursor or overlay
    fn render_overlay(&self, _context: &PluginContext) -> PluginResult<()> {
        Ok(())
    }
}

/// Tool categories
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    /// Selection tools
    Select,
    /// Shape creation
    Shape,
    /// Drawing
    Draw,
    /// Text
    Text,
    /// Navigation
    Navigate,
    /// Measurement
    Measure,
    /// Custom
    Custom,
}

/// Keyboard shortcut for a tool
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolShortcut {
    /// Keys to press (e.g., "V", "Ctrl+Z", "Shift+Delete")
    pub keys: String,
    /// Description of what the shortcut does
    pub description: String,
    /// Action identifier
    pub action: String,
    /// Key codes that trigger this shortcut
    pub key_codes: Vec<KeyCode>,
    /// Required modifiers
    pub modifiers: Modifiers,
}

impl ToolShortcut {
    /// Creates a new tool shortcut
    pub fn new(
        keys: impl Into<String>,
        description: impl Into<String>,
        action: impl Into<String>,
        key_codes: Vec<KeyCode>,
    ) -> Self {
        Self {
            keys: keys.into(),
            description: description.into(),
            action: action.into(),
            key_codes,
            modifiers: Modifiers::default(),
        }
    }

    /// Creates a shortcut with modifiers
    pub fn with_modifiers(
        keys: impl Into<String>,
        description: impl Into<String>,
        action: impl Into<String>,
        key_codes: Vec<KeyCode>,
        ctrl: bool,
        shift: bool,
        alt: bool,
        meta: bool,
    ) -> Self {
        Self {
            keys: keys.into(),
            description: description.into(),
            action: action.into(),
            key_codes,
            modifiers: Modifiers {
                ctrl,
                shift,
                alt,
                meta,
            },
        }
    }

    /// Checks if a key event matches this shortcut
    pub fn matches(&self, event: &KeyEvent) -> bool {
        if !event.key_down || event.repeated {
            return false;
        }

        if !self.key_codes.contains(&event.key_code) {
            return false;
        }

        event.modifiers.ctrl == self.modifiers.ctrl
            && event.modifiers.shift == self.modifiers.shift
            && event.modifiers.alt == self.modifiers.alt
            && event.modifiers.meta == self.modifiers.meta
    }
}

/// Shape renderer trait
pub trait ShapeRenderer: Send + Sync {
    /// Returns the shape type this renderer handles
    fn shape_type(&self) -> &str;

    /// Renders the shape
    fn render(&self, shape: &Shape, context: &PluginContext) -> PluginResult<()>;

    /// Returns whether this renderer handles hit testing
    fn handles_hit_test(&self) -> bool {
        false
    }

    /// Performs hit testing
    fn hit_test(&self, _shape: &Shape, _point: Vec2) -> bool {
        false
    }
}

/// Menu item for plugins
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuItem {
    /// Item ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Parent menu path (e.g., "Edit/Plugins/MyPlugin")
    pub path: String,
    /// Icon
    pub icon: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<Shortcut>,
    /// Whether it's a separator
    pub is_separator: bool,
    /// Whether it's enabled
    pub enabled: bool,
    /// Whether it's checked
    pub checked: bool,
    /// Action to perform
    pub action: MenuAction,
}

/// Menu action types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MenuAction {
    /// Callback action
    Callback(String),
    /// Toggle action
    Toggle {
        /// State key
        key: String,
        /// Default state
        default_state: bool,
    },
    /// Submenu
    Submenu(Vec<MenuItem>),
}

/// Keyboard shortcut
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    /// Key
    pub key: String,
    /// Modifier keys
    pub modifiers: Vec<Modifier>,
    /// Action
    pub action: String,
}

/// Keyboard modifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// Ctrl or Command
    Ctrl,
    /// Shift
    Shift,
    /// Alt
    Alt,
    /// Meta
    Meta,
}

/// Plugin registry for managing all plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: HashMap<PluginId, RegisteredPlugin>,
    /// Plugin capabilities index
    capabilities: HashMap<PluginCapability, Vec<PluginId>>,
    /// Default tool
    active_tool: Option<String>,
    /// Plugin context
    context: PluginContext,
}

struct RegisteredPlugin {
    plugin: Box<dyn Plugin>,
    state: PluginState,
    load_order: usize,
}

impl PluginRegistry {
    /// Creates a new registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            capabilities: HashMap::new(),
            active_tool: None,
            context: PluginContext::default(),
        }
    }

    /// Registers a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> PluginResult<PluginId> {
        let id = plugin.metadata().id.clone();

        if self.plugins.contains_key(&id) {
            return Err(PluginError::AlreadyRegistered(id));
        }

        // Check dependencies
        for dep in &plugin.metadata().dependencies {
            if !self.plugins.contains_key(&dep.plugin_id) {
                return Err(PluginError::MissingDependency(dep.plugin_id.clone()));
            }
        }

        let plugin_info = RegisteredPlugin {
            plugin,
            state: PluginState::NotLoaded,
            load_order: self.plugins.len(),
        };

        self.plugins.insert(id.clone(), plugin_info);

        Ok(id)
    }

    /// Initializes all plugins
    pub fn initialize_all(&mut self, host: &mut dyn PluginHost) -> PluginResult<()> {
        // Collect plugins in order and initialize
        let mut plugins: Vec<_> = self.plugins.iter_mut().collect();
        plugins.sort_by_key(|(_, p)| p.load_order);

        for (id, plugin_info) in plugins {
            plugin_info.state = PluginState::Loading;
            if let Err(e) = plugin_info.plugin.initialize(host) {
                plugin_info.state = PluginState::Error;
                return Err(PluginError::InitializationFailed(format!(
                    "Plugin {} failed to initialize: {}",
                    id.0, e
                )));
            }
            plugin_info.state = PluginState::Initialized;
        }

        // Enable all initialized plugins
        for plugin_info in self.plugins.values_mut() {
            if let Err(e) = plugin_info.plugin.on_enable(host) {
                plugin_info.state = PluginState::Error;
                return Err(PluginError::InitializationFailed(format!(
                    "Plugin enable failed: {}",
                    e
                )));
            }
            plugin_info.state = PluginState::Enabled;
        }

        Ok(())
    }

    /// Shuts down all plugins
    pub fn shutdown_all(&mut self, host: &mut dyn PluginHost) -> PluginResult<()> {
        for (id, plugin_info) in self.plugins.iter_mut() {
            if let Err(e) = plugin_info.plugin.shutdown(host) {
                log::warn!("Plugin {} shutdown error: {}", id.0, e);
            }
            plugin_info.state = PluginState::NotLoaded;
        }
        Ok(())
    }

    /// Gets a plugin by ID
    pub fn get(&self, id: &PluginId) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(|p| p.plugin.as_ref())
    }

    /// Gets a plugin mutably by ID - requires mutable borrow for lifetime
    pub fn get_plugin_mut(&mut self, id: &PluginId) -> Option<&mut Box<dyn Plugin>> {
        self.plugins.get_mut(id).map(|p| &mut p.plugin)
    }

    /// Gets all plugins with a capability
    pub fn with_capability(&self, capability: PluginCapability) -> Vec<&dyn Plugin> {
        self.capabilities
            .get(&capability)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
            .unwrap_or_default()
    }

    /// Updates a plugin
    pub fn update(&mut self, delta_time: f32) -> PluginResult<()> {
        for plugin_info in self.plugins.values_mut() {
            if plugin_info.state == PluginState::Enabled {
                plugin_info.plugin.update(&self.context, delta_time)?;
            }
        }
        Ok(())
    }

    /// Handles an event
    pub fn handle_event(&mut self, event: &CanvasEvent) -> PluginResult<()> {
        for plugin_info in self.plugins.values_mut() {
            if plugin_info.state == PluginState::Enabled {
                plugin_info.plugin.on_event(event, &self.context)?;
            }
        }
        Ok(())
    }

    /// Sets the active tool
    pub fn set_active_tool(&mut self, tool_id: &str) {
        self.active_tool = Some(tool_id.to_string());
    }

    /// Gets the active tool
    pub fn active_tool(&self) -> Option<&str> {
        self.active_tool.as_deref()
    }

    /// Updates the context
    pub fn update_context<F>(&mut self, f: F)
    where
        F: FnOnce(&mut PluginContext),
    {
        f(&mut self.context);
    }

    /// Returns number of plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Returns whether empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple plugin host implementation
pub struct SimplePluginHost {
    registry: PluginRegistry,
}

impl SimplePluginHost {
    /// Creates a new host
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }
}

impl Default for SimplePluginHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHost for SimplePluginHost {
    fn get_plugin(&self, id: &PluginId) -> Option<&dyn Plugin> {
        self.registry.get(id)
    }

    fn get_plugin_mut(&mut self, id: &PluginId) -> Option<&mut Box<dyn Plugin>> {
        self.registry.get_plugin_mut(id)
    }

    fn context(&self) -> &PluginContext {
        &self.registry.context
    }

    fn context_mut(&mut self) -> &mut PluginContext {
        &mut self.registry.context
    }

    fn emit_event(&mut self, _event: CanvasEvent) -> PluginResult<()> {
        // Events are handled through the registry directly
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;

    /// Test plugin implementation
    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            static METADATA: std::sync::OnceLock<PluginMetadata> = std::sync::OnceLock::new();
            METADATA.get_or_init(|| PluginMetadata {
                id: PluginId::new("test-plugin"),
                name: "Test Plugin".to_string(),
                version: PluginVersion::new(1, 0, 0),
                description: "A test plugin".to_string(),
                author: "Test".to_string(),
                dependencies: Vec::new(),
                sdk_version_range: ">=0.12.0".to_string(),
            })
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::Tool]
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = TestPlugin;
        let metadata = plugin.metadata();

        assert_eq!(metadata.id.0, "test-plugin");
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.version.to_string(), "1.0.0");
    }

    #[test]
    fn test_plugin_version() {
        let v1 = PluginVersion::new(1, 2, 3);
        assert_eq!(v1.to_string(), "1.2.3");

        let v2 = PluginVersion::pre_release(1, 2, 3, "beta");
        assert_eq!(v2.to_string(), "1.2.3-beta");
    }

    #[test]
    fn test_plugin_registration() {
        let mut host = SimplePluginHost::new();
        let plugin: Box<dyn Plugin> = Box::new(TestPlugin);
        let id = host.registry.register(plugin).unwrap();

        assert_eq!(id.0, "test-plugin");
        assert!(host.get_plugin(&id).is_some());
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();

        let plugin: Box<dyn Plugin> = Box::new(TestPlugin);
        let id = registry.register(plugin).unwrap();

        assert_eq!(id.0, "test-plugin");
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn test_duplicate_plugin() {
        let mut registry = PluginRegistry::new();

        let plugin: Box<dyn Plugin> = Box::new(TestPlugin);
        let plugin2: Box<dyn Plugin> = Box::new(TestPlugin);

        registry.register(plugin).unwrap();
        assert!(matches!(
            registry.register(plugin2),
            Err(PluginError::AlreadyRegistered(_))
        ));
    }

    #[test]
    fn test_plugin_capabilities() {
        let plugin = TestPlugin;
        let caps = plugin.capabilities();

        assert_eq!(caps, vec![PluginCapability::Tool]);
    }

    #[test]
    fn test_plugin_context() {
        let ctx = PluginContext::default();

        assert_eq!(ctx.c4_level, C4Level::Context);
        assert!(ctx.mouse_position.is_none());
        assert_eq!(ctx.canvas_width, 800.0);
        assert_eq!(ctx.canvas_height, 600.0);
    }

    #[test]
    fn test_plugin_config_option() {
        let option = PluginConfigOption {
            key: "test_option".to_string(),
            name: "Test Option".to_string(),
            description: "A test option".to_string(),
            default_value: serde_json::json!(true),
            required: false,
            value_type: ConfigValueType::Boolean,
        };

        assert_eq!(option.key, "test_option");
        assert_eq!(option.value_type, ConfigValueType::Boolean);
    }

    #[test]
    fn test_menu_item() {
        let item = MenuItem {
            id: "test_item".to_string(),
            name: "Test Item".to_string(),
            path: "Edit/Test".to_string(),
            icon: None,
            shortcut: None,
            is_separator: false,
            enabled: true,
            checked: false,
            action: MenuAction::Callback("test_action".to_string()),
        };

        assert_eq!(item.id, "test_item");
        assert_eq!(item.path, "Edit/Test");
    }

    #[test]
    fn test_shortcut() {
        let shortcut = Shortcut {
            key: "s".to_string(),
            modifiers: vec![Modifier::Ctrl],
            action: "save".to_string(),
        };

        assert_eq!(shortcut.key, "s");
        assert_eq!(shortcut.modifiers.len(), 1);
    }

    #[test]
    fn test_plugin_state() {
        assert_eq!(PluginState::NotLoaded, PluginState::NotLoaded);
        assert_ne!(PluginState::NotLoaded, PluginState::Enabled);
    }

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::Debug, LogLevel::Debug);
        assert_ne!(LogLevel::Debug, LogLevel::Error);
    }

    #[test]
    fn test_tool_category() {
        assert_eq!(ToolCategory::Select, ToolCategory::Select);
        assert_ne!(ToolCategory::Select, ToolCategory::Shape);
    }

    #[test]
    fn test_modifier() {
        assert_eq!(Modifier::Ctrl, Modifier::Ctrl);
        assert_ne!(Modifier::Ctrl, Modifier::Shift);
    }

    #[test]
    fn test_registry_len() {
        let mut registry = PluginRegistry::new();
        assert!(registry.is_empty());

        let plugin: Box<dyn Plugin> = Box::new(TestPlugin);
        registry.register(plugin).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }
}
