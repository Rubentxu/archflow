// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Export - Serialization & IaC Export
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 19, 20
//
// This crate contains export functionality:
// - Terraform code generation from diagrams
// - Mermaid diagram export
// - FlatBuffers zero-copy serialization
// ═══════════════════════════════════════════════════════════════════════════════

//! # ArchFlow Export - Serialization & Infrastructure as Code
//!
//! This crate provides export functionality for ArchFlow projects:
//! - Binary project serialization/deserialization
//! - Terraform HCL code generation for cloud deployment
//! - Mermaid diagram export for documentation
//!
//! ## Architecture Reference
//!
//! See `ARQUITECTURA_FINAL_V3.md` Sections 19, 20 for detailed design.

// TODO: Implement TerraformExporter, MermaidExporter, ProjectSerializer
// See: ARQUITECTURA_FINAL_V3.md - Sections 19, 20

/// Binary serialization module for project save/load
pub mod mermaid;
/// Terraform HCL code generation
pub mod serialization;
/// Mermaid diagram export
pub mod terraform;

pub use mermaid::MermaidExporter;
pub use serialization::{ProjectDeserializer, ProjectSerializer};
pub use terraform::TerraformExporter;
