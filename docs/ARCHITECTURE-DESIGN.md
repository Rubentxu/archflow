# ArchFlow Architecture Design Document

**Version:** 1.0  
**Date:** 2026-01-22  
**Status:** Design Specification  
**Approach:** DDD + EDA + Hexagonal Architecture

---

## 1. Architectural Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Presentation Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │  Leptos/WASM    │  │     CLI (/)     │  │  VS Code Extension  │  │
│  │  (Browser)      │  │   (Rust Bin)    │  │    (Future)         │  │
│  └────────┬────────┘  └────────┬────────┘  └──────────┬──────────┘  │
└───────────┼────────────────────┼───────────────────────┼─────────────┘
            │                    │                       │
            │   gRPC/WebSocket   │      stdio/IPC        │
            │                    │                       │
            ▼                    ▼                       ▼
┌─────────────────────────────────────────────────────────────────────┤
│                      Application Layer (Core)                        │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    Event Bus (Publisher/Subscriber)            │  │
│  └───────────────────────────────────────────────────────────────┘  │
│            │                    │                    │              │
│  ┌─────────┴─────────┐  ┌───────┴───────┐  ┌──────┴──────────┐     │
│  │  Architecture     │  │   Command     │  │   Query          │     │
│  │  Service          │  │   Handler     │  │   Handler        │     │
│  └─────────┬─────────┘  └───────────────┘  └─────────────────┘     │
└────────────┼───────────────────────────────────────────────────────┘
             │
┌────────────▼───────────────────────────────────────────────────────┤
│                        Domain Layer                                  │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐ │
│  │  Architecture  │  │   Component    │  │   AUF Format           │ │
│  │  Aggregate     │  │   Aggregate    │  │   (Parser/Serializer)  │ │
│  └────────────────┘  └────────────────┘  └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
             │
┌────────────▼───────────────────────────────────────────────────────┤
│                    Infrastructure Layer                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐ │
│  │  PostgreSQL    │  │   S3/Local     │  │   Terraform Exporter   │ │
│  │  Repository    │  │   Storage      │  │   Engine               │ │
│  └────────────────┘  └────────────────┘  └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 Crate Structure

```
crates/
├── Cargo.toml                    # Workspace configuration
├── Cargo.lock
│
├── domain/                       # Core domain logic (pure Rust)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── architecture/         # Architecture aggregate
│   │   │   ├── mod.rs
│   │   │   ├── architecture.rs
│   │   │   ├── component.rs
│   │   │   ├── layer.rs
│   │   │   ├── relationship.rs
│   │   │   └── policy.rs
│   │   ├── events/               # Domain events
│   │   │   ├── mod.rs
│   │   │   ├── architecture_events.rs
│   │   │   └── component_events.rs
│   │   ├── value_objects/        # Reusable value objects
│   │   │   ├── mod.rs
│   │   │   ├── component_id.rs
│   │   │   ├── position.rs
│   │   │   └── version.rs
│   │   ├── errors/               # Domain errors
│   │   │   └── mod.rs
│   │   └── lib.rs
│   └── tests/
│
├── application/                  # Use cases and orchestration
│   ├── Cargo.toml
│   ├── src/
│   │   ├── commands/             # Command handlers
│   │   │   ├── mod.rs
│   │   │   ├── create_architecture.rs
│   │   │   ├── add_component.rs
│   │   │   └── export_terraform.rs
│   │   ├── queries/              # Query handlers
│   │   │   ├── mod.rs
│   │   │   ├── get_architecture.rs
│   │   │   └── list_components.rs
│   │   ├── services/             # Application services
│   │   │   ├── mod.rs
│   │   │   ├── architecture_service.rs
│   │   │   └── sync_service.rs
│   │   ├── dto/                  # Data transfer objects
│   │   │   └── mod.rs
│   │   └── lib.rs
│   └── tests/
│
├── infrastructure/               # External adapters
│   ├── Cargo.toml
│   ├── src/
│   │   ├── persistence/          # Database adapters
│   │   │   ├── mod.rs
│   │   │   ├── postgres/
│   │   │   │   ├── mod.rs
│   │   │   │   └── repository.rs
│   │   │   └── memory/
│   │   │       └── mod.rs
│   │   ├── event_bus/            # Event bus implementation
│   │   │   ├── mod.rs
│   │   │   └── in_memory_bus.rs
│   │   ├── export/               # IaC exporters
│   │   │   ├── mod.rs
│   │   │   ├── terraform/
│   │   │   │   ├── mod.rs
│   │   │   │   └── hcl_generator.rs
│   │   │   └── kubernetes/
│   │   │       └── mod.rs
│   │   ├── storage/              # File storage adapters
│   │   │   ├── mod.rs
│   │   │   └── local_fs.rs
│   │   ├──auf/                   # AUF parser/serializer
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   └── serializer.rs
│   │   └── lib.rs
│   └── tests/
│
├── presentation/                 # Frontend and interfaces
│   ├── Cargo.toml
│   ├── src/
│   │   ├── leptos_app/           # Leptos frontend
│   │   │   ├── mod.rs
│   │   │   ├── app.rs
│   │   │   ├── components/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── canvas.rs
│   │   │   │   ├── toolbar.rs
│   │   │   │   └── properties_panel.rs
│   │   │   ├── pages/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── home.rs
│   │   │   │   └── editor.rs
│   │   │   ├── state/            # Frontend state management
│   │   │   │   ├── mod.rs
│   │   │   │   └── store.rs
│   │   │   └── lib.rs
│   │   ├── cli/                   # CLI application
│   │   │   ├── mod.rs
│   │   │   ├── commands/
│   │   │   │   ├── init.rs
│   │   │   │   ├── export.rs
│   │   │   │   └── serve.rs
│   │   │   └── main.rs
│   │   ├── grpc/                  # gRPC server (future)
│   │   │   └── mod.rs
│   │   └── lib.rs
│   └── tests/
│
├── shared/                       # Shared utilities
│   ├── Cargo.toml
│   ├── src/
│   │   ├── config/               # Configuration management
│   │   │   ├── mod.rs
│   │   │   └── settings.rs
│   │   ├── logging/              # Logging utilities
│   │   │   └── mod.rs
│   │   ├── telemetry/            # Tracing and metrics
│   │   │   └── mod.rs
│   │   ├── proto/                # Protobuf definitions
│   │   │   └── mod.rs
│   │   ├── async_utils/          # Async utilities
│   │   │   └── mod.rs
│   │   └── lib.rs
│   └── tests/
│
└── Cargo.toml                    # Workspace root
```

