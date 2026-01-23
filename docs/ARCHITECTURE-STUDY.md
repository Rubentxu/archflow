# ArchFlow: Estudio de Arquitectura

**Versión:** 1.0  
**Fecha:** 2026-01-22  
**Estado:** Borrador para Revisión  
**Base:** Análisis PRD + PRD-CRITICA.md

---

## 1. Introducción

### 1.1 Propósito del Documento

Este estudio de arquitectura define la estructura técnica de ArchFlow, una plataforma de documentación y generación de arquitecturas cloud-native. El documento establece las decisiones arquitectónicas fundamentadas en los requisitos del Product Requirements Document (PRD) y el análisis crítico realizado en `docs/PRD-CRITICA.md`.

### 1.2 Alcance

El estudio abarca:
- Arquitectura multicrate con separación por dominio
- Implementación de Domain-Driven Design (DDD)
- Arquitectura orientada a eventos (EDA)
- Configuración centralizada en workspace
- Frontend con Leptos y WebAssembly
- Persistencia y exportadores de Infrastructure as Code

### 1.3 Referencias

| Documento | Descripción |
|-----------|-------------|
| `docs/prd.md` | Product Requirements Document original |
| `docs/PRD-CRITICA.md` | Análisis crítico del PRD con propuestas de mejora |

---

## 2. Análisis de Requisitos

### 2.1 Resumen de Requisitos Funcionales

El PRD establece las siguientes capacidades principales:

| Requisito | Prioridad Original | Prioridad Ajustada |
|-----------|-------------------|-------------------|
| Editor visual con renderizado de diagramas | MVP | MVP |
| Formato universal de arquitectura (AUF) | MVP | MVP |
| Exportación a Terraform | MVP | MVP |
| Exportación a Kubernetes | MVP | Phase 2 |
| Sistema de componentes (AWS, Azure, GCP) | MVP | MVP (AWS solo) |
| Simulación de costos | MVP | Phase 2 |
| Colaboración en tiempo real | Phase 2 | Phase 2 |
| Integración Git | Phase 2 | Phase 2 |
| Asistente AI | Phase 3 | Phase 5 (eliminado) |

### 2.2 Análisis de Connascence Identificado

Del análisis en PRD-CRITICA.md, se identificaron los siguientes problemas de acoplamiento:

**Connascence de Significado (Alta Prioridad):**
- Términos ambiguos: "Component", "Layer", "Sync", "Policy"
- Resolución: Glosario estricto con tipos específicos en Rust

**Connascence de Posición (Media Prioridad):**
- Orden de propiedades en AUF
- Resolución: Uso de claves explícitas, no arrays ordenados

**Connascence de Tipo (Alta Prioridad):**
- Propiedades de componentes sin tipado
- Resolución: Esquemas JSON Schema + tipos Rust específicos

**Connascence de Ejecución (Media Prioridad):**
- Pipeline de exportación acoplado
- Resolución: Etapas aisladas con contratos explícitos

### 2.3 Cambios Recomendados al Alcance

| Cambio | Razón | Impacto |
|--------|-------|---------|
| Reducir MVP a solo Terraform | Complejidad de exportadores | Reducción de scope 50% |
| Eliminar AI del roadmap inmediato | Dependencias externas, responsabilidad legal | Eliminación de riesgo alto |
| Enfocar en AWS solo inicialmente | Recursos limitados de componentes | Reducción de testing |
| Postergar sync bidireccional | Imposibilidad técnica demostrada | Solo importación unidireccional |

---

## 3. Decisiones Arquitectónicas (ADR)

### ADR-001: Arquitectura Multicrate con DDD

**Estado:** Aceptado  
**Contexto:** El sistema requiere separación clara de responsabilidades para permitir evolución independiente de componentes.  
**Decisión:** Implementar arquitectura de workspace Rust con crates separados por capas DDD.  
**Consecuencias:**
- Positiva: Separación clara de concerns
- Positiva: Compilación incremental
- Positiva: Testing aislado por dominio
- Negativa: Sobrecarga de configuración inicial
- Negativa: Complejidad en dependencias cruzadas

### ADR-002: Event-Driven Architecture para Colaboración

**Estado:** Aceptado  
**Contexto:** El sistema requiere comunicación asíncrona entre componentes para futuras funcionalidades de colaboración en tiempo real.  
**Decisión:** Implementar bus de eventos en memoria como foundation, permitiendo extensión a Message Queue.  
**Consecuencias:**
- Positiva: Desacoplamiento de productores/consumidores
- Positiva: Trazabilidad de cambios
- Positiva: Extensibilidad para audit logging
- Negativa: Complejidad en debugging
- Negativa: Posible duplicación de eventos

### ADR-003: Leptos + WebAssembly + wgpu para Frontend

**Estado:** Aceptado  
**Contexto:** El PRD especifica rendimiento de rendering de 10k nodos a 60fps.  
**Decisión:** Usar Leptos (framework React-like en Rust) compilado a WASM con **wgpu** como motor de renderizado (WebGL2 por defecto, WebGPU cuando esté disponible).  
**Consecuencias:**
- Positiva: Rendimiento nativo Rust con GPU acceleration
- Positiva: Instanced rendering para 10k+ componentes a 60fps
- Positiva: Bundle size ~2.8 MB (aceptable para web app)
- Positiva: Futuro WebGPU sin migración de código
- Positiva: Type-safety extremo a través de toda la pila
- Positiva: SSR opcional para SEO/performance
- Negativa: Curva de aprendizaje para desarrolladores web
- Negativa: Tamaño de bundle WASM mayor que soluciones JS puras

### ADR-004: Formato AUF como Source of Truth

