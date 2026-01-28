//! ArchFlow SDK - High-performance Rust-based diagramming SDK for web
//!
//! This crate provides the core SDK functionality for building diagramming
//! applications with ArchFlow. It follows the principle of complete delegation
//! to the Rust engine while exposing a clean API for JavaScript/TypeScript.
//!
//! # Architecture
//!
//! - **Canvas**: Infinite canvas with viewport management
//! - **Background**: Grid and background rendering system
//! - **Layers**: C4 model layer system for multi-level diagrams
//!
//! # Usage
//!
//! This crate is primarily used via WASM bindings. See the JavaScript SDK
//! package for the public TypeScript API.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

pub mod a11y;
pub mod background;
pub mod canvas;
pub mod collab;
pub mod commands;
pub mod events;
pub mod layers;
pub mod plugin;
pub mod selection;
pub mod tools;
pub mod viewport;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use background::{BackgroundRenderer, GridConfig, GridType};
pub use canvas::{Canvas, CanvasError, CanvasOperation};
pub use events::{
    EventBuilder, EventHandler, EventId, EventMetadata, EventSnapshot, EventStore,
    EventStoreConfig, RecordedEvent, UndoManager,
};
pub use layers::{C4Level, Layer, LayerManager};
pub use plugin::{
    MenuAction, MenuItem, Modifier, Plugin, PluginCapability, PluginContext, PluginError,
    PluginHost, PluginId, PluginMetadata, PluginRegistry, PluginState, PluginVersion, Shortcut,
    Tool, ToolCategory, ToolShortcut,
};
pub use selection::{SelectionDelta, SelectionManager, SelectionMode};
pub use viewport::{Viewport, ViewportManager};

pub use a11y::{
    A11yBounds, A11yConfig, A11yManager, A11yNode, A11yProperties, A11yReport, A11ySummary,
    A11yVerbosity, FocusableElement, FocusableType, KeyCode, KeyEvent, KeyboardShortcutHelp,
    LiveRegionType, Modifiers, NavigationDirection, NavigationMode, ShortcutInfo,
};
pub use collab::{
    CollabConfig, CollabError, CollabManager, CollabMergeResult, CollabRecord, CursorPosition,
    PresenceManager, UserInfo, UserPresence, UserSelection,
};
pub use commands::{
    Command, CommandError, CommandExecutor, CommandResult, CreateRectangleCommand,
    DeleteShapeCommand, MoveShapeCommand,
};
pub use tools::{
    CursorType, DrawShapeType, DrawTool, EraseMode, EraseTool, MouseButton, MouseEvent,
    ResizeHandle, SelectTool, SelectToolState, ToolError, ToolResult,
};

/// Re-export core types for convenience
pub use archflow_core::{Color, EntityId, Vec2};
pub use archflow_records::RecordStore;