---

## 2. Domain Layer (DDD)

### 2.1 Core Aggregates

#### 2.1.1 Architecture Aggregate

```rust
// crates/domain/src/architecture/architecture.rs

use crate::events::{ArchitectureEvent, EventEnvelope};
use crate::value_objects::{ArchitectureId, Version};
use crate::architecture::{Component, Layer, Relationship, Policy};
use crate::errors::DomainError;
use std::collections::HashMap;

/// Architecture is the root aggregate for the entire system.
/// All modifications flow through this aggregate.
#[derive(Debug, Clone)]
pub struct Architecture {
    id: ArchitectureId,
    name: String,
    description: String,
    version: Version,
    metadata: ArchitectureMetadata,
    layers: Vec<Layer>,
    components: HashMap<ComponentId, Component>,
    relationships: Vec<Relationship>,
    policies: Vec<Policy>,
    state: ArchitectureState,
}

impl Architecture {
    /// Creates a new architecture with given name
    pub fn new(name: String, description: String) -> Result<Self, DomainError> {
        Ok(Architecture {
            id: ArchitectureId::new(),
            name,
            description,
            version: Version::v1(),
            metadata: ArchitectureMetadata::default(),
            layers: Vec::new(),
            components: HashMap::new(),
            relationships: Vec::new(),
            policies: Vec::new(),
            state: ArchitectureState::Draft,
        })
    }

    /// Adds a component to the architecture
    pub fn add_component(&mut self, component: Component) -> Result<(), DomainError> {
        if self.components.contains_key(&component.id()) {
            return Err(DomainError::ComponentAlreadyExists(component.id()));
        }
        
        self.components.insert(component.id(), component.clone());
        
        Ok(())
    }

    /// Removes a component and its relationships
    pub fn remove_component(&mut self, id: &ComponentId) -> Result<(), DomainError> {
        let component = self.components.remove(id)
            .ok_or(DomainError::ComponentNotFound(*id))?;
        
        // Remove all relationships involving this component
        self.relationships.retain(|r| !r.involves(id));
        
        Ok(())
    }

    /// Updates component properties
    pub fn update_component(
        &mut self, 
        id: &ComponentId, 
        properties: HashMap<String, PropertyValue>
    ) -> Result<(), DomainError> {
        let component = self.components.get_mut(id)
            .ok_or(DomainError::ComponentNotFound(*id))?;
        
        component.update_properties(properties);
        
        Ok(())
    }

    /// Bumps version and marks as changed
    pub fn version_bump(&mut self) {
        self.version = self.version.bump_minor();
    }

    /// Returns all domain events generated by operations
    pub fn take_events(&mut self) -> Vec<ArchitectureEvent> {
        std::mem::take(&mut self.uncommitted_events)
    }
    
    // Getters
    pub fn id(&self) -> &ArchitectureId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn version(&self) -> &Version { &self.version }
    pub fn components(&self) -> impl Iterator<Item = &Component> { self.components.values() }
    pub fn relationships(&self) -> &[Relationship] { &self.relationships }
    pub fn state(&self) -> ArchitectureState { self.state }
}
```