**Estado:** Aceptado  
**Contexto:** El diagrama debe ser single source of truth.  
**Decisión:** Architecture Universal Format (AUF) en YAML como formato canónico de almacenamiento.  
**Consecuencias:**
- Positional: Formato legible por humanos
- Positiva: Versionable en Git
- Positiva: Herramientas estándar para diff
- Negativa: Rendimiento de parsing vs. binario
- Negativa: Validación requerirá schema externo

### ADR-005: Hexagonal Architecture (Puertos y Adaptadores)

**Estado:** Aceptado  
**Contexto:** El sistema debe permitir intercambio de implementaciones de infraestructura (DB, exportadores).  
**Decisión:** Implementar puertos (traits) en domain/application e adaptadores en infrastructure.  
**Consecuencias:**
- Positiva: Testabilidad sin dependencias externas
- Positiva: Intercambio de implementaciones
- Positiva: Aislar cambios de infraestructura
- Negativa: Sobrecarga de indirección
- Negativa: Complejidad en mapeo de errores

---

## 4. Vista de Arquitectura

### 4.1 Diagrama de Capas

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            PRESENTATION LAYER                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────────────────────┐│
│  │  Leptos/WASM    │  │     CLI App     │  │  gRPC Server (futuro)          ││
│  │  (Navegador)    │  │   (Rust Bin)    │  │  (Comunicación externa)        ││
│  └────────┬────────┘  └────────┬────────┘  └───────────────┬────────────────┘│
└───────────┼────────────────────┼───────────────────────────┼──────────────────┘
            │                    │                           │
            │    HTTP/WebSocket  │        gRPC/stdio         │   gRPC
            │                    │                           │
            ▼                    ▼                           ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          APPLICATION LAYER                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                      Event Bus (Mediador Central)                    │   │
│  │                    ┌─────────────────────────┐                       │   │
│  │                    │  Publish/Subscribe      │                       │   │
│  │                    │  Correlation IDs        │                       │   │
│  │                    └─────────────────────────┘                       │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│            │                        │                        │                │
│  ┌─────────┴─────────┐    ┌────────┴────────┐    ┌───────┴────────┐       │
│  │  Command Handlers │    │ Query Handlers  │    │  Services      │       │
│  │  · CreateArch     │    │ · GetArch       │    │  · Sync        │       │
│  │  · AddComponent   │    │ · ListArchs     │    │  · Export      │       │
│  │  · ExportTF       │    │ · GetDiff       │    │  · Validate    │       │
│  └───────────────────┘    └─────────────────┘    └────────────────┘       │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            │   Invocación de dominio
            ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            DOMAIN LAYER                                       │
│  ┌────────────────────┐  ┌────────────────────┐  ┌────────────────────────┐ │
│  │  Aggregates        │  │  Value Objects     │  │  Domain Events         │ │
│  │  · Architecture    │  │  · ComponentId     │  │  · ArchitectureCreated │ │
│  │  · Component       │  │  · Position        │  │  · ComponentAdded      │ │
│  │  · Relationship    │  │  · Version         │  │  · ExportRequested     │ │
│  │  · Policy          │  │  · ComponentType   │  │  · RelationshipCreated │ │
│  └────────────────────┘  └────────────────────┘  └────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
            │
            │   Implementación de puertos
            ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        INFRASTRUCTURE LAYER                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────────────────────┐│
│  │  Persistence     │  │  Exporters      │  │  AUF Parser/Serializer         ││
│  │  · PostgreSQL    │  │  · Terraform    │  │  · YAML parsing                ││
│  │  · In-Memory     │  │  · Kubernetes   │  │  · Schema validation           ││
│  │  · File (AUF)    │  │  · (Extensible) │  │  · Diff generation             ││
│  └─────────────────┘  └─────────────────┘  └────────────────────────────────┘│
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────────────────────┐│
│  │  Event Bus      │  │  Storage        │  │  Component Providers           ││
│  │  · In-Memory    │  │  · Local FS     │  │  · AWS Components              ││
│  │  · (Extensible) │  │  · S3           │  │  · Custom Components           ││
│  └─────────────────┘  └─────────────────┘  └────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Diagrama de Flujo de Datos (Exportación)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Flujo de Exportación a Terraform                     │
└─────────────────────────────────────────────────────────────────────────────┘

  Presentation              Application              Domain            Infrastructure
     Layer                    Layer                  Layer                 Layer
       │                       │                       │                     │
       │  1. User solicita     │                       │                     │
       │  "Export to TF"       │                       │                     │
       │──────────────────────>│                       │                     │
       │                       │                       │                     │
       │                       │  2. Command:          │                     │
       │                       │  ExportToTerraform    │                     │
       │                       │───────────────────────>│                     │
       │                       │                       │                     │
       │                       │                       │  3. Load            │
       │                       │                       │  Architecture       │
       │                       │                       │◄────────────────────│
       │                       │                       │                     │
       │                       │                       │  4. Events:         │
       │                       │                       │  ExportRequested    │
       │                       │                       │────────────────────►│
       │                       │                       │                     │  5. Publish
       │                       │                       │                     │  Event
       │                       │                       │                     │◄───────────
       │                       │                       │                     │
       │                       │                       │  6. Return          │
       │                       │                       │  Architecture       │
       │                       │                       │◄────────────────────│
       │                       │                       │                     │
       │                       │  7. Execute           │                     │
       │                       │  HclGenerator         │                     │
       │                       │───────────────────────┼────────────────────►│
       │                       │                       │                     │
       │                       │                       │                     │  8. Generate
       │                       │                       │                     │  HCL Output
       │                       │                       │                     │◄───────────
       │                       │                       │                     │
       │                       │  9. Return HCL        │                     │
       │                       │◄──────────────────────┼─────────────────────│
       │                       │                       │                     │
       │  10. Download/        │                       │                     │
       │  Save File            │                       │                     │
       │◄──────────────────────│                       │                     │
       │                       │                       │                     │
