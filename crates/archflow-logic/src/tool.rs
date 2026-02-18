// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Tool Types
//
// Type-safe enum for all editor tools.
// Replaces string-based tool names for compile-time safety.
//
// Tools:
// - Selection tools: select, box_select
// - Creation tools: rectangle, circle, triangle, diamond, square, line, text
// - Navigation: pan, zoom
// - Connection tools: connection
// - Actions: delete
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::{String, ToString};

/// Type-safe tool identifiers for the editor
///
/// Replaces string-based tool names with compile-time type checking.
/// Each variant represents a distinct editing mode in the canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ToolType {
    /// Default selection tool for selecting and manipulating entities
    Select = 0,
    /// Box selection for multi-select operations
    BoxSelect,
    /// Pan tool for canvas navigation
    Pan,
    /// Zoom tool (typically mouse wheel)
    Zoom,
    /// Rectangle shape creation tool
    Rectangle,
    /// Circle/ellipse shape creation tool
    Circle,
    /// Triangle shape creation tool
    Triangle,
    /// Diamond shape creation tool
    Diamond,
    /// Square shape creation tool (constrained rectangle)
    Square,
    /// Line/arrow creation tool
    Line,
    /// Text creation tool
    Text,
    /// Connection/arrow creation tool
    Connection,
    /// Delete tool for removing entities
    Delete,
}

impl ToolType {
    /// Get all available tool types
    pub fn all() -> [ToolType; 13] {
        [
            ToolType::Select,
            ToolType::BoxSelect,
            ToolType::Pan,
            ToolType::Zoom,
            ToolType::Rectangle,
            ToolType::Circle,
            ToolType::Triangle,
            ToolType::Diamond,
            ToolType::Square,
            ToolType::Line,
            ToolType::Text,
            ToolType::Connection,
            ToolType::Delete,
        ]
    }

    /// Check if this tool is a creation tool
    pub fn is_creation_tool(self) -> bool {
        matches!(
            self,
            ToolType::Rectangle
                | ToolType::Circle
                | ToolType::Triangle
                | ToolType::Diamond
                | ToolType::Square
                | ToolType::Line
                | ToolType::Text
                | ToolType::Connection
        )
    }

    /// Check if this tool is a selection tool
    pub fn is_selection_tool(self) -> bool {
        matches!(self, ToolType::Select | ToolType::BoxSelect)
    }

    /// Check if this tool is a navigation tool
    pub fn is_navigation_tool(self) -> bool {
        matches!(self, ToolType::Pan | ToolType::Zoom)
    }

    /// Get default keyboard shortcut for this tool
    pub fn shortcut(self) -> Option<char> {
        match self {
            ToolType::Select => Some('V'),
            ToolType::BoxSelect => Some('B'),
            ToolType::Pan => Some('H'),
            ToolType::Zoom => Some('Z'),
            ToolType::Rectangle => Some('R'),
            ToolType::Circle => Some('C'),
            ToolType::Triangle => Some('T'),
            ToolType::Diamond => Some('D'),
            ToolType::Square => Some('U'), // U for "Square" (alternative)
            ToolType::Line => Some('L'),
            ToolType::Text => Some('X'),       // X for "Text"
            ToolType::Connection => Some('A'), // A for "Arrow"
            ToolType::Delete => Some('X'),     // X for "Delete" (alternative to text)
        }
    }

    /// Convert tool to lowercase string (for compatibility)
    pub fn as_str(self) -> &'static str {
        match self {
            ToolType::Select => "select",
            ToolType::BoxSelect => "box_select",
            ToolType::Pan => "pan",
            ToolType::Zoom => "zoom",
            ToolType::Rectangle => "rectangle",
            ToolType::Circle => "circle",
            ToolType::Triangle => "triangle",
            ToolType::Diamond => "diamond",
            ToolType::Square => "square",
            ToolType::Line => "line",
            ToolType::Text => "text",
            ToolType::Connection => "connection",
            ToolType::Delete => "delete",
        }
    }

    /// Parse tool from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<ToolType> {
        match s.to_lowercase().as_str() {
            "select" => Some(ToolType::Select),
            "box_select" | "boxselect" => Some(ToolType::BoxSelect),
            "pan" => Some(ToolType::Pan),
            "zoom" => Some(ToolType::Zoom),
            "rectangle" => Some(ToolType::Rectangle),
            "circle" => Some(ToolType::Circle),
            "triangle" => Some(ToolType::Triangle),
            "diamond" => Some(ToolType::Diamond),
            "square" => Some(ToolType::Square),
            "line" => Some(ToolType::Line),
            "text" => Some(ToolType::Text),
            "connection" | "arrow" => Some(ToolType::Connection),
            "delete" => Some(ToolType::Delete),
            _ => None,
        }
    }
}

impl Default for ToolType {
    fn default() -> Self {
        ToolType::Select
    }
}

