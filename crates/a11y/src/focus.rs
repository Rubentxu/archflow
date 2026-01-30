//! Focusable elements and focus management

use crate::{A11yBounds, EntityId};
use serde::{Deserialize, Serialize};

/// Focusable element in the canvas
#[derive(Clone, Debug)]
pub struct FocusableElement {
    /// Element ID
    pub id: EntityId,
    /// Element type
    pub element_type: FocusableType,
    /// Display name
    pub name: String,
    /// Bounds
    pub bounds: A11yBounds,
    /// Order in focus sequence
    pub focus_order: usize,
}

/// Types of focusable elements
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusableType {
    /// Shape element
    Shape,
    /// Layer
    Layer,
    /// Tool
    Tool,
    /// Menu item
    MenuItem,
    /// Panel
    Panel,
}

impl FocusableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FocusableType::Shape => "shape",
            FocusableType::Layer => "layer",
            FocusableType::Tool => "tool",
            FocusableType::MenuItem => "menu item",
            FocusableType::Panel => "panel",
        }
    }
}

/// Screen reader announcement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yAnnouncement {
    /// Announcement text
    pub text: String,
    /// Priority
    pub priority: crate::LiveRegionType,
    /// Whether to interrupt
    pub interrupt: bool,
}