```

### 4.3 Diagrama de Estados (Architecture Aggregate)

```
                                      ┌─────────────────┐
                                      │     DRAFT       │
                                      │  (Initial)      │
                                      └────────┬────────┘
                                               │
                              ┌────────────────┼────────────────┐
                              │                │                │
                              │  add_component │  rename        │
                              ▼                ▼                ▼
                     ┌────────────────┐ ┌────────────────┐ ┌────────────────┐
                     │   HAS_CONTENT  │ │   RENAMED      │ │                │
                     └───────┬────────┘ └───────┬────────┘ │                │
                             │                  │          │                │
                             │  add_relationship│          │                │
                             ▼                  │          │                │
                    ┌─────────────────┐         │          │                │
                    │  HAS_RELATIONS  │         │          │                │
                    └────────┬────────┘         │          │                │
                             │                  │          │                │
                             │  export_tf       │          │                │
                             ▼                  │          │                │
                    ┌─────────────────┐         │          │                │
                    │   EXPORTABLE    │         │          │                │
                    └────────┬────────┘         │          │                │
                             │                  │          │                │
                             │  version_bump    │          │                │
                             │  publish         │          │                │
                             ▼                  │          │                │
                    ┌─────────────────┐         │          │                │
                    │   PUBLISHED     │◄─────────┘          │                │
                    └────────┬────────┘                     │                │
                             │                             │                │
                             │  deprecated                 │                │
                             ▼                             │                │
                    ┌─────────────────┐                    │                │
                    │   DEPRECATED    │                    │                │
                    └─────────────────┘                    │                │
                                                         │                │
                              ┌───────────────────────────┘                │
                              │                                              │
                              │  delete                                      │
                              ▼                                              │
                    ┌─────────────────┐                                     │
                    │    DELETED      │◄────────────────────────────────────┘
                    └─────────────────┘
```

---

## 5. Diseño de Crates

### 5.1 Estructura del Workspace

```
/home/rubentxu/Proyectos/rust/hodei-archFlow/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
│
├── crates/
│   ├── Cargo.toml                      # Workspace configuration
│   │
│   ├── domain/                         # Core domain logic (0 dependencias externas)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── architecture/           # Architecture aggregate root
│   │   │   │   ├── mod.rs
│   │   │   │   ├── architecture.rs
│   │   │   │   ├── component.rs
│   │   │   │   ├── relationship.rs
│   │   │   │   └── policy.rs
│   │   │   ├── events/                 # Domain events
│   │   │   │   ├── mod.rs
│   │   │   │   └── architecture_events.rs
│   │   │   ├── value_objects/          # Value objects
│   │   │   │   ├── mod.rs
│   │   │   │   ├── component_id.rs
│   │   │   │   ├── position.rs
│   │   │   │   └── version.rs
│   │   │   └── errors/                 # Domain errors
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   ├── application/                    # Use cases and orchestration
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── commands/               # Command handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── create_architecture.rs
│   │   │   │   ├── add_component.rs
│   │   │   │   └── export_terraform.rs
│   │   │   ├── queries/                # Query handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── get_architecture.rs
│   │   │   │   └── list_architectures.rs
│   │   │   ├── services/               # Application services
│   │   │   │   ├── mod.rs
│   │   │   │   └── export_service.rs
│   │   │   └── dto/                    # Data transfer objects
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   ├── infrastructure/                 # External adapters
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── persistence/            # Database adapters
│   │   │   │   ├── mod.rs
│   │   │   │   ├── postgres/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── repository.rs
│   │   │   │   └── memory/
│   │   │   │       └── mod.rs
│   │   │   ├── event_bus/              # Event bus implementation
│   │   │   │   ├── mod.rs
│   │   │   │   └── in_memory_bus.rs
│   │   │   ├── export/                 # IaC exporters
│   │   │   │   ├── mod.rs
│   │   │   │   └── terraform/
│   │   │   │       ├── mod.rs
│   │   │   │       └── hcl_generator.rs
│   │   │   ├── storage/                # File storage
│   │   │   │   ├── mod.rs
│   │   │   │   └── local_fs.rs
│   │   │   ├── auf/                    # AUF format
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parser.rs
│   │   │   │   └── serializer.rs
│   │   │   └── components/             # Component providers
│   │   │       ├── mod.rs
│   │   │       └── aws/
│   │   │           └── mod.rs
│   │   └── tests/
│   │
│   ├── presentation/                   # Frontend and interfaces
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── leptos_app/             # Leptos WASM frontend
│   │   │   │   ├── mod.rs
│   │   │   │   ├── app.rs
│   │   │   │   ├── components/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── canvas.rs
│   │   │   │   │   ├── toolbar.rs
│   │   │   │   │   └── properties_panel.rs
│   │   │   │   ├── pages/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── home.rs
│   │   │   │   │   └── editor.rs
│   │   │   │   ├── state/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── store.rs
│   │   │   │   └── lib.rs
│   │   │   ├── cli/                    # CLI application
│   │   │   │   ├── mod.rs
│   │   │   │   ├── commands/
│   │   │   │   │   ├── init.rs
│   │   │   │   │   ├── export.rs
│   │   │   │   │   └── serve.rs
│   │   │   │   └── main.rs
│   │   │   └── grpc/                   # gRPC service (futuro)
│   │   │       └── mod.rs
│   │   └── tests/
│   │
│   └── shared/                         # Shared utilities
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config/                 # Configuration
│       │   │   ├── mod.rs
│       │   │   └── settings.rs
│       │   ├── logging/                # Logging
│       │   │   └── mod.rs
│       │   ├── telemetry/              # Tracing/metrics
│       │   │   └── mod.rs
│       │   └── async_utils/            # Async helpers
│       │       └── mod.rs
│       └── tests/
│
├── docs/                               # Documentación
│   ├── prd.md
│   ├── PRD-CRITICA.md
│   └── ARCHITECTURE-STUDY.md           # Este documento
│
└── .github/
    └── workflows/
        └── ci.yml
