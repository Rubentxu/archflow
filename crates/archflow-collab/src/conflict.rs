//! # Conflict Detection and Resolution Module
//!
//! SOLID-compliant conflict resolution system with extensible strategies,
//! detectors, and notifiers for collaborative editing scenarios.

use crate::types::SiteId;
use archflow_records::{Record, RecordChange, RecordId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Conflict type enumeration with full details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Two sites updated the same record
    UpdateUpdate {
        record_id: RecordId,
        site_a: SiteId,
        site_b: SiteId,
    },
    /// One site updated while another deleted
    UpdateDelete {
        record_id: RecordId,
        updater: SiteId,
        deleter: SiteId,
    },
    /// Two sites inserted records with same ID
    InsertInsert {
        id_a: RecordId,
        id_b: RecordId,
        site_a: SiteId,
        site_b: SiteId,
    },
    /// Nested field conflict (e.g., same sub-property modified)
    NestedField {
        record_id: RecordId,
        field_path: String,
        conflicting_values: Vec<String>,
    },
    /// Structural conflict (parent-child relationship)
    Structural {
        parent_id: RecordId,
        children_conflict: Vec<RecordId>,
    },
}

impl fmt::Display for ConflictType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConflictType::UpdateUpdate { record_id, .. } => {
                write!(f, "UpdateUpdate on record {}", record_id)
            }
            ConflictType::UpdateDelete { record_id, .. } => {
                write!(f, "UpdateDelete on record {}", record_id)
            }
            ConflictType::InsertInsert { id_a, id_b, .. } => {
                write!(f, "InsertInsert: {} vs {}", id_a, id_b)
            }
            ConflictType::NestedField {
                record_id,
                field_path,
                ..
            } => {
                write!(f, "NestedField on {}.{}", record_id, field_path)
            }
            ConflictType::Structural { parent_id, .. } => {
                write!(f, "Structural on parent {}", parent_id)
            }
        }
    }
}

/// Conflict resolution error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionError {
    Unresolvable(String),
    StrategyNotApplicable,
}

impl fmt::Display for ConflictResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConflictResolutionError::Unresolvable(msg) => {
                write!(f, "Conflict cannot be resolved: {}", msg)
            }
            ConflictResolutionError::StrategyNotApplicable => {
                write!(f, "No applicable strategy for this conflict")
            }
        }
    }
}

impl std::error::Error for ConflictResolutionError {}

/// Metrics for conflict tracking
#[derive(Debug, Default)]
pub struct ConflictMetrics {
    total_conflicts: AtomicU64,
    update_update_count: AtomicU64,
    update_delete_count: AtomicU64,
    insert_insert_count: AtomicU64,
    nested_field_count: AtomicU64,
    structural_count: AtomicU64,
}