impl From<ToolType> for String {
    fn from(tool: ToolType) -> String {
        tool.as_str().to_string()
    }
}

impl TryFrom<&str> for ToolType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ToolType::from_str(value).ok_or("Unknown tool type")
    }
}

impl TryFrom<String> for ToolType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ToolType::from_str(&value).ok_or("Unknown tool type")
    }
}

impl core::fmt::Display for ToolType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Serde support for ToolType serialization
mod serde_support {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for ToolType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for ToolType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            ToolType::from_str(&s).ok_or_else(|| serde::de::Error::custom("Unknown tool type"))
        }
    }
}

/// Default tool when editor starts
pub const DEFAULT_TOOL: ToolType = ToolType::Select;

/// Configuration for a specific tool
///
/// Contains tool-specific settings that can be customized by the user.
#[derive(Clone, Debug, Default)]
pub struct ToolConfig {
    /// Grid snapping enabled for this tool
    pub snap_to_grid: bool,
    /// Smart guides enabled for this tool
    pub smart_guides: bool,
    /// Creation mode: drag-to-create vs click-to-create
    pub drag_to_create: bool,
    /// Default size for shapes created with this tool
    pub default_width: f32,
    pub default_height: f32,
}

/// Global tool state manager
///
/// Since there's only one active tool at a time, this is a singleton-style
/// state container rather than an ECS component per entity.
///
/// # Usage
///
/// ```
/// use archflow_logic::{ToolState, ToolType, DEFAULT_TOOL};
///
/// let mut state = ToolState::default();
/// assert_eq!(state.active_tool, DEFAULT_TOOL);
///
/// state.set_tool(ToolType::Circle);
/// assert_eq!(state.active_tool, ToolType::Circle);
/// assert_eq!(state.previous_tool(), Some(ToolType::Select));
/// ```
#[derive(Clone, Debug)]
pub struct ToolState {
    /// Currently active tool
    pub active_tool: ToolType,
    /// Tool that was active before the current one
    previous_tool: Option<ToolType>,
    /// Tool-specific configuration
    config: ToolConfig,
    /// Is the user currently creating a shape (mouse down)
    is_creating: bool,
    /// Start position of current creation (if creating)
    creation_start: Option<(f32, f32)>,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            active_tool: DEFAULT_TOOL,
            previous_tool: None,
            config: ToolConfig::default(),
            is_creating: false,
            creation_start: None,
        }
    }
}

impl ToolState {
    /// Create a new ToolState with default settings
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the active tool, tracking the previous one
    #[inline(always)]
    pub fn set_tool(&mut self, tool: ToolType) {
        if tool != self.active_tool {
            self.previous_tool = Some(self.active_tool);
            self.active_tool = tool;
            // Reset creation state when switching tools
            self.is_creating = false;
            self.creation_start = None;
        }
    }

    /// Get the previously active tool
    #[inline(always)]
    pub fn previous_tool(&self) -> Option<ToolType> {
        self.previous_tool
    }

    /// Get tool configuration
    #[inline(always)]
    pub fn config(&self) -> &ToolConfig {
        &self.config
    }

    /// Get mutable tool configuration
    #[inline(always)]
    pub fn config_mut(&mut self) -> &mut ToolConfig {
        &mut self.config
    }

    /// Start creation mode (mouse down)
    #[inline(always)]
    pub fn start_creation(&mut self, x: f32, y: f32) {
        self.is_creating = true;
        self.creation_start = Some((x, y));
    }

    /// End creation mode (mouse up)
    #[inline(always)]
    pub fn end_creation(&mut self) {
        self.is_creating = false;
        self.creation_start = None;
    }

    /// Check if currently creating
    #[inline(always)]
    pub fn is_creating(&self) -> bool {
        self.is_creating
    }

    /// Get creation start position
    #[inline(always)]
    pub fn creation_start(&self) -> Option<(f32, f32)> {
        self.creation_start
    }

    /// Switch back to previous tool
    #[inline(always)]
    pub fn revert_tool(&mut self) {
        if let Some(previous) = self.previous_tool {
            self.active_tool = previous;
            self.previous_tool = None;
        }
    }
}

/// ToolActuator for managing tool transitions and state changes
///
/// Unlike other actuators that operate on entities, this actuator operates
/// on the global tool state. It provides:
///
/// - Tool activation/deactivation with history tracking
/// - Tool-specific configuration
/// - Event callbacks for UI integration
///
/// # Usage
///
/// ```rust
/// use archflow_logic::{ToolActuator, ToolState, ToolType};
///
/// let mut tool_state = ToolState::default();
/// let mut actuator = ToolActuator::new();
///
/// // Activate a tool
/// actuator.activate(&mut tool_state, ToolType::Rectangle);
///
/// // Check if tool is active
/// assert!(actuator.is_active(&tool_state, ToolType::Rectangle));
/// ```
#[derive(Clone, Debug, Default)]
pub struct ToolActuator {
    /// Callbacks for tool activation events
    activation_callback: Option<fn(ToolType)>,
    /// Callbacks for tool deactivation events
    deactivation_callback: Option<fn(ToolType)>,
    /// Track if a tool was activated this frame
    just_activated: bool,
    /// Track which tool was just activated
    last_activated_tool: Option<ToolType>,
}

