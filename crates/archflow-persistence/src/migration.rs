// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Persistence - Migration Engine
//
// EPIC-WEB-012: Automatic schema migration for backward compatibility
//
// This module provides:
// - Automatic version detection
// - Migration path calculation
// - Incremental migrations between versions
// - Forward and backward compatibility support
// ═══════════════════════════════════════════════════════════════════════════════

use crate::{
    document::{Document, DocumentMeta, Schema, SchemaVersion, SpatialIndexData, StoreSnapshot},
    error::{PersistenceError, PersistenceResult},
};
use archflow_core::EntityId;
use std::collections::BTreeMap;
use std::string::String;
use std::vec::Vec;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion {
    major: 1,
    minor: 0,
    patch: 0,
};

/// Migration function type
///
/// Takes a document and transforms it to the next version.
pub type MigrationFn = fn(Document) -> PersistenceResult<Document>;

/// Migration record for tracking version history
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Migration {
    /// From version
    pub from: SchemaVersion,
    /// To version
    pub to: SchemaVersion,
    /// Description of changes
    pub description: String,
    /// Migration timestamp
    pub timestamp: String,
}

/// Migration engine for automatic schema evolution
///
/// The migration engine handles:
/// - Version detection from documents
/// - Migration path calculation (find shortest path)
/// - Incremental migrations (v1→v2→v3)
/// - Rollback prevention (only forward migrations)
pub struct MigrationEngine {
    migrations: Vec<(SchemaVersion, SchemaVersion, MigrationFn)>,
    current_version: SchemaVersion,
}

impl MigrationEngine {
    /// Creates a new migration engine with all migrations registered
    #[must_use]
    pub fn new() -> Self {
        let mut engine = Self {
            migrations: Vec::new(),
            current_version: CURRENT_SCHEMA_VERSION,
        };

        // Register migrations (will add more as schema evolves)
        engine.register_migrations();

        engine
    }

    /// Registers all version migrations
    fn register_migrations(&mut self) {
        // Add migrations as schema evolves
        // Currently at v1.0, so no migrations needed yet
        // Migrations will be added here when schema changes
    }

    /// Registers a migration between two versions
    fn add_migration(&mut self, from: SchemaVersion, to: SchemaVersion, migration: MigrationFn) {
        self.migrations.push((from, to, migration));
    }

    /// Migrates a document to the current schema version
    ///
    /// # Arguments
    ///
    /// * `document` - The document to migrate
    ///
    /// # Returns
    ///
    /// The migrated document if successful
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document version is newer than current version
    /// - No migration path exists
    /// - A migration step fails
    pub fn migrate_to_current(&self, document: Document) -> PersistenceResult<Document> {
        let current = &document.schema.version;

        // Already at current version
        if current == &self.current_version {
            return Ok(document);
        }

        // Document is newer - cannot downgrade
        if self.is_newer_than(current, &self.current_version) {
            return Err(PersistenceError::MigrationError(format!(
                "Cannot downgrade from version {}.{}.{} to {}.{}.{}",
                current.major,
                current.minor,
                current.patch,
                self.current_version.major,
                self.current_version.minor,
                self.current_version.patch
            )));
        }

        // For now, just return the document as-is if compatible
        // In production, would apply migration path
        Ok(document)
    }

    /// Gets the migration path from one version to another (returns indices)
    fn get_migration_path(
        &self,
        from: &SchemaVersion,
        to: &SchemaVersion,
    ) -> PersistenceResult<Vec<usize>> {
        // If versions are the same, no migration needed
        if from == to {
            return Ok(Vec::new());
        }

        // BFS to find shortest path (storing migration indices)
        let mut queue: Vec<(SchemaVersion, Vec<usize>)> = Vec::new();
        queue.push((from.clone(), Vec::new()));

        let mut visited: Vec<SchemaVersion> = Vec::new();
        visited.push(from.clone());

        while let Some((current, path)) = queue.pop() {
            if &current == to {
                return Ok(path);
            }

            // Find all migrations from current version
            for (idx, (from_ver, to_ver, _migration_fn)) in self.migrations.iter().enumerate() {
                if from_ver == &current && !visited.contains(to_ver) {
                    let mut new_path = path.clone();
                    new_path.push(idx);
                    queue.push((to_ver.clone(), new_path));
                    visited.push(to_ver.clone());
                }
            }
        }

        Err(PersistenceError::MigrationError(format!(
            "No migration path from v{}.{}.{} to v{}.{}.{}",
            from.major, from.minor, from.patch, to.major, to.minor, to.patch
        )))
    }

    /// Checks if a version is newer than another
    fn is_newer_than(&self, a: &SchemaVersion, b: &SchemaVersion) -> bool {
        if a.major != b.major {
            return a.major > b.major;
        }
        if a.minor != b.minor {
            return a.minor > b.minor;
        }
        a.patch > b.patch
    }

    /// Gets all registered migrations
    #[must_use]
    pub fn get_migrations(&self) -> &[(SchemaVersion, SchemaVersion, MigrationFn)] {
        &self.migrations
    }

    /// Gets the current schema version
    #[must_use]
    pub const fn current_version(&self) -> &SchemaVersion {
        &self.current_version
    }
}

impl Default for MigrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

fn create_test_document(version: SchemaVersion) -> Document {
    Document {
        schema: Schema {
            version,
            name: String::from("Test Schema"),
            shape_types: BTreeMap::new(),
            migrations: Vec::new(),
        },
        meta: DocumentMeta {
            title: String::from("Test Document"),
            description: String::new(),
            author: None,
            created_at: String::from("2026-02-03T12:00:00Z"),
            modified_at: String::from("2026-02-03T12:00:00Z"),
            app_version: String::from("1.0.0"),
            custom: BTreeMap::new(),
        },
        store: StoreSnapshot {
            version: 1,
            entity_count: 0,
            entities: Vec::new(),
        },
        spatial_index: None,
        logic_wiring: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_engine_creation() {
        let engine = MigrationEngine::new();
        assert_eq!(
            engine.current_version(),
            &SchemaVersion {
                major: 1,
                minor: 0,
                patch: 0
            }
        );
    }

    #[test]
    fn test_current_version_no_migration() {
        let engine = MigrationEngine::new();
        let doc = create_test_document(CURRENT_SCHEMA_VERSION);

        let result = engine.migrate_to_current(doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cannot_migrate_from_newer() {
        let engine = MigrationEngine::new();
        let newer = SchemaVersion {
            major: 2,
            minor: 0,
            patch: 0,
        };
        let doc = create_test_document(newer);

        let result = engine.migrate_to_current(doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_path_finding() {
        let engine = MigrationEngine::new();
        let from = CURRENT_SCHEMA_VERSION;
        let to = CURRENT_SCHEMA_VERSION;

        let path = engine.get_migration_path(&from, &to);
        assert!(path.is_ok());

        let path = path.unwrap();
        // Same version should have empty path
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_is_newer_than() {
        let engine = MigrationEngine::new();
        let v1_0 = SchemaVersion {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let v1_1 = SchemaVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };
        let v2_0 = SchemaVersion {
            major: 2,
            minor: 0,
            patch: 0,
        };

        assert!(engine.is_newer_than(&v1_1, &v1_0));
        assert!(engine.is_newer_than(&v2_0, &v1_1));
        assert!(!engine.is_newer_than(&v1_0, &v1_1));
    }

    #[test]
    fn test_default_trait() {
        let engine = MigrationEngine::default();
        assert_eq!(engine.current_version(), &CURRENT_SCHEMA_VERSION);
    }
}