#### 2.1.2 Component Aggregate

```rust
// crates/domain/src/architecture/component.rs

use crate::value_objects::{ComponentId, Position, ComponentType, CloudProvider};
use crate::errors::DomainError;
use std::collections::HashMap;

/// Component represents an atomic architectural element
#[derive(Debug, Clone)]
pub struct Component {
    id: ComponentId,
    name: String,
    component_type: ComponentType,
    cloud_provider: Option<CloudProvider>,
    position: Position,
    properties: ComponentProperties,
    constraints: Vec<Constraint>,
    iac_mappings: HashMap<IacType, String>,
}

impl Component {
    pub fn new(
        name: String,
        component_type: ComponentType,
        position: Position,
    ) -> Self {
        Component {
            id: ComponentId::new(),
            name,
            component_type,
            cloud_provider: None,
            position,
            properties: ComponentProperties::new(),
            constraints: Vec::new(),
            iac_mappings: HashMap::new(),
        }
    }

    pub fn with_cloud_provider(mut self, provider: CloudProvider) -> Self {
        self.cloud_provider = Some(provider);
        self
    }

    pub fn update_properties(&mut self, properties: HashMap<String, PropertyValue>) {
        for (key, value) in properties {
            self.properties.0.insert(key, value);
        }
    }

    pub fn add_iac_mapping(&mut self, iac_type: IacType, resource_id: String) {
        self.iac_mappings.insert(iac_type, resource_id);
    }

    // Getters
    pub fn id(&self) -> &ComponentId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn component_type(&self) -> &ComponentType { &self.component_type }
    pub fn position(&self) -> &Position { &self.position }
    pub fn properties(&self) -> &ComponentProperties { &self.properties }
}

/// Component type hierarchy for extensibility
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentType {
    // Compute
    Ec2Instance,
    LambdaFunction,
    Container,
    
    // Storage
    S3Bucket,
    RdsInstance,
    DynamoTable,
    
    // Network
    Vpc,
    LoadBalancer,
    CloudFrontDistribution,
    
    // Security
    IamRole,
    Waf,
    
    // Custom (user-defined)
    Custom(String),
}

impl ComponentType {
    pub fn category(&self) -> ComponentCategory {
        match self {
            ComponentType::Ec2Instance | ComponentType::LambdaFunction | ComponentType::Container 
                => ComponentCategory::Compute,
            ComponentType::S3Bucket | ComponentType::RdsInstance | ComponentType::DynamoTable 
                => ComponentCategory::Storage,
            ComponentType::Vpc | ComponentType::LoadBalancer | ComponentType::CloudFrontDistribution 
                => ComponentCategory::Network,
            ComponentType::IamRole | ComponentType::Waf 
                => ComponentCategory::Security,
            ComponentType::Custom(_) => ComponentCategory::Custom,
        }
    }
}
```

### 2.2 Domain Events (EDA Foundation)

