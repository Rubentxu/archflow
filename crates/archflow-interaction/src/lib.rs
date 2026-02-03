// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - Hit Testing, Camera, Input Processing, History, CRDT
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 6, 7, 13, 15, 16, 17
//
// This crate contains the interaction layer:
// - Hit testing with O(1) spatial queries
// - Camera 2D infinite with zoom-to-cursor
// - Input processing via SharedArrayBuffer
// - Gizmo renderer for UI
// - History manager for undo/redo
// - CRDT manager for real-time collaboration
// ═══════════════════════════════════════════════════════════════════════════════

//! # ArchFlow Interaction - User Input and State Management
//!
//! This crate provides the interaction layer for ArchFlow:
//! - Hit testing with O(1) spatial queries using SpatialHash
//! - Camera 2D infinite canvas with zoom-to-cursor
//! - Input processing via SharedArrayBuffer for WASM
//! - Gizmo renderer for transform UI
//! - History manager for undo/redo operations
//! - CRDT manager for real-time collaboration
//!
//! ## Architecture Reference
//!
//! See `ARQUITECTURA_FINAL_V3.md` Sections 6, 7, 13, 15, 16, 17 for detailed design.

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

/// Camera controller for 2D infinite canvas
pub mod camera_controller;
/// CRDT manager for real-time collaboration
pub mod crdt;
/// Gizmo renderer for transform UI
pub mod gizmos;
/// History manager for undo/redo
pub mod history;
/// Hit testing with spatial queries
pub mod hit_testing;
/// Input processing for keyboard and mouse
pub mod input;

pub use camera_controller::CameraController;
pub use crdt::{ConflictResolution, CrdtManager, RemoteCommand};
pub use gizmos::GizmoRenderer;
pub use history::{DEFAULT_MAX_DEPTH, HistoryCommands, HistoryManager, UndoEntry};
pub use hit_testing::HitTester;
pub use input::{
    Buttons, EVENT_CAPACITY, EVENT_SIZE, InputEventType, InputProcessor, InputRingBuffer,
    MAX_POINTERS, Modifiers, RawInputEvent, SelectionState,
};
