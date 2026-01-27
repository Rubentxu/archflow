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

pub mod background;
pub mod canvas;
pub mod layers;
pub mod viewport;

pub use background::{BackgroundRenderer, GridConfig, GridType};
pub use canvas::{Canvas, CanvasError};
pub use layers::{C4Level, Layer, LayerManager};
pub use viewport::{Viewport, ViewportManager};

/// Re-export core types for convenience
pub use archflow_core::{Color, EntityId, Vec2};
pub use archflow_records::RecordStore;