```rust
// crates/domain/src/events/mod.rs

use crate::value_objects::{ArchitectureId, ComponentId};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

/// Domain events are the foundation of EDA
/// All state changes in the domain emit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchitectureEvent {
    // Architecture lifecycle
    ArchitectureCreated {
        architecture_id: ArchitectureId,
        name: String,
        timestamp: SystemTime,
    },
    ArchitectureRenamed {
        architecture_id: ArchitectureId,
        old_name: String,
        new_name: String,
        timestamp: SystemTime,
    },
    ArchitecturePublished {
        architecture_id: ArchitectureId,
        version: String,
        timestamp: SystemTime,
    },
    
    // Component events
    ComponentAdded {
        architecture_id: ArchitectureId,
        component_id: ComponentId,
        component_type: String,
        timestamp: SystemTime,
    },
    ComponentRemoved {
        architecture_id: ArchitectureId,
        component_id: ComponentId,
        timestamp: SystemTime,
    },
    ComponentUpdated {
        architecture_id: ArchitectureId,
        component_id: ComponentId,
        changed_properties: Vec<String>,
        timestamp: SystemTime,
    },
    ComponentMoved {
        architecture_id: ArchitectureId,
        component_id: ComponentId,
        old_position: (f64, f64),
        new_position: (f64, f64),
        timestamp: SystemTime,
    },
    
    // Relationship events
    RelationshipCreated {
        architecture_id: ArchitectureId,
        from: ComponentId,
        to: ComponentId,
        relationship_type: String,
        timestamp: SystemTime,
    },
    RelationshipRemoved {
        architecture_id: ArchitectureId,
        from: ComponentId,
        to: ComponentId,
        timestamp: SystemTime,
    },
    
    // Export events
    ExportRequested {
        architecture_id: ArchitectureId,
        target_format: String,
        timestamp: SystemTime,
    },
    ExportCompleted {
        architecture_id: ArchitectureId,
        target_format: String,
        output_path: String,
        timestamp: SystemTime,
    },
}

/// Wrapper for events with metadata
#[derive(Debug, Clone)]
pub struct EventEnvelope<T> {
    pub event: T,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: SystemTime,
}
```

### 2.3 Value Objects

```rust
// crates/domain/src/value_objects/mod.rs

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Unique identifier for Architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArchitectureId(pub Uuid);

impl ArchitectureId {
    pub fn new() -> Self {
        ArchitectureId(Uuid::new_v4())
    }
}

impl Default for ArchitectureId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ArchitectureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "arch-{}", self.0.simple())
    }
}

/// Unique identifier for Component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub Uuid);

impl ComponentId {
    pub fn new() -> Self {
        ComponentId(Uuid::new_v4())
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "comp-{}", self.0.simple())
    }
}

/// Position on the canvas
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64, // Layer z-index
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }
    
    pub fn with_z(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Version tracking for architectures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: Option<String>,
    build: Option<String>,
}

impl Version {
    pub fn v1() -> Self {
        Version {
            major: 1,
            minor: 0,
            patch: 0,
            prerelease: None,
            build: None,
        }
    }
    
    pub fn bump_major(mut self) -> Self {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self
    }
    
    pub fn bump_minor(mut self) -> Self {
        self.minor += 1;
        self.patch = 0;
        self
    }
    
    pub fn bump_patch(mut self) -> Self {
        self.patch += 1;
        self
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(prerelease) = &self.prerelease {
            write!(f, "-{}", prerelease)?;
        }
        if let Some(build) = &self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}
```

---

## 3. Application Layer (Use Cases)

### 3.1 Command Handlers

```rust
// crates/application/src/commands/create_architecture.rs

use crate::dto::{CreateArchitectureDto, ArchitectureDto};
use domain::architecture::Architecture;
use domain::events::ArchitectureEvent;
use domain::errors::DomainError;
use infrastructure::persistence::ArchitectureRepository;
use infrastructure::event_bus::EventBus;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateArchitectureError {
    #[error("Failed to create architecture: {0}")]
    CreationFailed(#[from] DomainError),
    #[error("Failed to persist: {0}")]
    PersistenceFailed(String),
    #[error("Failed to publish event: {0}")]
    EventPublishFailed(String),
}

pub struct CreateArchitectureCommand {
    repository: Box<dyn ArchitectureRepository>,
    event_bus: Box<dyn EventBus>,
}

impl CreateArchitectureCommand {
    pub fn new(
        repository: Box<dyn ArchitectureRepository>,
        event_bus: Box<dyn EventBus>,
    ) -> Self {
        Self { repository, event_bus }
    }
    
    pub async fn execute(&self, dto: CreateArchitectureDto) -> Result<ArchitectureDto, CreateArchitectureError> {
        // 1. Create domain object
        let mut architecture = Architecture::new(
            dto.name.clone(),
            dto.description.unwrap_or_default(),
        )?;
        
        // 2. Persist
        self.repository.save(&architecture)
            .await
            .map_err(|e| CreateArchitectureError::PersistenceFailed(e.to_string()))?;
        
        // 3. Publish event
        let event = ArchitectureEvent::ArchitectureCreated {
            architecture_id: *architecture.id(),
            name: architecture.name().to_string(),
            timestamp: std::time::SystemTime::now(),
        };
        
        self.event_bus.publish(event)
            .await
            .map_err(|e| CreateArchitectureError::EventPublishFailed(e.to_string()))?;
        
        // 4. Return DTO
        Ok(ArchitectureDto::from_domain(&architecture))
    }
}
```

