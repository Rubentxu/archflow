//! # ArchFlow Accessibility - Bounded Context for Accessibility Features
//!
//! This crate consolidates all accessibility-related functionality from the old architecture:
//! - **A11yManager** (from `archflow-sdk/src/a11y/`): Focus management, screen reader support
//! - **Keyboard Navigation**: Spatial navigation, focus modes, keyboard shortcuts
//! - **ARIA Support**: Accessibility tree generation, ARIA attributes
//! - **WCAG 2.1 Compliance**: Focus indicators, high contrast mode, reduced motion
//!
//! # Architecture
//!
//! This bounded context follows the **Connascence of Meaning** principle:
//! - All concepts share the same domain language (Focus, A11yNode, ScreenReader, NavigationMode)
//! - High cohesion: changes to accessibility concepts stay localized
//! - Low coupling: depends only on `archflow-core` for shared types
//!
//! # Migration
//!
//! This crate replaces:
//! - `archflow-sdk/src/a11y/` → `crates/a11y/src/`

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod bounds;
mod config;
mod focus;
mod manager;
mod navigation;

pub use bounds::A11yBounds;
pub use config::{A11yConfig, A11yVerbosity, LiveRegionType};
pub use focus::{A11yAnnouncement, FocusableElement, FocusableType};
pub use manager::{A11yManager, A11yNode, A11yProperties};
pub use navigation::{
    KeyCode, KeyEvent, KeyEventResult, Modifiers, NavigationDirection, NavigationMode,
};

/// Re-export core types for convenience
pub use archflow_core::{EntityId, Vec2};

#[cfg(test)]
mod a11y_tests;

// Mock Canvas trait for building accessibility trees
// This will be replaced with canvas crate dependency when available
pub trait CanvasLike {
    fn all_shapes(&self) -> Vec<ShapeLike>;
    fn all_layers(&self) -> Vec<LayerLike>;
}

#[derive(Clone, Debug)]
pub struct ShapeLike {
    pub id: EntityId,
    pub shape_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct LayerLike {
    pub id: EntityId,
    pub name: String,
    pub visible: bool,
}