```

### 5.2 Dependencias del Workspace

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
# Core
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
thiserror = "2.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }

# Configuration
config = "0.14"

# Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Async
futures = "0.3"
tokio-stream = "0.1"

# Testing
rstest = "0.18"
proptest = "1.4"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

### 5.3 Dependencias por Crate

**domain/Cargo.toml:**
```toml
[package]
name = "archflow-domain"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
derive_more = "1.0"

[dev-dependencies]
rstest.workspace = true
```

**application/Cargo.toml:**
```toml
[package]
name = "archflow-application"
version.workspace = true
edition.workspace = true

[dependencies]
archflow-domain = { path = "../domain" }
async-trait.workspace = true
thiserror.workspace = true
tokio.workspace = true
serde.workspace = true
futures.workspace = true
```

**infrastructure/Cargo.toml:**
```toml
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
serde_yaml.workspace = true
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"], optional = true }
```

**presentation/Cargo.toml:**
```toml
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
leptos = "0.6"
leptos_use = { version = "0.10", optional = true }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "console",
    "Window",
    "Document",
    "HtmlCanvasElement",
    "WebGl2RenderingContext",
] }
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O4"]
```

**shared/Cargo.toml:**
```toml
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

## 6. Modelo de Dominio Detallado

### 6.1 Arquitectura Aggregate

```rust
// domain/src/architecture/architecture.rs

use crate::value_objects::{ArchitectureId, Version};
use crate::architecture::{Component, Layer, Relationship, Policy};
use crate::events::ArchitectureEvent;
use crate::errors::DomainError;
use std::collections::HashMap;

/// Architecture es el aggregate root del sistema.
/// Todo acceso al dominio pasa por este aggregate.
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
    uncommitted_events: Vec<ArchitectureEvent>,
}

impl Architecture {
    pub fn new(name: String, description: String) -> Result<Self, DomainError> {
        let architecture = Architecture {
            id: ArchitectureId::new(),
            name,
            description,
            version: Version::v1(),
            metadata: ArchitectureMetadata::default(),
            layers: vec![Layer::new("default", LayerType::Container)],
            components: HashMap::new(),
            relationships: Vec::new(),
            policies: Vec::new(),
            state: ArchitectureState::Draft,
            uncommitted_events: Vec::new(),
        };
        
        architecture.record_event(ArchitectureEvent::ArchitectureCreated {
            architecture_id: architecture.id,
            name: architecture.name.clone(),
            timestamp: std::time::SystemTime::now(),
        });
        
        Ok(architecture)
    }

    pub fn add_component(&mut self, component: Component) -> Result<(), DomainError> {
        if self.components.contains_key(&component.id()) {
            return Err(DomainError::ComponentAlreadyExists(component.id()));
        }
        
        self.components.insert(component.id(), component.clone());
        
        self.record_event(ArchitectureEvent::ComponentAdded {
            architecture_id: self.id,
            component_id: component.id,
            component_type: format!("{:?}", component.component_type()),
            timestamp: std::time::SystemTime::now(),
        });
        
        Ok(())
    }

    pub fn remove_component(&mut self, id: &ComponentId) -> Result<(), DomainError> {
        let component = self.components.remove(id)
            .ok_or(DomainError::ComponentNotFound(*id))?;
        
        self.relationships.retain(|r| !r.involves(id));
        
        self.record_event(ArchitectureEvent::ComponentRemoved {
            architecture_id: self.id,
            component_id: *id,
            timestamp: std::time::SystemTime::now(),
        });
        
        Ok(())
    }

    pub fn update_component(
        &mut self, 
        id: &ComponentId, 
        properties: HashMap<String, PropertyValue>,
    ) -> Result<(), DomainError> {
        let component = self.components.get_mut(id)
            .ok_or(DomainError::ComponentNotFound(*id))?;
        
        component.update_properties(properties.clone());
        
        self.record_event(ArchitectureEvent::ComponentUpdated {
            architecture_id: self.id,
            component_id: *id,
            changed_properties: properties.keys().cloned().collect(),
            timestamp: std::time::SystemTime::now(),
        });
        
        Ok(())
    }

    pub fn version_bump(&mut self) {
        self.version = self.version.bump_minor();
        self.record_event(ArchitectureEvent::ArchitecturePublished {
            architecture_id: self.id,
            version: self.version.to_string(),
            timestamp: std::time::SystemTime::now(),
        });
    }

    fn record_event(&mut self, event: ArchitectureEvent) {
        self.uncommitted_events.push(event);
    }

    pub fn take_events(&mut self) -> Vec<ArchitectureEvent> {
        std::mem::take(&mut self.uncommitted_events)
    }

    // Getters
    pub fn id(&self) -> &ArchitectureId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn version(&self) -> &Version { &self.version }
    pub fn components(&self) -> impl Iterator<Item = &Component> { self.components.values() }
    pub fn relationships(&self) -> &[Relationship] { &self.relationships }
    pub fn state(&self) -> ArchitectureState { self.state }
    pub fn component(&self, id: &ComponentId) -> Option<&Component> {
        self.components.get(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArchitectureState {
    Draft,
    HasContent,
    HasRelations,
    Exportable,
    Published,
    Deprecated,
    Deleted,
}

#[derive(Debug, Clone, Default)]
pub struct ArchitectureMetadata {
    pub created_by: Option<String>,
    pub created_at: Option<std::time::SystemTime>,
    pub updated_at: Option<std::time::SystemTime>,
    pub tags: Vec<String>,
    pub owners: Vec<String>,
}
```