### 3.2 Query Handlers

```rust
// crates/application/src/queries/get_architecture.rs

use crate::dto::{ArchitectureDto, ComponentDto};
use domain::architecture::Architecture;
use infrastructure::persistence::ArchitectureRepository;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetArchitectureError {
    #[error("Architecture not found: {0}")]
    NotFound(String),
    #[error("Repository error: {0}")]
    RepositoryError(String),
}

pub struct GetArchitectureQuery {
    repository: Box<dyn ArchitectureRepository>,
}

impl GetArchitectureQuery {
    pub fn new(repository: Box<dyn ArchitectureRepository>) -> Self {
        Self { repository }
    }
    
    pub async fn execute(&self, id: &str) -> Result<ArchitectureDto, GetArchitectureError> {
        self.repository
            .find_by_id(id)
            .await
            .map(|opt| {
                opt.map(ArchitectureDto::from_domain)
                    .ok_or_else(|| GetArchitectureError::NotFound(id.to_string()))
            })
            .map_err(|e| GetArchitectureError::RepositoryError(e.to_string()))?
    }
    
    pub async fn execute_with_components(&self, id: &str) 
        -> Result<(ArchitectureDto, Vec<ComponentDto>), GetArchitectureError> 
    {
        let architecture = self.repository
            .find_by_id(id)
            .await
            .map_err(|e| GetArchitectureError::RepositoryError(e.to_string()))?
            .ok_or_else(|| GetArchitectureError::NotFound(id.to_string()))?;
            
        let components: Vec<ComponentDto> = architecture
            .components()
            .map(ComponentDto::from_domain)
            .collect();
            
        Ok((ArchitectureDto::from_domain(&architecture), components))
    }
}
```

### 3.3 DTOs

```rust
// crates/application/src/dto/mod.rs

use domain::architecture::{Architecture, Component};
use domain::value_objects::{ArchitectureId, ComponentId, Position, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateArchitectureDto {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchitectureDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub state: String,
    pub component_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl ArchitectureDto {
    pub fn from_domain(architecture: &Architecture) -> Self {
        Self {
            id: architecture.id().to_string(),
            name: architecture.name().to_string(),
            description: architecture.description().to_string(),
            version: architecture.version().to_string(),
            state: format!("{:?}", architecture.state()),
            component_count: architecture.components().count(),
            created_at: /* timestamp */ String::new(),
            updated_at: /* timestamp */ String::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentDto {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub cloud_provider: Option<String>,
    pub position: PositionDto,
    pub properties: HashMap<String, PropertyValueDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionDto {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropertyValueDto {
    pub value: serde_json::Value,
    pub type_hint: Option<String>,
}
```

---

## 4. Infrastructure Layer

### 4.1 Event Bus (In-Memory for MVP)

```rust
// crates/infrastructure/src/event_bus/in_memory_bus.rs

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use domain::events::ArchitectureEvent;
use crate::event_bus::{EventBus, EventHandler, SubscriptionId};

pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<SubscriptionId, Vec<Box<dyn EventHandler>>>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: ArchitectureEvent) -> Result<(), String> {
        let handlers = self.handlers.read().unwrap();
        
        for handler in handlers.values().flatten() {
            if handler.event_type() == std::any::type_name_of_val(&event) {
                handler.handle(&event).await;
            }
        }
        
        Ok(())
    }
    
    async fn subscribe(
        &self, 
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, String> {
        let id = SubscriptionId::new();
        let mut handlers = self.handlers.write().unwrap();
        
        handlers.entry(id).or_insert_with(Vec::new).push(handler);
        
        Ok(id)
    }
    
    async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), String> {
        let mut handlers = self.handlers.write().unwrap();
        handlers.remove(&id);
        Ok(())
    }
}

/// Trait for event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn event_type(&self) -> &'static str;
    async fn handle(&self, event: &ArchitectureEvent);
}

/// Event handler for logging
pub struct LoggingEventHandler;

#[async_trait]
impl EventHandler for LoggingEventHandler {
    fn event_type(&self) -> &'static str {
        std::any::type_name::<ArchitectureEvent>()
    }
    
    async fn handle(&self, event: &ArchitectureEvent) {
        tracing::info!("Event: {:?}", event);
    }
}
```

