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

// TODO: Implement TerraformExporter, MermaidExporter, ProjectSerializer
// See: ARQUITECTURA_FINAL_V3.md - Sections 19, 20

pub mod mermaid;
pub mod serialization;
pub mod terraform;

pub use mermaid::MermaidExporter;
pub use serialization::{ProjectDeserializer, ProjectSerializer};
pub use terraform::TerraformExporter;