impl ConflictMetrics {
    pub fn record_conflict(&self, conflict_type: &ConflictType) {
        self.total_conflicts.fetch_add(1, Ordering::SeqCst);
        match conflict_type {
            ConflictType::UpdateUpdate { .. } => {
                self.update_update_count.fetch_add(1, Ordering::SeqCst);
            }
            ConflictType::UpdateDelete { .. } => {
                self.update_delete_count.fetch_add(1, Ordering::SeqCst);
            }
            ConflictType::InsertInsert { .. } => {
                self.insert_insert_count.fetch_add(1, Ordering::SeqCst);
            }
            ConflictType::NestedField { .. } => {
                self.nested_field_count.fetch_add(1, Ordering::SeqCst);
            }
            ConflictType::Structural { .. } => {
                self.structural_count.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    pub fn get_report(&self) -> ConflictReport {
        ConflictReport {
            total_conflicts: self.total_conflicts.load(Ordering::SeqCst),
            update_update: self.update_update_count.load(Ordering::SeqCst),
            update_delete: self.update_delete_count.load(Ordering::SeqCst),
            insert_insert: self.insert_insert_count.load(Ordering::SeqCst),
            nested_field: self.nested_field_count.load(Ordering::SeqCst),
            structural: self.structural_count.load(Ordering::SeqCst),
        }
    }
}

/// Summary report of conflict metrics
#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub total_conflicts: u64,
    pub update_update: u64,
    pub update_delete: u64,
    pub insert_insert: u64,
    pub nested_field: u64,
    pub structural: u64,
}

/// Conflict detector trait - Single Responsibility Principle
pub trait ConflictDetector<R: Record>: Send + Sync {
    fn detect(&self, local: &RecordChange<R>, remote: &RecordChange<R>) -> Option<ConflictType>;
    fn name(&self) -> &'static str;
}

/// Conflict resolver trait - Open/Closed Principle
pub trait ConflictResolver<R: Record>: Send + Sync {
    fn resolve(&self, conflict: &ConflictType) -> Result<RecordChange<R>, ConflictResolutionError>;
    fn name(&self) -> &'static str;
}

/// Conflict notifier trait - Observer pattern
#[async_trait::async_trait]
pub trait ConflictNotifier: Send + Sync {
    async fn notify_conflicts_resolved(&self, applied: &AppliedChange);
}

/// Applied change notification
#[derive(Debug, Clone)]
pub struct AppliedChange {
    pub record_id: RecordId,
    pub resolved_by: SiteId,
    pub timestamp: std::time::SystemTime,
}

/// Conflict resolution strategy - Strategy Pattern
pub trait ConflictResolutionStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, conflict: &ConflictType) -> bool;
    fn resolve(&self, conflict: &ConflictType) -> Result<ResolvedChange, ConflictResolutionError>;
}

/// Resolved change result
#[derive(Debug, Clone)]
pub struct ResolvedChange {
    pub record_id: RecordId,
    pub value: String,
    pub strategy: &'static str,
}

/// Last Writer Wins Strategy
#[derive(Debug, Clone)]
pub struct LastWriterWinsStrategy;

impl LastWriterWinsStrategy {
    pub fn new(_site_id: SiteId) -> Self {
        Self
    }
}

impl ConflictResolutionStrategy for LastWriterWinsStrategy {
    fn name(&self) -> &'static str {
        "LastWriterWins"
    }

    fn can_handle(&self, conflict: &ConflictType) -> bool {
        matches!(conflict, ConflictType::UpdateUpdate { .. })
    }

    fn resolve(&self, conflict: &ConflictType) -> Result<ResolvedChange, ConflictResolutionError> {
        match conflict {
            ConflictType::UpdateUpdate {
                record_id,
                site_a,
                site_b,
            } => {
                let winner = if site_a > site_b { site_a } else { site_b };
                Ok(ResolvedChange {
                    record_id: record_id.clone(),
                    value: format!("winner:{}", winner.as_u32()),
                    strategy: "LastWriterWins",
                })
            }
            _ => Err(ConflictResolutionError::StrategyNotApplicable),
        }
    }
}

/// Multi-Value Register Strategy
#[derive(Debug, Clone)]
pub struct MultiValueRegisterStrategy;

impl MultiValueRegisterStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiValueRegisterStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictResolutionStrategy for MultiValueRegisterStrategy {
    fn name(&self) -> &'static str {
        "MultiValueRegister"
    }

    fn can_handle(&self, conflict: &ConflictType) -> bool {
        matches!(conflict, ConflictType::UpdateUpdate { .. })
    }

    fn resolve(&self, conflict: &ConflictType) -> Result<ResolvedChange, ConflictResolutionError> {
        match conflict {
            ConflictType::UpdateUpdate {
                record_id,
                site_a,
                site_b,
            } => Ok(ResolvedChange {
                record_id: record_id.clone(),
                value: format!("multi:{}|{}", site_a.as_u32(), site_b.as_u32()),
                strategy: "MultiValueRegister",
            }),
            _ => Err(ConflictResolutionError::StrategyNotApplicable),
        }
    }
}