### 4.2 Terraform Exporter

```rust
// crates/infrastructure/src/export/terraform/hcl_generator.rs

use domain::architecture::{Architecture, Component, ComponentType};
use std::collections::HashMap;

pub struct HclGenerator;

impl HclGenerator {
    pub fn generate(architecture: &Architecture) -> String {
        let mut output = String::new();
        
        output.push_str("# Generated by ArchFlow\n");
        output.push_str("# DO NOT EDIT MANUALLY\n\n");
        
        output.push_str("terraform {\n");
        output.push_str("  required_providers {\n");
        output.push_str("    aws = {\n");
        output.push_str("      source  = \"hashicorp/aws\"\n");
        output.push_str("      version = \"~> 5.0\"\n");
        output.push_str("    }\n");
        output.push_str("  }\n");
        output.push_str("}\n\n");
        
        output.push_str("provider \"aws\" {\n");
        output.push_str("  region = var.aws_region\n");
        output.push_str("}\n\n");
        
        // Generate variables
        output.push_str("variable \"aws_region\" {\n");
        output.push_str("  description = \"AWS region\"\n");
        output.push_str("  type        = string\n");
        output.push_str("  default     = \"us-east-1\"\n");
        output.push_str("}\n\n");
        
        // Generate resources
        for component in architecture.components() {
            output.push_str(&Self::generate_resource(component));
            output.push('\n');
        }
        
        output
    }
    
    fn generate_resource(component: &Component) -> String {
        match component.component_type() {
            ComponentType::Ec2Instance => Self::generate_ec2(component),
            ComponentType::S3Bucket => Self::generate_s3(component),
            ComponentType::LambdaFunction => Self::generate_lambda(component),
            ComponentType::Vpc => Self::generate_vpc(component),
            _ => Self::generate_generic(component),
        }
    }
    
    fn generate_ec2(component: &Component) -> String {
        let name = component.name();
        let instance_type = component.properties()
            .get("instance_type")
            .map(|v| v.as_str().unwrap_or("t3.micro"))
            .unwrap_or("t3.micro");
            
        format!(
            r#"resource "aws_instance" "{}" {{
  ami           = "ami-0c55b159cbfafe1f0"
  instance_type = "{}"
  
  tags = {{
    Name        = "{}"
    ManagedBy   = "ArchFlow"
    Architecture = "arch-{}"
  }}
}}"#,
            Self::sanitize_name(name),
            instance_type,
            name,
            "TODO"
        )
    }
    
    fn generate_s3(component: &Component) -> String {
        let name = component.name();
        
        format!(
            r#"resource "aws_s3_bucket" "{}" {{
  bucket = "{}"
  
  tags = {{
    Name        = "{}"
    ManagedBy   = "ArchFlow"
  }}
}}"#,
            Self::sanitize_name(name),
            name,
            name
        )
    }
    
    fn generate_lambda(component: &Component) -> String {
        let name = component.name();
        
        format!(
            r#"resource "aws_lambda_function" "{}" {{
  function_name = "{}"
  runtime        = "python3.9"
  handler        = "index.handler"
  
  source_code_hash = data.external.archive_prepared.result.hash
  filename         = "function.zip"
  
  role = aws_iam_role.lambda_exec.arn
  
  tags = {{
    Name        = "{}"
    ManagedBy   = "ArchFlow"
  }}
}}"#,
            Self::sanitize_name(name),
            name,
            name
        )
    }
    
    fn generate_vpc(component: &Component) -> String {
        let name = component.name();
        let cidr = component.properties()
            .get("cidr_block")
            .map(|v| v.as_str().unwrap_or("10.0.0.0/16"))
            .unwrap_or("10.0.0.0/16");
            
        format!(
            r#"resource "aws_vpc" "{}" {{
  cidr_block = "{}"
  
  tags = {{
    Name        = "{}"
    ManagedBy   = "ArchFlow"
  }}
}}"#,
            Self::sanitize_name(name),
            cidr,
            name
        )
    }
    
    fn generate_generic(component: &Component) -> String {
        let name = component.name();
        let component_type = format!("{:?}", component.component_type());
        
        format!(
            r#"# Resource for {} ({})
resource "null_resource" "{}" {{
  triggers = {{
    name = "{}"
    type = "{}"
  }}
}}"#,
            name, component_type,
            Self::sanitize_name(name),
            name,
            component_type
        )
    }
    
    fn sanitize_name(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }
}
```

