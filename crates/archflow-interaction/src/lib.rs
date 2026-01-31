// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - Hit Testing, Camera, Input Processing
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 6, 7, 13, 15
//
// This crate contains the interaction layer:
// - Hit testing with O(1) spatial queries
// - Camera 2D infinite with zoom-to-cursor
// - Input processing via SharedArrayBuffer
// - Gizmo renderer for UI
// ═══════════════════════════════════════════════════════════════════════════════

// TODO: Implement HitTester, CameraController, InputProcessor, GizmoRenderer
// See: ARQUITECTURA_FINAL_V3.md - Sections 6, 7, 13, 15

pub mod camera_controller;
pub mod gizmos;
pub mod hit_testing;
pub mod input;

pub use camera_controller::CameraController;
pub use gizmos::GizmoRenderer;
pub use hit_testing::HitTester;
pub use input::InputProcessor;