### 6.2 Component Aggregate

```rust
// domain/src/architecture/component.rs

use crate::value_objects::{ComponentId, Position, ComponentType, CloudProvider};
use crate::errors::DomainError;
use std::collections::HashMap;

/// Component representa un elemento atómico de la arquitectura.
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

    pub fn to_hcl_resource(&self) -> Option<String> {
        self.iac_mappings.get(&IacType::Terraform).cloned()
    }

    // Getters
    pub fn id(&self) -> &ComponentId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn component_type(&self) -> &ComponentType { &self.component_type }
    pub fn position(&self) -> &Position { &self.position }
    pub fn properties(&self) -> &ComponentProperties { &self.properties }
    pub fn cloud_provider(&self) -> Option<CloudProvider> { self.cloud_provider }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    
    // Custom
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentCategory {
    Compute,
    Storage,
    Network,
    Security,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    Aws,
    Azure,
    Gcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IacType {
    Terraform,
    Pulumi,
    Kubernetes,
    CloudFormation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProperties(HashMap<String, PropertyValue>);

impl ComponentProperties {
    pub fn new() -> Self {
        ComponentProperties(HashMap::new())
    }
    
    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.0.get(key)
    }
    
    pub fn insert(&mut self, key: String, value: PropertyValue) {
        self.0.insert(key, value);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValue {
    pub value: serde_json::Value,
    pub type_hint: Option<String>,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub expression: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    RequiredProperty,
    TypeConstraint,
    RelationshipConstraint,
    CapacityConstraint,
}
```

### 6.3 Domain Events

```rust
// domain/src/events/architecture_events.rs

use crate::value_objects::{ArchitectureId, ComponentId};
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchitectureEvent {
    // Arquitectura
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
    
    // Componentes
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
    
    // Relaciones
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
    
    // Exportación
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
```

### 6.4 Value Objects

```rust
// domain/src/value_objects/mod.rs

use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::fmt;

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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }
    
    pub fn with_z(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

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

## 7. Formato Architecture Universal Format (AUF)

### 7.1 Schema JSON

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://archflow.dev/schemas/auf/v1.0",
  "title": "Architecture Universal Format",
  "description": "Schema for ArchFlow architecture definitions",
  
  "type": "object",
  "required": ["version", "metadata", "components"],
  
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^[0-9]+\\.[0-9]+\\.[0-9]+$"
    },
    
    "metadata": {
      "type": "object",
      "required": ["id", "name"],
      "properties": {
        "id": { "type": "string", "pattern": "^arch-[a-f0-9]+$" },
        "name": { "type": "string", "minLength": 1, "maxLength": 255 },
        "description": { "type": "string", "maxLength": 1000 },
        "version": { "type": "string" },
        "created": { "type": "string", "format": "date-time" },
        "updated": { "type": "string", "format": "date-time" },
        "owners": { "type": "array", "items": { "type": "string" } },
        "tags": { "type": "array", "items": { "type": "string" } }
      }
    },
    
    "layers": {
      "type": "array",
      "items": { "$ref": "#/$defs/layer" }
    },
    
    "components": {
      "type": "array",
      "items": { "$ref": "#/$defs/component" }
    },
    
    "relationships": {
      "type": "array",
      "items": { "$ref": "#/$defs/relationship" }
    },
    
    "policies": {
      "type": "object",
      "properties": {
        "security": { "type": "array", "items": { "$ref": "#/$defs/securityPolicy" } },
        "cost": { "type": "array", "items": { "$ref": "#/$defs/costPolicy" } },
        "compliance": { "type": "array", "items": { "$ref": "#/$defs/compliancePolicy" } }
      }
    }
  },
  
  "$defs": {
    "layer": {
      "type": "object",
      "required": ["id", "type"],
      "properties": {
        "id": { "type": "string" },
        "type": { "type": "string", "enum": ["context", "container", "component", "deployment"] },
        "name": { "type": "string" },
        "description": { "type": "string" }
      }
    },
    
    "component": {
      "type": "object",
      "required": ["id", "name", "type", "position"],
      "properties": {
        "id": { "type": "string", "pattern": "^comp-[a-f0-9]+$" },
        "name": { "type": "string" },
        "type": { "type": "string" },
        "cloud_provider": { "type": "string", "enum": ["aws", "azure", "gcp"] },
        "position": { "$ref": "#/$defs/position" },
        "properties": { "type": "object" },
        "constraints": { "type": "array", "items": { "type": "object" } },
        "iac_mappings": {
          "type": "object",
          "properties": {
            "terraform": { "type": "string" },
            "kubernetes": { "type": "string" }
          }
        }
      }
    },
    
    "position": {
      "type": "object",
      "required": ["x", "y"],
      "properties": {
        "x": { "type": "number" },
        "y": { "type": "number" },
        "z": { "type": "number", "default": 0 }
      }
    },
    
    "relationship": {
      "type": "object",
      "required": ["from", "to", "type"],
      "properties": {
        "from": { "type": "string" },
        "to": { "type": "string" },
        "type": { "type": "string" },
        "properties": { "type": "object" }
      }
    },
    
    "securityPolicy": {
      "type": "object",
      "required": ["policy"],
      "properties": {
        "policy": { "type": "string" },
        "enforcement": { "type": "string", "enum": ["required", "optional", "audit"] }
      }
    },
    
    "costPolicy": {
      "type": "object",
      "required": ["policy"],
      "properties": {
        "policy": { "type": "string" },
        "schedule": { "type": "string" }
      }
    },
    
    "compliancePolicy": {
      "type": "object",
      "required": ["standard"],
      "properties": {
        "standard": { "type": "string" },
        "controls": { "type": "array", "items": { "type": "string" } }
      }
    }
  }
}
```