---

## 5. Presentation Layer (Leptos/WASM)

### 5.1 Leptos App Structure

```rust
// crates/presentation/src/leptos_app/app.rs

use leptos::*;
use crate::leptos_app::components::{Canvas, Toolbar, PropertiesPanel};
use crate::leptos_app::state::AppStore;

#[component]
pub fn App() -> impl IntoView {
    // Initialize global state
    let store = AppStore::new();
    
    // Provide store to components
    provide_context(store);
    
    view! {
        <div class="archflow-app">
            <Toolbar />
            <div class="main-content">
                <Canvas />
                <PropertiesPanel />
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    mount_to_body(App)
}
```

### 5.2 Canvas Component with Rust/WASM

```rust
// crates/presentation/src/leptos_app/components/canvas.rs

use leptos::*;
use crate::leptos_app::state::AppStore;
use crate::leptos_app::components::canvas_renderer::CanvasRenderer;

#[component]
pub fn Canvas() -> impl IntoView {
    let store = expect_context::<AppStore>();
    let canvas_ref = create_signal(NodeRef::default());
    
    // Initialize WebGPU/WebGL renderer
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let renderer = CanvasRenderer::new(canvas);
            store.set_renderer(Some(renderer));
        }
    });
    
    // Handle mouse events for drag-drop
    let on_mousedown = move |event: web_sys::MouseEvent| {
        // Calculate position and select component
    };
    
    let on_mousemove = move |event: web_sys::MouseEvent| {
        // Pan/zoom or drag component
    };
    
    let on_mouseup = move |event: web_sys::MouseEvent| {
        // Commit drag operation
    };
    
    view! {
        <div class="canvas-container">
            <canvas
                ref=canvas_ref
                on:mousedown=on_mousedown
                on:mousemove=on_mousemove
                on:mouseup=on_mouseup
            />
        </div>
    }
}
```

### 5.3 Frontend State Management

```rust
// crates/presentation/src/leptos_app/state/store.rs

use leptos::*;
use std::sync::Arc;
use crate::leptos_app::components::canvas_renderer::CanvasRenderer;

#[derive(Clone)]
pub struct AppStore {
    // Architecture state
    pub architecture_id: RwSignal<Option<String>>,
    pub architecture_name: RwSignal<String>,
    pub components: RwSignal<Vec<ComponentState>>,
    pub selected_component_id: RwSignal<Option<String>>,
    
    // Canvas state
    pub zoom: RwSignal<f32>,
    pub pan_offset: RwSignal<(f32, f32)>,
    
    // Renderer (WebGPU/WebGL)
    pub renderer: RwSignal<Option<Arc<CanvasRenderer>>>,
    
    // UI state
    pub is_loading: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            architecture_id: RwSignal::new(None),
            architecture_name: RwSignal::new("Untitled Architecture".to_string()),
            components: RwSignal::new(Vec::new()),
            selected_component_id: RwSignal::new(None),
            zoom: RwSignal::new(1.0),
            pan_offset: RwSignal::new((0.0, 0.0)),
            renderer: RwSignal::new(None),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
        }
    }
    
    pub async fn load_architecture(&self, id: &str) {
        self.is_loading.set(true);
        
        // Call API to load architecture
        // Update signals on response
    }
    
    pub fn select_component(&self, id: Option<String>) {
        self.selected_component_id.set(id);
        // Notify renderer to highlight selection
    }
    
    pub fn add_component(&self, component_type: &str, position: (f64, f64)) {
        // Create component and add to state
        // Trigger renderer update
    }
}

#[derive(Clone, Debug)]
pub struct ComponentState {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub position: (f64, f64),
    pub properties: std::collections::HashMap<String, String>,
}
```

---

## 6. Shared Utilities

### 6.1 Configuration

```rust
// crates/shared/src/config/mod.rs

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub storage: StorageSettings,
    pub telemetry: TelemetrySettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub environment: Environment,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub pool_size: u32,
    pub ssl_mode: SslMode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    pub local_path: PathBuf,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetrySettings {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::default())
            .build()?;
            
        config.try_deserialize()
    }
}
```

---

## 7. Workspace Configuration

