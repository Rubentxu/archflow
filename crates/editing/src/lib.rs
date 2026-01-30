//! # ArchFlow Editing - Bounded Context for Editing Operations
//!
//! This crate consolidates all editing-related functionality from the old architecture:
//! - **Commands** (from `archflow-sdk/src/commands/`): Create, Delete, Move, Update commands
//! - **Undo/Redo** (from `archflow-workspace/`): Command history, event sourcing
//! - **Alignment** (from `archflow-sdk/src/alignment/`): Shape alignment and distribution
//! - **Grouping** (from `archflow-sdk/src/group/`): Group/ungroup operations
//! - **Text Editing** (from `archflow-sdk/src/text/`): Text creation and editing
//!
//! # Architecture
//!
//! This bounded context follows the **Connascence of Meaning** principle:
//! - All concepts share the same domain language (Command, Undo, Redo, Align, Group)
//! - High cohesion: changes to editing operations stay localized
//! - Low coupling: depends only on `archflow-core` for shared types
//!
//! # Migration
//!
//! This crate replaces:
//! - `archflow-sdk/src/commands/` → `crates/editing/src/commands/`
//! - `archflow-sdk/src/alignment/` → `crates/editing/src/alignment/`
//! - `archflow-sdk/src/group/` → `crates/editing/src/group/`
//! - `archflow-sdk/src/text/` → `crates/editing/src/text/`
//! - `archflow-workspace/` (undo/redo part) → `crates/editing/src/history/`

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

mod command;
mod history;

pub use command::{Command, CommandError, CommandExecutor, CommandResult};
pub use history::{HistoryConfig, HistoryManager};

/// Re-export core types for convenience
pub use archflow_core::EntityId;

#[cfg(test)]
mod editing_tests;