### 7.2 Ejemplo AUF

```yaml
# ArchFlow Architecture Universal Format v1.0
version: "1.0.0"

metadata:
  id: "arch-prod-ecommerce-2024q1"
  name: "E-Commerce Platform Production"
  description: "Production architecture for e-commerce platform"
  version: "3.2.1"
  created: "2024-01-15T10:30:00Z"
  updated: "2024-01-20T14:45:00Z"
  owners:
    - "team:platform"
    - "team:security"
  tags:
    - "production"
    - "ecommerce"
    - "aws"

layers:
  - id: "c4_context"
    type: "context"
    name: "System Context"
  
  - id: "c4_container"
    type: "container"
    name: "Container Architecture"
  
  - id: "deployment"
    type: "deployment"
    name: "AWS Deployment View"

components:
  # Compute - Web Tier
  - id: "comp-web-alb"
    name: "Web ALB"
    type: "LoadBalancer"
    cloud_provider: "aws"
    position: { x: 100, y: 50, z: 0 }
    properties:
      scheme: "internet-facing"
      idle_timeout: 60
      cross_zone_load_balancing: true
    iac_mappings:
      terraform: "aws_lb.web_alb"

  - id: "comp-web-asg"
    name: "Web ASG"
    type: "Ec2Instance"
    cloud_provider: "aws"
    position: { x: 100, y: 150, z: 0 }
    properties:
      instance_type: "t3.medium"
      min_size: 2
      max_size: 10
      ami: "ami-0c55b159cbfafe1f0"

  # Storage
  - id: "comp-s3-assets"
    name: "S3 Assets Bucket"
    type: "S3Bucket"
    cloud_provider: "aws"
    position: { x: 300, y: 50, z: 0 }
    properties:
      bucket_name: "prod-ecommerce-assets"
      versioning: true
      encryption: "AES256"
    iac_mappings:
      terraform: "aws_s3_bucket.assets"

  # Database
  - id: "comp-rds-postgres"
    name: "RDS PostgreSQL"
    type: "RdsInstance"
    cloud_provider: "aws"
    position: { x: 300, y: 200, z: 0 }
    properties:
      engine: "postgres"
      instance_class: "db.t3.medium"
      multi_az: true
      backup_retention: 30

  # Serverless
  - id: "comp-lambda-auth"
    name: "Auth Lambda"
    type: "LambdaFunction"
    cloud_provider: "aws"
    position: { x: 500, y: 100, z: 0 }
    properties:
      runtime: "python3.11"
      timeout: 30
      memory_size: 256

  # Network
  - id: "comp-vpc-main"
    name: "Main VPC"
    type: "Vpc"
    cloud_provider: "aws"
    position: { x: 100, y: 300, z: 0 }
    properties:
      cidr_block: "10.0.0.0/16"

relationships:
  - from: "comp-web-alb"
    to: "comp-web-asg"
    type: "http_request"
    properties:
      protocol: "HTTPS"
      port: 443

  - from: "comp-web-asg"
    to: "comp-rds-postgres"
    type: "database_connection"
    properties:
      port: 5432
      ssl: "required"

  - from: "comp-web-asg"
    to: "comp-s3-assets"
    type: "data_access"
    properties:
      permission: "read_write"

  - from: "comp-lambda-auth"
    to: "comp-rds-postgres"
    type: "database_connection"
    properties:
      port: 5432

policies:
  security:
    - policy: "encryption_in_transit"
      enforcement: "required"
    - policy: "no_public_s3_buckets"
      enforcement: "required"
  
  cost:
    - policy: "auto_shutdown_dev"
      schedule: "weekdays_19:00-07:00"
  
  compliance:
    - standard: "SOC2"
      controls: ["CC6.1", "CC7.1"]
```

---

## 8. Persistencia de Datos

### 8.1 Modelo de Datos PostgreSQL

```sql
-- Esquema de base de datos para ArchFlow

-- Tabla principal de arquitecturas
CREATE TABLE architectures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version_major INTEGER NOT NULL DEFAULT 1,
    version_minor INTEGER NOT NULL DEFAULT 0,
    version_patch INTEGER NOT NULL DEFAULT 0,
    state VARCHAR(50) NOT NULL DEFAULT 'draft',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT unique_architecture_version UNIQUE (id, version_major, version_minor, version_patch)
);

CREATE INDEX idx_architectures_name ON architectures(name);
CREATE INDEX idx_architectures_state ON architectures(state);

-- Tabla de componentes
CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    architecture_id UUID NOT NULL REFERENCES architectures(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    component_type VARCHAR(100) NOT NULL,
    cloud_provider VARCHAR(50),
    position_x DOUBLE PRECISION NOT NULL,
    position_y DOUBLE PRECISION NOT NULL,
    position_z DOUBLE PRECISION DEFAULT 0,
    properties JSONB DEFAULT '{}',
    constraints JSONB DEFAULT '[]',
    iac_mappings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_components_architecture ON components(architecture_id);
CREATE INDEX idx_components_type ON components(component_type);

-- Tabla de relaciones
CREATE TABLE relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    architecture_id UUID NOT NULL REFERENCES architectures(id) ON DELETE CASCADE,
    from_component UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    to_component UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT valid_relationship_components CHECK (from_component != to_component)
);

CREATE INDEX idx_relationships_architecture ON relationships(architecture_id);
CREATE INDEX idx_relationships_from ON relationships(from_component);
CREATE INDEX idx_relationships_to ON relationships(to_component);

-- Tabla de eventos (para event sourcing futuro)
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    correlation_id UUID,
    causation_id UUID,
    user_id VARCHAR(255),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    sequence_number BIGSERIAL
);

CREATE INDEX idx_events_aggregate ON events(aggregate_id, aggregate_type);
CREATE INDEX idx_events_correlation ON events(correlation_id);
CREATE INDEX idx_events_timestamp ON events(timestamp);

-- Tabla de snapshots (para event sourcing)
CREATE TABLE snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    version INTEGER NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_snapshots_aggregate ON snapshots(aggregate_id, aggregate_type);
```