/// Default conflict resolver
#[derive(Debug, Clone)]
pub struct DefaultConflictResolver;

impl DefaultConflictResolver {
    pub fn new() -> Self {
        Self
    }
}

impl<R: Record> ConflictResolver<R> for DefaultConflictResolver {
    fn resolve(&self, conflict: &ConflictType) -> Result<RecordChange<R>, ConflictResolutionError> {
        // For generic R, we can't construct a full record from just an ID
        // This returns an error which will be handled by the pipeline
        let _conflict_id = match conflict {
            ConflictType::UpdateUpdate { record_id, .. } => record_id.clone(),
            ConflictType::UpdateDelete { record_id, .. } => record_id.clone(),
            ConflictType::InsertInsert { id_a, .. } => id_a.clone(),
            ConflictType::NestedField { record_id, .. } => record_id.clone(),
            ConflictType::Structural { parent_id, .. } => parent_id.clone(),
        };
        Err(ConflictResolutionError::Unresolvable(
            "Cannot resolve for generic Record type".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "DefaultConflictResolver"
    }
}

/// Conflict Resolution Pipeline - DIP: Depends on abstractions
pub struct ConflictResolutionPipeline<R: Record> {
    detectors: Vec<Arc<dyn ConflictDetector<R>>>,
    resolvers: Vec<Arc<dyn ConflictResolver<R>>>,
    strategies: Vec<Arc<dyn ConflictResolutionStrategy>>,
    notifier: Option<Arc<dyn ConflictNotifier>>,
    metrics: Arc<ConflictMetrics>,
}

impl<R: Record> ConflictResolutionPipeline<R> {
    pub fn new(metrics: Arc<ConflictMetrics>) -> Self {
        Self {
            detectors: Vec::new(),
            resolvers: Vec::new(),
            strategies: Vec::new(),
            notifier: None,
            metrics,
        }
    }

    pub fn with_detector(mut self, detector: Arc<dyn ConflictDetector<R>>) -> Self {
        self.detectors.push(detector);
        self
    }

    pub fn with_resolver(mut self, resolver: Arc<dyn ConflictResolver<R>>) -> Self {
        self.resolvers.push(resolver);
        self
    }

    pub fn with_strategy(mut self, strategy: Arc<dyn ConflictResolutionStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn with_notifier(mut self, notifier: Arc<dyn ConflictNotifier>) -> Self {
        self.notifier = Some(notifier);
        self
    }

    /// Process an incoming change and detect/resolve conflicts
    pub fn process_incoming_change(
        &self,
        local: &RecordChange<R>,
        remote: &RecordChange<R>,
    ) -> Result<ProcessedChange, ConflictResolutionError> {
        // Detect conflicts
        for detector in &self.detectors {
            if let Some(conflict) = detector.detect(local, remote) {
                self.metrics.record_conflict(&conflict);

                // Try strategies first
                for strategy in &self.strategies {
                    if strategy.can_handle(&conflict) {
                        return strategy.resolve(&conflict).map(|resolved| ProcessedChange {
                            record_id: resolved.record_id,
                            resolved_value: resolved.value,
                            conflict_type: conflict,
                        });
                    }
                }

                // Fall back to resolvers
                for resolver in &self.resolvers {
                    match resolver.resolve(&conflict) {
                        Ok(_change) => {
                            return Ok(ProcessedChange {
                                record_id: local.id().clone(),
                                resolved_value: "resolved".to_string(),
                                conflict_type: conflict,
                            });
                        }
                        Err(_) => continue,
                    }
                }

                return Err(ConflictResolutionError::Unresolvable(conflict.to_string()));
            }
        }

        // No conflict detected
        Ok(ProcessedChange {
            record_id: local.id().clone(),
            resolved_value: "no_conflict".to_string(),
            conflict_type: ConflictType::UpdateUpdate {
                record_id: local.id().clone(),
                site_a: SiteId::new(),
                site_b: SiteId::new(),
            },
        })
    }

    pub fn get_metrics(&self) -> ConflictReport {
        self.metrics.get_report()
    }
}

/// Result of processing a change
#[derive(Debug, Clone)]
pub struct ProcessedChange {
    pub record_id: RecordId,
    pub resolved_value: String,
    pub conflict_type: ConflictType,
}

/// Mock notifier for testing
#[derive(Debug, Default)]
pub struct MockNotifier;

#[async_trait::async_trait]
impl ConflictNotifier for MockNotifier {
    async fn notify_conflicts_resolved(&self, _applied: &AppliedChange) {}
}

/// Mock resolver for testing
pub struct MockResolver;

impl<R: Record> ConflictResolver<R> for MockResolver {
    fn resolve(
        &self,
        _conflict: &ConflictType,
    ) -> Result<RecordChange<R>, ConflictResolutionError> {
        Err(ConflictResolutionError::Unresolvable(
            "Mock resolver always fails".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "MockResolver"
    }
}

#[cfg(test)]
mod conflict_tests {
    use super::*;
    use archflow_records::{Record, RecordId};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TestRecord {
        pub id: RecordId,
        pub name: String,
        pub value: i32,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }
        fn type_name(&self) -> &'static str {
            "TestRecord"
        }
    }

    #[test]
    fn test_conflict_type_update_update() {
        let id = RecordId::from_str("conflict_update_001").unwrap();
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let conflict = ConflictType::UpdateUpdate {
            record_id: id.clone(),
            site_a,
            site_b,
        };

        match conflict {
            ConflictType::UpdateUpdate { record_id, .. } => {
                assert_eq!(record_id, id);
            }
            _ => panic!("Wrong conflict type"),
        }
    }

    #[test]
    fn test_conflict_metrics_recording() {
        let metrics = Arc::new(ConflictMetrics::default());

        metrics.record_conflict(&ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("metrics_test_01").unwrap(),
            site_a: SiteId::new(),
            site_b: SiteId::new(),
        });

        let report = metrics.get_report();
        assert_eq!(report.total_conflicts, 1);
        assert_eq!(report.update_update, 1);
    }

    #[test]
    fn test_pipeline_process_incoming_change() {
        let metrics = Arc::new(ConflictMetrics::default());
        let pipeline = ConflictResolutionPipeline::<TestRecord>::new(metrics)
            .with_strategy(Arc::new(LastWriterWinsStrategy::new(SiteId::new())));

        let id = RecordId::from_str("pipeline_test_001").unwrap();
        let local = RecordChange::Created {
            id: id.clone(),
            record: TestRecord {
                id: id.clone(),
                name: "local".into(),
                value: 1,
            },
        };
        let remote = RecordChange::Created {
            id: id.clone(),
            record: TestRecord {
                id: id.clone(),
                name: "remote".into(),
                value: 2,
            },
        };

        // Process should succeed
        let result = pipeline.process_incoming_change(&local, &remote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strategy_swap_lsp() {
        // LSP: Strategies must be interchangeable
        let last_wins: Arc<dyn ConflictResolutionStrategy> =
            Arc::new(LastWriterWinsStrategy::new(SiteId::new()));
        let multi_value: Arc<dyn ConflictResolutionStrategy> =
            Arc::new(MultiValueRegisterStrategy::new());

        let conflict = ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("lsp_test_001").unwrap(),
            site_a: SiteId::new(),
            site_b: SiteId::new(),
        };

        assert!(last_wins.can_handle(&conflict));
        assert!(multi_value.can_handle(&conflict));
    }

    #[test]
    fn test_last_writer_wins_strategy() {
        let site_a = SiteId::new();
        let site_b = SiteId::new();
        let strategy = LastWriterWinsStrategy::new(site_a);

        let conflict = ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("lww_test_001").unwrap(),
            site_a,
            site_b,
        };

        let result = strategy.resolve(&conflict);
        assert!(result.is_ok());
    }
}