impl ToolActuator {
    /// Create a new ToolActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate a tool, tracking the previous one
    ///
    /// This will:
    /// 1. Store the current tool as previous
    /// 2. Set the new tool as active
    /// 3. Call any activation callback
    /// 4. Mark as just activated
    pub fn activate(&mut self, state: &mut ToolState, tool: ToolType) {
        let previous = state.active_tool;

        state.set_tool(tool);

        self.last_activated_tool = Some(tool);
        self.just_activated = true;

        // Call activation callback if set
        if let Some(callback) = self.activation_callback {
            callback(tool);
        }

        // Call deactivation callback for previous tool if different
        if let Some(callback) = self.deactivation_callback {
            if previous != tool {
                callback(previous);
            }
        }
    }

    /// Deactivate current tool (revert to previous or default)
    pub fn deactivate(&mut self, state: &mut ToolState) {
        let current = state.active_tool;

        state.revert_tool();

        // Call deactivation callback
        if let Some(callback) = self.deactivation_callback {
            callback(current);
        }

        self.just_activated = false;
        self.last_activated_tool = None;
    }

    /// Check if a specific tool is currently active
    #[inline(always)]
    pub fn is_active(&self, state: &ToolState, tool: ToolType) -> bool {
        state.active_tool == tool
    }

    /// Check if the tool was just activated this frame
    #[inline(always)]
    pub fn was_just_activated(&self) -> bool {
        self.just_activated
    }

    /// Get the tool that was just activated
    #[inline(always)]
    pub fn last_activated(&self) -> Option<ToolType> {
        self.last_activated_tool
    }

    /// Clear the just-activated flag (call at end of frame)
    #[inline(always)]
    pub fn clear_just_activated(&mut self) {
        self.just_activated = false;
    }

    /// Set callback for tool activation
    pub fn on_activate(&mut self, callback: fn(ToolType)) {
        self.activation_callback = Some(callback);
    }

    /// Set callback for tool deactivation
    pub fn on_deactivate(&mut self, callback: fn(ToolType)) {
        self.deactivation_callback = Some(callback);
    }

    /// Configure a tool with specific settings
    pub fn configure(&mut self, state: &mut ToolState, config: ToolConfig) {
        *state.config_mut() = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_type_default() {
        assert_eq!(ToolType::default(), ToolType::Select);
    }

    #[test]
    fn test_tool_type_as_str() {
        assert_eq!(ToolType::Select.as_str(), "select");
        assert_eq!(ToolType::Rectangle.as_str(), "rectangle");
        assert_eq!(ToolType::Circle.as_str(), "circle");
    }

    #[test]
    fn test_tool_type_from_str() {
        assert_eq!(ToolType::from_str("select"), Some(ToolType::Select));
        assert_eq!(ToolType::from_str("SELECT"), Some(ToolType::Select));
        assert_eq!(ToolType::from_str("rectangle"), Some(ToolType::Rectangle));
        assert_eq!(ToolType::from_str("unknown"), None);
    }

    #[test]
    fn test_tool_type_conversion() {
        let tool = ToolType::Circle;
        let s: String = tool.into();
        assert_eq!(s, "circle");
        let back = ToolType::try_from(s).unwrap();
        assert_eq!(back, ToolType::Circle);
    }

    #[test]
    fn test_tool_categories() {
        assert!(ToolType::Select.is_selection_tool());
        assert!(ToolType::BoxSelect.is_selection_tool());
        assert!(!ToolType::Rectangle.is_selection_tool());

        assert!(ToolType::Rectangle.is_creation_tool());
        assert!(!ToolType::Select.is_creation_tool());

        assert!(ToolType::Pan.is_navigation_tool());
        assert!(!ToolType::Select.is_navigation_tool());
    }

    #[test]
    fn test_tool_shortcuts() {
        assert_eq!(ToolType::Select.shortcut(), Some('V'));
        assert_eq!(ToolType::Rectangle.shortcut(), Some('R'));
        assert_eq!(ToolType::Circle.shortcut(), Some('C'));
    }

    #[test]
    fn test_tool_type_display() {
        assert_eq!(format!("{}", ToolType::Select), "select");
        assert_eq!(format!("{}", ToolType::BoxSelect), "box_select");
    }

    #[test]
    fn test_all_tools() {
        let tools = ToolType::all();
        assert_eq!(tools.len(), 13);
        assert!(tools.contains(&ToolType::Select));
        assert!(tools.contains(&ToolType::Delete));
    }
}