---

## 9. Integración de Infraestructura

### 9.1 Event Bus (Interfaz)

```rust
// infrastructure/src/event_bus/mod.rs

use async_trait::async_trait;
use domain::events::ArchitectureEvent;
use std::fmt::Debug;

/// Event Bus trait para publicación/suscripción de eventos
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: ArchitectureEvent) -> Result<(), String>;
    async fn subscribe(&self, handler: Box<dyn EventHandler>) -> Result<SubscriptionId, String>;
    async fn unsubscribe(&self, id: SubscriptionId) -> Result<(), String>;
}

/// Identificador único de suscripción
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionId(uuid::Uuid);

impl SubscriptionId {
    pub fn new() -> Self {
        SubscriptionId(uuid::Uuid::new_v4())
    }
}

/// Handler de eventos
#[async_trait]
pub trait EventHandler: Send + Sync {
    fn event_type(&self) -> &'static str;
    async fn handle(&self, event: &ArchitectureEvent);
}
```

### 9.2 Repositorio (Puerto)

```rust
// infrastructure/src/persistence/mod.rs

use async_trait::async_trait;
use domain::architecture::Architecture;
use domain::value_objects::ArchitectureId;
use std::error::Error;

/// Puerto para persistencia de arquitecturas
#[async_trait]
pub trait ArchitectureRepository: Send + Sync {
    async fn save(&self, architecture: &Architecture) -> Result<(), Box<dyn Error>>;
    async fn find_by_id(&self, id: &ArchitectureId) -> Result<Option<Architecture>, Box<dyn Error>>;
    async fn find_all(&self) -> Result<Vec<Architecture>, Box<dyn Error>>;
    async fn delete(&self, id: &ArchitectureId) -> Result<(), Box<dyn Error>>;
    async fn exists(&self, id: &ArchitectureId) -> Result<bool, Box<dyn Error>>;
}
```

---

## 10. Interfaz de Usuario (Leptos)

### 10.1 Estructura de Componentes

```
presentation/src/leptos_app/
├── app.rs                    # Componente raíz
├── lib.rs                    # Exports públicos
├── components/
│   ├── mod.rs
│   ├── canvas.rs             # Canvas principal (WebGPU/WebGL)
│   ├── toolbar.rs            # Barra de herramientas
│   ├── properties_panel.rs   # Panel de propiedades
│   ├── component_palette.rs  # Biblioteca de componentes
│   └── layer_panel.rs        # Capas y navegación
├── pages/
│   ├── mod.rs
│   ├── home.rs               # Página de inicio
│   ├── editor.rs             # Editor de arquitectura
│   └── settings.rs           # Configuración
└── state/
    ├── mod.rs
    └── store.rs              # Estado global (Leptos signals)
```

### 10.2 Store de Estado

```rust
// presentation/src/leptos_app/state/store.rs

use leptos::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppStore {
    // Arquitectura actual
    pub architecture_id: RwSignal<Option<String>>,
    pub architecture_name: RwSignal<String>,
    pub architecture_version: RwSignal<String>,
    
    // Componentes
    pub components: RwSignal<Vec<ComponentState>>,
    pub selected_component_id: RwSignal<Option<String>>,
    
    // Canvas
    pub zoom: RwSignal<f32>,
    pub pan_offset: RwSignal<(f32, f32)>,
    pub show_grid: RwSignal<bool>,
    pub snap_to_grid: RwSignal<bool>,
    
    // UI
    pub is_loading: RwSignal<bool>,
    pub error_message: RwSignal<Option<String>>,
    pub notification: RwSignal<Option<Notification>>,
    
    // Propiedades del componente seleccionado
    pub selected_component_properties: RwSignal<HashMap<String, PropertyInput>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            architecture_id: RwSignal::new(None),
            architecture_name: RwSignal::new("Untitled Architecture".to_string()),
            architecture_version: RwSignal::new("1.0.0".to_string()),
            components: RwSignal::new(Vec::new()),
            selected_component_id: RwSignal::new(None),
            zoom: RwSignal::new(1.0),
            pan_offset: RwSignal::new((0.0, 0.0)),
            show_grid: RwSignal::new(true),
            snap_to_grid: RwSignal::new(true),
            is_loading: RwSignal::new(false),
            error_message: RwSignal::new(None),
            notification: RwSignal::new(None),
            selected_component_properties: RwSignal::new(HashMap::new()),
        }
    }
    
    pub fn select_component(&self, id: Option<String>) {
        self.selected_component_id.set(id);
        
        if let Some(id) = &id {
            if let Some(component) = self.components.read().iter().find(|c| &c.id == id) {
                self.selected_component_properties.set(component.properties.clone());
            }
        }
    }
    
    pub fn add_component(&self, component_type: &str, position: (f64, f64)) {
        let id = format!("comp-{}", uuid::Uuid::new_v4().simple());
        
        self.components.update(|components| {
            components.push(ComponentState {
                id: id.clone(),
                name: format!("New {}", component_type),
                component_type: component_type.to_string(),
                position,
                properties: HashMap::new(),
            });
        });
        
        self.select_component(Some(id));
    }
    
    pub fn update_component_position(&self, id: &str, position: (f64, f64)) {
        self.components.update(|components| {
            if let Some(component) = components.iter_mut().find(|c| &c.id == id) {
                component.position = position;
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct ComponentState {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub position: (f64, f64),
    pub properties: HashMap<String, PropertyInput>,
}

#[derive(Clone, Debug)]
pub struct PropertyInput {
    pub value: String,
    pub type_hint: String,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration_ms: u32,
}

#[derive(Clone, Debug)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}
```

