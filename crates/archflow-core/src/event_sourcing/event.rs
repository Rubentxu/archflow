//! Domain Events - All event types for the domain
//!
//! All domain events are variants of the DomainEvent enum for
//! type-safe serialization and easier handling.

use crate::EntityId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Wrapper for SystemTime that serializes to/from ISO string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SerializableTime(pub DateTime<Utc>);

impl Default for SerializableTime {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl Serialize for SerializableTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for SerializableTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc3339(&s)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Self(dt.with_timezone(&Utc)))
    }
}

impl std::ops::Deref for SerializableTime {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SerializableTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Metadata attached to every domain event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: EntityId,
    /// Aggregate root ID this event belongs to
    pub aggregate_id: EntityId,
    /// Version of the aggregate after this event
    pub version: u64,
    /// Timestamp when the event was created
    pub timestamp: SerializableTime,
    /// User or system that caused this event
    pub causation_id: EntityId,
    /// Correlation ID for tracking related events
    pub correlation_id: EntityId,
    /// Event type name for serialization
    pub event_type: String,
}

impl EventMetadata {
    /// Create new metadata for an event
    pub fn new(
        aggregate_id: EntityId,
        version: u64,
        causation_id: EntityId,
        event_type: String,
    ) -> Self {
        Self {
            event_id: EntityId::new(),
            aggregate_id,
            version,
            timestamp: SerializableTime::default(),
            causation_id,
            correlation_id: causation_id,
            event_type,
        }
    }

    /// Create metadata for the first event in an aggregate
    pub fn for_new_aggregate(aggregate_id: EntityId, causation_id: EntityId) -> Self {
        Self::new(aggregate_id, 1, causation_id, "Created".to_string())
    }
}

/// All domain events as an enum for type-safe serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "")]
#[serde(tag = "event_type")]
pub enum DomainEvent {
    /// A primitive was created
    #[serde(rename = "PrimitiveCreated")]
    PrimitiveCreated {
        metadata: EventMetadata,
        primitive_id: EntityId,
        primitive_type: String,
        position: (f32, f32),
        size: (f32, f32),
    },
    /// A primitive was deleted
    #[serde(rename = "PrimitiveDeleted")]
    PrimitiveDeleted {
        metadata: EventMetadata,
        primitive_id: EntityId,
    },
    /// A primitive was moved
    #[serde(rename = "PrimitiveMoved")]
    PrimitiveMoved {
        metadata: EventMetadata,
        primitive_id: EntityId,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// A primitive was resized
    #[serde(rename = "PrimitiveResized")]
    PrimitiveResized {
        metadata: EventMetadata,
        primitive_id: EntityId,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// A connection was created
    #[serde(rename = "ConnectionCreated")]
    ConnectionCreated {
        metadata: EventMetadata,
        connection_id: EntityId,
        source_port: EntityId,
        target_port: EntityId,
    },
    /// A connection was deleted
    #[serde(rename = "ConnectionDeleted")]
    ConnectionDeleted {
        metadata: EventMetadata,
        connection_id: EntityId,
    },
    /// Primitives were grouped
    #[serde(rename = "PrimitivesGrouped")]
    PrimitivesGrouped {
        metadata: EventMetadata,
        group_id: EntityId,
        primitive_ids: Vec<EntityId>,
    },
    /// Primitives were ungrouped
    #[serde(rename = "PrimitivesUngrouped")]
    PrimitivesUngrouped {
        metadata: EventMetadata,
        group_id: EntityId,
        primitive_ids: Vec<EntityId>,
    },
    /// A property was changed
    #[serde(rename = "PropertyChanged")]
    PropertyChanged {
        metadata: EventMetadata,
        entity_id: EntityId,
        property_name: String,
        from: String,
        to: String,
    },
}

impl DomainEvent {
    /// Get the event metadata
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            DomainEvent::PrimitiveCreated { metadata, .. } => metadata,
            DomainEvent::PrimitiveDeleted { metadata, .. } => metadata,
            DomainEvent::PrimitiveMoved { metadata, .. } => metadata,
            DomainEvent::PrimitiveResized { metadata, .. } => metadata,
            DomainEvent::ConnectionCreated { metadata, .. } => metadata,
            DomainEvent::ConnectionDeleted { metadata, .. } => metadata,
            DomainEvent::PrimitivesGrouped { metadata, .. } => metadata,
            DomainEvent::PrimitivesUngrouped { metadata, .. } => metadata,
            DomainEvent::PropertyChanged { metadata, .. } => metadata,
        }
    }