```toml
# crates/Cargo.toml

[workspace]
members = [
    "domain",
    "application", 
    "infrastructure",
    "presentation",
    "shared",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
# Core dependencies
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
thiserror = "2.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Configuration
config = "0.14"

# Tracing
tracing = "0.1"
tracing-subscriber = "0.3"

# Event Bus
futures = "0.3"

# Testing
rstest = "0.18"
proptest = "1.4"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1

[patch.crates-io]
# Patch dependencies if needed
```

### 7.1 Domain Crate

```toml
# crates/domain/Cargo.toml

[package]
name = "archflow-domain"
version.workspace = true
edition.workspace = true

[dependencies]
# Core
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
derive_more = "1.0"

# Testing
rstest.workspace = true
proptest.workspace = true
```

### 7.2 Application Crate

```toml
# crates/application/Cargo.toml

[package]
name = "archflow-application"
version.workspace = true
edition.workspace = true

[dependencies]
archflow-domain = { path = "../domain" }
archflow-infrastructure = { path = "../infrastructure" }
async-trait.workspace = true
thiserror.workspace = true
tokio.workspace = true
serde.workspace = true
futures.workspace = true

[dev-dependencies]
# Add test utilities
```

### 7.3 Infrastructure Crate

```toml
# crates/infrastructure/Cargo.toml

[package]
name = "archflow-infrastructure"
version.workspace = true
edition.workspace = true

[dependencies]
archflow-domain = { path = "../domain" }
archflow-shared = { path = "../shared" }
async-trait.workspace = true
thiserror.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }

# Storage
aws-config = "1.1"
aws-sdk-s3 = "1.1"

[dev-dependencies]
tempfile = "4.0"
```

### 7.4 Presentation Crate (Leptos)

```toml
# crates/presentation/Cargo.toml

[package]
name = "archflow-presentation"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = ["hydrate"]
ssr = ["leptos_use"]

[dependencies]
# Leptos
leptos = "0.6"
leptos_use = { version = "0.10", optional = true }

# WASM
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "HtmlCanvasElement",
    "WebGl2RenderingContext",
] }

# State
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true

# Styling
tailwindcss = "0.1"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O4"]
```

### 7.5 Shared Crate

```toml
# crates/shared/Cargo.toml

[package]
name = "archflow-shared"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
config.workspace = true
```

---

## 8. Event-Driven Architecture Patterns

### 8.1 Event Sourcing (Future Consideration)

For future phases, implement event sourcing:

```rust
// Event store interface
#[async_trait]
pub trait EventStore {
    async fn append(&self, events: &[EventEnvelope<ArchitectureEvent>]) -> Result<(), Error>;
    async fn load(&self, aggregate_id: &ArchitectureId) -> Result<Vec<ArchitectureEvent>, Error>;
    async fn stream(&self, from: u64) -> impl Stream<Item = ArchitectureEvent>;
}
```

### 8.2 CQRS Pattern

Separate command and query responsibilities:

```
Commands (Write):
- CreateArchitecture
- AddComponent  
- UpdateComponent
- ExportToTerraform
- PublishArchitecture

Queries (Read):
- GetArchitecture
- ListArchitectures
- GetComponentDetails
- SearchComponents
- GetArchitectureDiff
```

---

## 9. Development Roadmap

### Phase 1: Foundation (Sprint 1-4)
1. Set up workspace and CI/CD
2. Implement domain aggregates (Architecture, Component)
3. Implement AUF parser/serializer
4. Create basic Leptos frontend with canvas

### Phase 2: Core Features (Sprint 5-8)
1. Command handlers for CRUD operations
2. In-memory repository
3. Terraform HCL generator
4. Basic collaboration (local)

### Phase 3: Polish (Sprint 9-12)
1. PostgreSQL persistence
2. Real-time collaboration (WebSocket)
3. Advanced canvas interactions
4. Cost estimation (MVP)

---

## 10. Conclusion

This architecture provides:

1. **Clear separation** via DDD layers (Domain, Application, Infrastructure, Presentation)
2. **Extensibility** through plugin system and trait-based interfaces
3. **Performance** via Rust/WASM for rendering and Leptos for reactive UI
4. **Maintainability** via strict typing and event-driven design
5. **Testability** via dependency injection and interface segregation

The design follows the Hexagonal Architecture pattern, allowing any component to be replaced without affecting others. The EDA foundation positions the system for future features like event sourcing and CQRS.

---

*Document generated as part of ArchFlow architecture design process.*