---

## 11. Roadmap de Implementación

### Fase 1: Fundación (Meses 1-3)

| Semana | Entregable | Descripción |
|--------|------------|-------------|
| 1-2 | Workspace setup | Configurar Cargo workspace, CI/CD, herramientas |
| 3-4 | Domain layer | Aggregates, Value Objects, Events básicos |
| 5-6 | AUF parser | YAML parsing, schema validation |
| 7-8 | Leptos scaffold | Frontend básico, Canvas component |
| 9-10 | CRUD commands | Create/Read/Update/Delete de arquitecturas |
| 11-12 | Integración básica | Conexión frontend con domain |

### Fase 2: Core Features (Meses 4-6)

| Semana | Entregable | Descripción |
|--------|------------|-------------|
| 13-14 | Terraform exporter | HCL generator para componentes básicos |
| 15-16 | AWS components | Librería de componentes AWS (10 tipos) |
| 17-18 | Event bus | Implementación in-memory |
| 19-20 | Persistence | PostgreSQL repository |
| 21-24 | Polish | UX refinements, bug fixing |

### Fase 3: Colaboración (Meses 7-9)

| Semana | Entregable | Descripción |
|--------|------------|-------------|
| 25-26 | CLI app | Herramienta de línea de comandos |
| 27-28 | Git integration | AUF files en repositorios |
| 29-30 | Real-time sync | WebSocket para colaboración |
| 31-32 | Comments | Sistema de comentarios |

---

## 12. Métricas de Calidad

### 12.1 Métricas de Código

| Métrica | Objetivo | Herramienta |
|---------|----------|-------------|
| Coverage de tests | > 80% | tarpaulin |
| Coverage de tipos | 100% | cargo-nextest |
| Debt ratio | < 5% | cargo-audit |
| Documentación pública | 100% | cargo-doc |

### 12.2 Métricas de Rendimiento

| Métrica | Objetivo | Condición |
|---------|----------|-----------|
| Tiempo de carga inicial | < 3s | Lighthouse |
| FPS del canvas | 60fps | Con 1000 nodos |
| Tiempo de exportación TF | < 2s | 100 componentes |
| Tiempo de parseo AUF | < 500ms | 10KB archivo |

### 12.3 Métricas de Arquitectura

| Métrica | Objetivo | Verificación |
|---------|----------|--------------|
| Acoplamiento | Baja cohesión | Dependency analysis |
| Testabilidad | Inyección de dependencias | Manual review |
| Extensibilidad | Plugins para componentes | Manual review |
| Portabilidad | Multiplataforma | CI/CD matrix |

---

## 13. Apéndices

### A. Glosario de Términos

| Término | Definición |
|---------|------------|
| Aggregate | Entidad raíz que encapsula invariantes de dominio |
| Bounded Context | Límite explícito donde un modelo es consistente |
| Domain Event | Evento que representa algo significativo en el dominio |
| Entity | Objeto con identidad persistente |
| Value Object | Objeto inmutable sin identidad conceptual |
| Port | Interfaz que define comunicación con el exterior |
| Adapter | Implementación concreta de un puerto |
| Event Bus | Sistema de publicación/suscripción de eventos |
| AUF | Architecture Universal Format (formato YAML) |

### B. Referencias Técnicas

| Tema | Referencia |
|------|------------|
| DDD | "Domain-Driven Design" - Eric Evans |
| Hexagonal Architecture | "Ports & Adapters" - Alistair Cockburn |
| Event Sourcing | "Event Sourcing" - Martin Fowler |
| CQRS | "CQRS" - Martin Fowler |
| Leptos | https://leptos.dev/ |
| Rust WASM | https://rustwasm.github.io/ |

### C. Checklist de Implementación

- [ ] Workspace configurado con crates separados
- [ ] Domain layer sin dependencias externas
- [ ] Application layer con handlers de comandos/queries
- [ ] Infrastructure layer con adaptadores
- [ ] Presentation layer con Leptos
- [ ] AUF parser implementado
- [ ] Terraform exporter implementado
- [ ] Event bus in-memory implementado
- [ ] Tests unitarios para domain
- [ ] Tests de integración para application
- [ ] CI/CD configurado
- [ ] Documentación de API

---

## 14. Aprobaciones

| Rol | Nombre | Firma | Fecha |
|-----|--------|-------|-------|
| Architecture Lead | [Pendiente] | | |
| Engineering Lead | [Pendiente] | | |
| Product Lead | [Pendiente] | | |

---

**Documento generado según análisis del PRD en `docs/prd.md` y crítica en `docs/PRD-CRITICA.md`.**

**Próximos pasos:**
1. Revisión por equipo técnico
2. Aprobación de decisiones arquitectónicas
3. Inicio de implementación según roadmap
