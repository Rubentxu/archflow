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

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod camera_controller;
pub mod crdt;
pub mod gizmos;
pub mod history;
pub mod hit_testing;
pub mod input;

pub use camera_controller::CameraController;
pub use crdt::{ConflictResolution, CrdtManager, RemoteCommand};
pub use gizmos::GizmoRenderer;
pub use history::{HistoryCommands, HistoryManager, UndoEntry, DEFAULT_MAX_DEPTH};
pub use hit_testing::HitTester;
pub use input::{
    Buttons, InputEventType, InputProcessor, InputRingBuffer, Modifiers, RawInputEvent,
    SelectionState, EVENT_CAPACITY, EVENT_SIZE, MAX_POINTERS,
};