    /// Get mutable reference to metadata
    pub fn metadata_mut(&mut self) -> &mut EventMetadata {
        match self {
            DomainEvent::PrimitiveCreated { metadata, .. } => metadata,
            DomainEvent::PrimitiveDeleted { metadata, .. } => metadata,
            DomainEvent::PrimitiveMoved { metadata, .. } => metadata,
            DomainEvent::PrimitiveResized { metadata, .. } => metadata,
            DomainEvent::ConnectionCreated { metadata, .. } => metadata,
            DomainEvent::ConnectionDeleted { metadata, .. } => metadata,
            DomainEvent::PrimitivesGrouped { metadata, .. } => metadata,
            DomainEvent::PrimitivesUngrouped { metadata, .. } => metadata,
            DomainEvent::PropertyChanged { metadata, .. } => metadata,
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> String {
        self.metadata().event_type.clone()
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            DomainEvent::PrimitiveCreated {
                primitive_type,
                position,
                size,
                ..
            } => {
                format!(
                    "Created {} at ({:.1}, {:.1}) with size ({:.1}, {:.1})",
                    primitive_type, position.0, position.1, size.0, size.1
                )
            }
            DomainEvent::PrimitiveDeleted { primitive_id, .. } => {
                format!("Deleted primitive {}", primitive_id)
            }
            DomainEvent::PrimitiveMoved {
                primitive_id,
                from,
                to,
                ..
            } => {
                format!(
                    "Moved primitive from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    from.0, from.1, to.0, to.1
                )
            }
            DomainEvent::PrimitiveResized {
                primitive_id,
                from,
                to,
                ..
            } => {
                format!(
                    "Resized primitive from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    from.0, from.1, to.0, to.1
                )
            }
            DomainEvent::ConnectionCreated { connection_id, .. } => {
                format!("Created connection {}", connection_id)
            }
            DomainEvent::ConnectionDeleted { connection_id, .. } => {
                format!("Deleted connection {}", connection_id)
            }
            DomainEvent::PrimitivesGrouped { primitive_ids, .. } => {
                format!("Grouped {} primitives", primitive_ids.len())
            }
            DomainEvent::PrimitivesUngrouped { primitive_ids, .. } => {
                format!("Ungrouped {} primitives", primitive_ids.len())
            }
            DomainEvent::PropertyChanged {
                entity_id,
                property_name,
                from,
                to,
                ..
            } => {
                format!(
                    "Changed {} of {} from '{}' to '{}'",
                    property_name, entity_id, from, to
                )
            }
        }
    }

    /// Create the inverse event for undo
    pub fn invert(&self) -> DomainEvent {
        match self.clone() {
            DomainEvent::PrimitiveCreated {
                metadata,
                primitive_id,
                primitive_type,
                position,
                size,
            } => DomainEvent::PrimitiveDeleted {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PrimitiveDeleted".to_string(),
                ),
                primitive_id,
            },
            DomainEvent::PrimitiveDeleted {
                metadata,
                primitive_id,
                ..
            } => DomainEvent::PrimitiveDeleted {
                metadata,
                primitive_id,
            },
            DomainEvent::PrimitiveMoved {
                metadata,
                primitive_id,
                from,
                to,
                ..
            } => DomainEvent::PrimitiveMoved {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PrimitiveMoved".to_string(),
                ),
                primitive_id,
                from: to,
                to: from,
            },
            DomainEvent::PrimitiveResized {
                metadata,
                primitive_id,
                from,
                to,
                ..
            } => DomainEvent::PrimitiveResized {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PrimitiveResized".to_string(),
                ),
                primitive_id,
                from: to,
                to: from,
            },
            DomainEvent::ConnectionCreated {
                metadata,
                connection_id,
                source_port,
                target_port,
            } => DomainEvent::ConnectionDeleted {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "ConnectionDeleted".to_string(),
                ),
                connection_id,
            },
            DomainEvent::ConnectionDeleted {
                metadata,
                connection_id,
                ..
            } => DomainEvent::ConnectionDeleted {
                metadata,
                connection_id,
            },
            DomainEvent::PrimitivesGrouped {
                metadata,
                group_id,
                primitive_ids,
            } => DomainEvent::PrimitivesUngrouped {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PrimitivesUngrouped".to_string(),
                ),
                group_id,
                primitive_ids,
            },
            DomainEvent::PrimitivesUngrouped {
                metadata,
                group_id,
                primitive_ids,
            } => DomainEvent::PrimitivesGrouped {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PrimitivesGrouped".to_string(),
                ),
                group_id,
                primitive_ids,
            },
            DomainEvent::PropertyChanged {
                metadata,
                entity_id,
                property_name,
                from,
                to,
                ..
            } => DomainEvent::PropertyChanged {
                metadata: EventMetadata::new(
                    metadata.aggregate_id,
                    metadata.version + 1,
                    metadata.causation_id,
                    "PropertyChanged".to_string(),
                ),
                entity_id,
                property_name,
                from: to,
                to: from,
            },
        }
    }
}
