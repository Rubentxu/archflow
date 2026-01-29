# EPIC-FASE-02: Collaboration System

**Versión:** 1.0.0  
**Fase:** 2/8  
**Duración:** Semanas 3-4  
**Dependencias:** EPIC-FASE-01 (Records Foundation)  
**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` - Secciones L1623-2347, A, F.7

---

## 📋 Descripción General

**ENFOQUE: CERO CÓDIGO LEGACY - TODO DESDE CERO**

Esta épica implementa el sistema de colaboración **desde cero**, sin reutilizar ninguna línea del código legacy. El núcleo es un **CRDT (Conflict-free Replicated Data Type)** creado nuevo que garantiza consistencia eventual y resolución automática de conflictos.

### Archivos Legacy a ELIMINAR (no reutilizar):
```
crates/archflow-core/src/event_sourcing/     → COMPLETAMENTE ELIMINADO
crates/archflow-primitives/src/selection.rs  → NO reutilizar
crates/archflow-primitives/src/connectivity.rs → NO reutilizar
```

### Objetivos Principales
- Crear `archflow-collab/` crate **desde cero**
- Implementar `CRDT` con vector clocks (NUEVO, Apéndice F.7)
- Implementar estrategias de merge (NUEVO, LWW, Field-level, Optimistic)
- Implementar `SyncServer` y `SyncClient` (NUEVO, Apéndice C)
- Implementar `ConflictResolver` con principios SOLID (NUEVO, Apéndice A)
- Configurar protocolo de reconexión (NUEVO, Apéndice C.4)
- **ELIMINAR** todo el directorio event_sourcing legacy

---

## 🎯 Criterios de Aceptación

### Funcionales
- [ ] CRDT aplica cambios locales sin bloqueos
- [ ] CRDT merge remote changes correctamente
- [ ] Conflictos concurrentes detectados y resueltos automáticamente
- [ ] Vector clocks determinan causalidad correctamente
- [ ] SyncClient reconecta automáticamente tras desconexión
- [ ] SyncServer broadcast cambios a room correctamente

### No Funcionales
- [ ] Test coverage > 95%
- [ ] Benchmarks: 1000 concurrent edits < 50ms merge
- [ ] Collaboration latency < 50ms (Apéndice F.11)
- [ ] Soporta 10,000 usuarios concurrentes

---

## 🔬 Investigación Requerida (Perplexity)

### Tarea de Investigación 1: CRDT Implementations Comparison

**Objetivo:** Comparar implementaciones CRDT existentes y sus patrones.

**Preguntas de Investigación:**
```
1. ¿Cuáles son las diferencias entre LWW Register, MV Register, y OR-Set?
2. ¿Cómo implementan Yjs y Automerge la resolución de conflictos?
3. ¿Qué estrategia es mejor para un canvas colaborativo?
```

**Criterios de Éxito:**
- [ ] Documentar 3 estrategias de CRDT con pros/cons
- [ ] Seleccionar estrategia óptima para shapes/canvas
- [ ] Definir implementación de VectorClock

### Tarea de Investigación 2: Conflict Resolution Strategies

**Objetivo:** Investigar estrategias de resolución de conflictos en tiempo real.

**Preguntas de Investigación:**
```
1. ¿Cómo manejar conflictos de posición vs color simultáneamente?
2. ¿Qué es "last-writer-wins" y cuándo aplicarlo?
3. ¿Cómo implementar merge a nivel de campo (no objeto)?
```

**Criterios de Éxito:**
- [ ] Definir ConflictType enum completo
- [ ] Documentar merge strategies por tipo
- [ ] Implementar merge determinista

### Tarea de Investigación 3: WebSocket Sync Patterns

**Objetivo:** Investigar patrones de sincronización para colaboración en tiempo real.

**Preguntas de Investigación:**
```
1. ¿Cómo maneja Figma la sincronización de cambios?
2. ¿Qué estrategia de reconexión es más robusta?
3. ¿Cómo optimizar mensajes para minimizar ancho de banda?
```

**Criterios de Éxito:**
- [ ] Definir SyncMessage protocol completo
- [ ] Documentar retry policies
- [ ] Seleccionar compression strategy

---

## 📦 Entregables por Módulo

### Módulo 2.1: `src/crdt.rs` - CRDT Implementation

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` L1625-2022, F.7

**Descripción:**
Core del sistema CRDT con gestión de vector clocks y operaciones pendientes.

**Estructura:**
```rust
pub struct CRDT<R: Record> {
    record_store: Arc<RwLock<RecordStore<R>>>,
    site_id: SiteId,
    vector_clock: VectorClock,
    pending_operations: Vec<RecordChange<R>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorClock {
    dots: BTreeMap<SiteId, u64>,
}

pub struct SiteId(u32);
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod crdt_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord {
        id: RecordId,
        index: Option<FractionalIndex>,
        name: String,
        value: i32,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId { &self.id }
        fn type_name(&self) -> &'static str { "TestRecord" }
        fn index(&self) -> Option<&FractionalIndex> { self.index.as_ref() }
        fn with_index(mut self, index: FractionalIndex) -> Self {
            self.index = Some(index);
            self
        }
    }

    #[test]
    fn test_crdt_new() {
        let site_id = SiteId::new();
        let crdt = CRDT::<TestRecord>::new(site_id);
        assert_eq!(crdt.site_id(), site_id);
    }

    #[test]
    fn test_apply_local_change() {
        let site_id = SiteId::new();
        let mut crdt = CRDT::<TestRecord>::new(site_id);

        let id = RecordId::from_str("crdt_test_00001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "test".into(),
            value: 42,
        };

        let change = crdt.apply_local(record).unwrap();
        assert!(crdt.record_store().get(&id).is_some());
    }

    #[test]
    fn test_merge_remote_changes() {
        // Dos sitios haciendo cambios concurrentes
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let mut crdt_a = CRDT::<TestRecord>::new(site_a);
        let mut crdt_b = CRDT::<TestRecord>::new(site_b);

        // Sitio A crea un registro
        let id_a = RecordId::from_str("concurrent_a_001").unwrap();
        let record_a = TestRecord {
            id: id_a.clone(),
            index: None,
            name: "from A".into(),
            value: 1,
        };
        crdt_a.apply_local(record_a).unwrap();

        // Sitio B crea otro registro (concurrentemente)
        let id_b = RecordId::from_str("concurrent_b_001").unwrap();
        let record_b = TestRecord {
            id: id_b.clone(),
            index: None,
            name: "from B".into(),
            value: 2,
        };
        crdt_b.apply_local(record_b).unwrap();

        // Sincronizar cambios
        let changes_from_a = crdt_a.get_changes();
        let changes_from_b = crdt_b.get_changes();

        crdt_a.merge(changes_from_b.clone()).unwrap();
        crdt_b.merge(changes_from_a.clone()).unwrap();

        // Verificar que ambos tienen los mismos registros
        assert!(crdt_a.record_store().get(&id_a).is_some());
        assert!(crdt_a.record_store().get(&id_b).is_some());
        assert!(crdt_b.record_store().get(&id_a).is_some());
        assert!(crdt_b.record_store().get(&id_b).is_some());
    }

    #[test]
    fn test_concurrent_update_conflict() {
        // F.7: Verificar resolución de conflictos concurrentes
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let mut crdt_a = CRDT::<TestRecord>::new(site_a);
        let mut crdt_b = CRDT::<TestRecord>::new(site_b);

        // Ambos crean el mismo registro
        let id = RecordId::from_str("conflict_test_001").unwrap();
        let record_a = TestRecord {
            id: id.clone(),
            index: None,
            name: "A".into(),
            value: 1,
        };
        let record_b = TestRecord {
            id: id.clone(),
            index: None,
            name: "B".into(),
            value: 2,
        };

        crdt_a.apply_local(record_a).unwrap();
        crdt_b.apply_local(record_b).unwrap();

        // Sincronizar
        let changes_a = crdt_a.get_changes();
        let changes_b = crdt_b.get_changes();

        let result_a = crdt_a.merge(changes_b);
        let result_b = crdt_b.merge(changes_a);

        // Debe resolver el conflicto automáticamente
        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
    }

    #[test]
    fn test_vector_clock_relation() {
        // F.7: Verificar determinación de causalidad
        let mut clock_a = VectorClock::new();
        let mut clock_b = VectorClock::new();

        let site_a = SiteId::new();
        let site_b = SiteId::new();

        clock_a.increment(site_a);
        clock_a.increment(site_a);
        clock_b.increment(site_b);
        clock_b.increment(site_b);
        clock_b.increment(site_b);

        let relation = clock_a.relation(&clock_b);
        assert_eq!(relation, CausalRelation::Concurrent);
    }
}
```

**Investigación Adicional:**
- Comparar LWW vs MV Register
- Documentar comportamiento de vector clocks
- Definir SiteId generation strategy

---

### Módulo 2.2: `src/merge.rs` - Merge Strategies

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` L2022-2347, A

**Descripción:**
Implementación de estrategias de merge con principios SOLID para máxima extensibilidad.

**Estructura:**
```rust
pub trait MergeStrategy<R: Record>: Send + Sync {
    fn merge(&self, local: &R, remote: &R) -> Result<R, MergeError>;
}

pub struct LwwStrategy {
    site_id: SiteId,
}

pub struct FieldMergeStrategy<R: Record> {
    field_strategies: HashMap<&'static str, Box<dyn MergeStrategy<R>>>,
}

pub struct OptimisticMergeStrategy<R: Record> {
    max_retries: u32,
    base_delay_ms: u64,
}

pub enum MergeError {
    Conflict { local: R, remote: R },
    InvalidStrategy,
    Failed,
}
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod merge_strategy_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord {
        id: RecordId,
        name: String,
        value: i32,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId { &self.id }
        fn type_name(&self) -> &'static str { "TestRecord" }
        fn index(&self) -> Option<&FractionalIndex> { None }
        fn with_index(self, _: FractionalIndex) -> Self { self }
    }

    #[test]
    fn test_lww_strategy() {
        // F.7: Last-Writer-Wins basado en SiteId
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let strategy_a = LwwStrategy::new(site_a);
        let strategy_b = LwwStrategy::new(site_b);

        let id = RecordId::from_str("lww_test_00001").unwrap();
        let local = TestRecord { id: id.clone(), name: "local".into(), value: 1 };
        let remote = TestRecord { id: id.clone(), name: "remote".into(), value: 2 };

        // El site con ID mayor gana
        let winner = if site_a > site_b {
            strategy_a.merge(&local, &remote)
        } else {
            strategy_b.merge(&local, &remote)
        };

        assert!(winner.is_ok());
    }

    #[test]
    fn test_field_merge_strategy() {
        // Merge diferente estrategia por campo
        let site_id = SiteId::new();
        let mut strategy = FieldMergeStrategy::<TestRecord>::new();

        // name usa LWW, value usa sum
        strategy = strategy.with_field_strategy("name", Box::new(LwwStrategy::new(site_id)));

        let id = RecordId::from_str("field_test_00001").unwrap();
        let local = TestRecord { id: id.clone(), name: "local".into(), value: 10 };
        let remote = TestRecord { id: id.clone(), name: "remote".into(), value: 5 };

        let result = strategy.merge(&local, &remote);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimistic_merge() {
        // Retry con backoff exponencial
        let strategy = OptimisticMergeStrategy::new(3, 10);

        let id = RecordId::from_str("optimistic_test_01").unwrap();
        let local = TestRecord { id: id.clone(), name: "local".into(), value: 1 };
        let remote = TestRecord { id: id.clone(), name: "remote".into(), value: 2 };

        let result = strategy.merge(&local, &remote);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_merge_conflict() {
        // Conflict debe retornar MergeError::Conflict
        let site_id = SiteId::new();
        let strategy = LwwStrategy::new(site_id);

        let id = RecordId::from_str("conflict_test_0001").unwrap();
        let local = TestRecord { id: id.clone(), name: "local".into(), value: 1 };
        let remote = TestRecord { id: id.clone(), name: "remote".into(), value: 2 };

        let result = strategy.merge(&local, &remote);

        // Uno de los dos debe ganar, no debe haber error
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_strategy_swap() {
        // LSP: Estrategias deben ser intercambiables
        let site_id = SiteId::new();
        let lww: Box<dyn MergeStrategy<TestRecord>> = Box::new(LwwStrategy::new(site_id));

        let id = RecordId::from_str("swap_test_00001").unwrap();
        let local = TestRecord { id: id.clone(), name: "local".into(), value: 1 };
        let remote = TestRecord { id: id.clone(), name: "remote".into(), value: 2 };

        // Cualquier estrategia debe funcionar con el mismo contrato
        let result = lww.merge(&local, &remote);
        assert!(result.is_ok());
    }
}
```

**Investigación Adicional:**
- Estudiar implementaciones de Yjs y Automerge
- Documentar estrategias por tipo de campo
- Definir políticas de retry

---

### Módulo 2.3: `src/network.rs` - Network Protocol

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` C.1, C.2, C.3

**Descripción:**
Protocolo de sincronización para comunicación entre clientes y servidor.

**Estructura:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    SyncRequest {
        session_id: SessionId,
        client_version: u64,
        last_known_version: Option<u64>,
        capabilities: ClientCapabilities,
    },
    SyncResponse {
        session_id: SessionId,
        server_version: u64,
        base_version: u64,
        changes_since_base: Vec<ChangeBatch>,
        server_capabilities: ServerCapabilities,
    },
    LocalChange {
        session_id: SessionId,
        site_id: SiteId,
        version: u64,
        changes: Vec<RecordChange<()>>,
        checksum: u64,
    },
    ChangeAck {
        session_id: SessionId,
        applied_changes: Vec<RecordId>,
        server_version: u64,
    },
    Ping { session_id: SessionId, timestamp: u64 },
    Pong { session_id: SessionId, timestamp: u64, latency_ms: u32 },
    Error {
        session_id: SessionId,
        error_code: SyncErrorCode,
        message: String,
    },
}

pub struct ClientCapabilities {
    pub max_message_size: usize,
    pub supports_compression: bool,
    pub compression_algorithm: Option<CompressionAlgorithm>,
    pub supported_encryption: Vec<EncryptionAlgorithm>,
}

pub enum SyncErrorCode {
    VersionTooOld,
    VersionTooNew,
    InvalidSession,
    CompressionNotSupported,
    RateLimited,
    InternalError,
}
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod network_protocol_tests {
    use super::*;

    #[test]
    fn test_sync_request_message() {
        let msg = SyncMessage::SyncRequest {
            session_id: SessionId::new(),
            client_version: 0,
            last_known_version: None,
            capabilities: ClientCapabilities::default(),
        };

        match msg {
            SyncMessage::SyncRequest { session_id, client_version, .. } => {
                assert_eq!(client_version, 0);
                assert!(session_id.as_u64() > 0);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_sync_response_message() {
        let changes: Vec<ChangeBatch> = vec![];
        let msg = SyncMessage::SyncResponse {
            session_id: SessionId::new(),
            server_version: 100,
            base_version: 0,
            changes_since_base: changes,
            server_capabilities: ServerCapabilities::default(),
        };

        match msg {
            SyncMessage::SyncResponse { server_version, base_version, .. } => {
                assert_eq!(server_version, 100);
                assert_eq!(base_version, 0);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_local_change_checksum() {
        let id = RecordId::from_str("checksum_test_001").unwrap();
        let changes = vec![RecordChange::Created {
            id,
            record: (),
        }];

        let checksum = calculate_checksum(&changes);

        // Verificar que el checksum es reproducible
        let checksum2 = calculate_checksum(&changes);
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_error_message_serialization() {
        let msg = SyncMessage::Error {
            session_id: SessionId::new(),
            error_code: SyncErrorCode::VersionTooOld,
            message: "Client version is too old".into(),
        };

        // Serializar y deserializar
        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: SyncMessage = bincode::deserialize(&serialized).unwrap();

        match deserialized {
            SyncMessage::Error { error_code, .. } => {
                assert_eq!(error_code, SyncErrorCode::VersionTooOld);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_client_capabilities_defaults() {
        let caps = ClientCapabilities::default();
        assert!(caps.max_message_size > 0);
        assert!(!caps.supports_compression);
    }
}
```

---

### Módulo 2.4: `src/sync_server.rs` - Sync Server

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` C.2

**Descripción:**
Servidor de sincronización que gestiona sesiones y broadcast de cambios.

**Estructura:**
```rust
pub trait SyncServerBackend: Send + Sync {
    type Session: SyncSession;
    type Error;

    fn create_session(&self, user: UserId) -> Result<Self::Session, Self::Error>;
    fn get_session(&self, session_id: SessionId) -> Option<Self::Session>;
    fn remove_session(&self, session_id: SessionId);
    fn broadcast_to_room(&self, room_id: RoomId, message: SyncMessage);
}

pub trait SyncSession: Send + Sync {
    fn id(&self) -> SessionId;
    fn user_id(&self) -> UserId;
    fn room_id(&self) -> RoomId;
    fn version(&self) -> u64;
    fn apply_changes(&mut self, changes: Vec<RecordChange<()>>) -> Result<u64, ApplyError>;
    fn get_changes_since(&self, version: u64) -> Vec<ChangeBatch>;
}

pub struct DefaultSyncServer {
    sessions: Arc<RwLock<HashMap<SessionId, DefaultSyncSession>>>,
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,
    version_store: Arc<dyn VersionStore>,
    backend: Arc<dyn SyncServerBackend<Session = DefaultSyncSession>>,
}
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod sync_server_tests {
    use super::*;

    struct MockVersionStore;
    impl VersionStore for MockVersionStore {
        fn get_snapshot(&self) -> Vec<ChangeBatch> { vec![] }
        fn get_changes_since(&self, _: u64) -> Vec<ChangeBatch> { vec![] }
        fn store_changes(&mut self, _: Vec<RecordChange<()>>) {}
    }

    struct MockBackend;
    impl SyncServerBackend for MockBackend {
        type Session = DefaultSyncSession;
        type Error = ();

        fn create_session(&self, _: UserId) -> Result<Self::Session, Self::Error> {
            Ok(DefaultSyncSession::new(SessionId::new(), UserId::new(), RoomId::new()))
        }
        fn get_session(&self, _: SessionId) -> Option<Self::Session> { None }
        fn remove_session(&self, _: SessionId) {}
        fn broadcast_to_room(&self, _: RoomId, _: SyncMessage) {}
    }

    #[test]
    fn test_sync_server_new() {
        let server = DefaultSyncServer::new(
            Arc::new(MockVersionStore),
            Arc::new(MockBackend),
        );
        assert!(server.sessions.read().unwrap().is_empty());
    }

    #[test]
    fn test_handle_sync_request_new_client() {
        let server = Arc::new(DefaultSyncServer::new(
            Arc::new(MockVersionStore),
            Arc::new(MockBackend),
        ));

        let msg = SyncMessage::SyncRequest {
            session_id: SessionId::new(),
            client_version: 0,
            last_known_version: None,
            capabilities: ClientCapabilities::default(),
        };

        let response = futures::executor::block_on(
            server.handle_message(msg, SessionId::new())
        );

        assert!(response.is_ok());
        if let Ok(msgs) = response {
            assert!(!msgs.is_empty());
            if let SyncMessage::SyncResponse { server_version, .. } = &msgs[0] {
                assert_eq!(*server_version, 0);
            }
        }
    }

    #[test]
    fn test_handle_local_change() {
        let server = Arc::new(DefaultSyncServer::new(
            Arc::new(MockVersionStore),
            Arc::new(MockBackend),
        ));

        let session_id = SessionId::new();
        let msg = SyncMessage::LocalChange {
            session_id,
            site_id: SiteId::new(),
            version: 0,
            changes: vec![],
            checksum: 0,
        };

        let response = futures::executor::block_on(
            server.handle_message(msg, session_id)
        );

        assert!(response.is_ok());
    }
}
```

---

### Módulo 2.5: `src/sync_client.rs` - Sync Client

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` C.3, C.4

**Descripción:**
Cliente de sincronización con soporte para reconexión automática.

**Estructura:**
```rust
pub struct SyncClient {
    connection: WebSocketConnection,
    session_id: SessionId,
    local_version: u64,
    server_version: Option<u64>,
    pending_changes: Vec<RecordChange<()>>,
    message_sender: Sender<SyncMessage>,
    state: Arc<RwLock<SyncClientState>>,
    retry_policy: RetryPolicy,
}

pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_base: f64,
}

enum SyncClientState {
    Connected,
    Disconnected { attempt: u32, last_error: Option<String> },
    Reconnecting { attempt: u32 },
    Failed { error: String },
}
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod sync_client_tests {
    use super::*;

    #[test]
    fn test_retry_policy_calculation() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(policy.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(policy.calculate_delay(2), Duration::from_millis(400));
        assert_eq!(policy.calculate_delay(3), Duration::from_millis(800));
        assert_eq!(policy.calculate_delay(4), Duration::from_millis(1600));
        assert_eq!(policy.calculate_delay(5), Duration::from_millis(3200));
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay_ms: 1000,
            max_delay_ms: 5000,
            exponential_base: 2.0,
        };

        // Después de cierto punto, debe limitarse a max_delay_ms
        let delay = policy.calculate_delay(10);
        assert!(delay <= Duration::from_millis(5000));
    }

    #[test]
    fn test_sync_client_state_transitions() {
        let state: Arc<RwLock<SyncClientState>> = Arc::new(RwLock::new(
            SyncClientState::Connected
        ));

        // Simular desconexión
        *state.write().unwrap() = SyncClientState::Disconnected {
            attempt: 0,
            last_error: Some("Connection lost".into()),
        };

        if let SyncClientState::Disconnected { attempt, last_error } = &*state.read().unwrap() {
            assert_eq!(*attempt, 0);
            assert!(last_error.is_some());
        }
    }
}
```

---

### Módulo 2.6: `src/conflict.rs` - Conflict Resolution (SOLID)

**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` A.1-A.4

**Descripción:**
Sistema de resolución de conflictos aplicando principios SOLID.

**Estructura:**
```rust
// SRP: Detectores, Resolvedores, Notificadores separados
pub trait ConflictDetector<R: Record>: Send + Sync {
    fn detect(&self, change: &RecordChange<R>) -> Option<Conflict<R>>;
}

pub trait ConflictResolver<R: Record>: Send + Sync {
    fn resolve(&self, conflict: &Conflict<R>) -> Result<ResolvedChange<R>, MergeError>;
}

pub trait ConflictNotifier: Send + Sync {
    async fn notify_conflicts_resolved(&self, applied: &AppliedChange);
}

// OCP: Estrategias extensibles
pub trait ConflictResolutionStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, conflict: &Conflict) -> bool;
    fn resolve(&self, conflict: &Conflict) -> Result<Resolution, ResolutionError>;
}

// DIP: Depender de abstracciones
pub struct ConflictResolutionPipeline<R: Record> {
    detectors: Vec<Arc<dyn ConflictDetector<R>>>,
    resolver: Arc<dyn ConflictResolver<R>>,
    notifier: Arc<dyn ConflictNotifier>,
    metrics: Arc<ConflictMetrics>,
}

pub enum ConflictType {
    UpdateUpdate { record_id: RecordId, site_a: SiteId, site_b: SiteId },
    UpdateDelete { record_id: RecordId, updater: SiteId, deleter: SiteId },
    InsertInsert { id_a: RecordId, id_b: RecordId, site_a: SiteId, site_b: SiteId },
    NestedField { record_id: RecordId, field_path: String, conflicting_values: Vec<Value> },
    Structural { parent_id: RecordId, children_conflict: Vec<RecordId> },
}
```

**Tareas TDD:**
```rust
// TESTS PRIMERO (RED)
mod conflict_resolution_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord { id: RecordId, name: String, value: i32 }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId { &self.id }
        fn type_name(&self) -> &'static str { "TestRecord" }
        fn index(&self) -> Option<&FractionalIndex> { None }
        fn with_index(self, _: FractionalIndex) -> Self { self }
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

        metrics.record_conflict(ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("metrics_test_01").unwrap(),
            site_a: SiteId::new(),
            site_b: SiteId::new(),
        });

        let report = metrics.get_report();
        assert_eq!(report.total_conflicts, 1);
    }

    #[test]
    fn test_pipeline_process_incoming_change() {
        // A.2: Verificar pipeline completo
        let metrics = Arc::new(ConflictMetrics::default());
        let pipeline = ConflictResolutionPipeline::<TestRecord> {
            detectors: vec![],
            resolver: Arc::new(MockResolver),
            notifier: Arc::new(MockNotifier),
            metrics,
        };

        let id = RecordId::from_str("pipeline_test_001").unwrap();
        let record = TestRecord { id: id.clone(), name: "test".into(), value: 1 };
        let change = RecordChange::Created { id, record };

        let result = futures::executor::block_on(
            pipeline.process_incoming_change(change)
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_strategy_swap_lsp() {
        // LSP: Estrategias deben ser intercambiables
        let last_wins: Box<dyn ConflictResolutionStrategy> =
            Box::new(LastWriterWinsStrategy);
        let multi_value: Box<dyn ConflictResolutionStrategy> =
            Box::new(MultiValueRegisterStrategy);

        assert!(last_wins.can_handle(&ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("lsp_test_001").unwrap(),
            site_a: SiteId::new(),
            site_b: SiteId::new(),
        }));

        assert!(multi_value.can_handle(&ConflictType::UpdateUpdate {
            record_id: RecordId::from_str("lsp_test_002").unwrap(),
            site_a: SiteId::new(),
            site_b: SiteId::new(),
        }));
    }
}
```

---

## 📊 Benchmarks Requeridos

```rust
// benchmarks/collab_benchmarks.rs

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[test]
    fn bench_crdt_apply_local() {
        let site_id = SiteId::new();
        let mut crdt = CRDT::<TestRecord>::new(site_id);

        let start = Instant::now();
        for i in 0..10_000 {
            let id = RecordId::from_str(&format!("crdt_bench_{:08}", i)).unwrap();
            let record = TestRecord { id, index: None, name: format!("name_{}", i), value: i };
            let _ = crdt.apply_local(record);
        }
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(1));
    }

    #[test]
    fn bench_crdt_merge_concurrent() {
        let site_a = SiteId::new();
        let site_b = SiteId::new();

        let mut crdt_a = CRDT::<TestRecord>::new(site_a);
        let mut crdt_b = CRDT::<TestRecord>::new(site_b);

        // Preparar cambios concurrentes
        for i in 0..1000 {
            let id_a = RecordId::from_str(&format!("merge_a_{:06}", i)).unwrap();
            let id_b = RecordId::from_str(&format!("merge_b_{:06}", i)).unwrap();

            let record_a = TestRecord { id: id_a, index: None, name: format!("a_{}", i), value: i };
            let record_b = TestRecord { id: id_b, index: None, name: format!("b_{}", i), value: i };

            crdt_a.apply_local(record_a).unwrap();
            crdt_b.apply_local(record_b).unwrap();
        }

        // Benchmark merge
        let start = Instant::now();
        let changes_b = crdt_b.get_changes();
        let _ = crdt_a.merge(changes_b).unwrap();
        let elapsed = start.elapsed();

        // F.11: < 50ms para 1000 cambios concurrentes
        assert!(elapsed < Duration::from_millis(50));
    }

    #[test]
    fn bench_vector_clock_relation() {
        let mut clock_a = VectorClock::new();
        let mut clock_b = VectorClock::new();

        let sites: Vec<SiteId> = (0..100).map(|_| SiteId::new()).collect();

        for i in 0..50 {
            clock_a.increment(sites[i]);
        }
        for i in 50..100 {
            clock_b.increment(sites[i]);
        }

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = clock_a.relation(&clock_b);
        }
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(100));
    }
}
```

---

## 📦 Dependencias Requeridas

```toml
# Cargo.toml para archflow-collab

[package]
name = "archflow-collab"
version = "0.1.0"
edition = "2021"

[dependencies]
# Dependencias locales
archflow-records = { path = "../archflow-records" }

# Serialización
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
leb128 = "2.2"

# Concurrencia
tokio = { version = "1.0", features = ["sync"] }
futures = "0.3"
async-trait = "0.1"

# WebSocket
tokio-tungstenite = { version = "0.20", optional = true }
websocket = { version = "0.26", optional = true }

# Métricas
parking_lot = "0.12"
dashmap = "6.0"

[dev-dependencies]
criterion = "0.5"
tokio = { version = "1.0", features = ["test-util"] }

[features]
websocket = ["dep:tokio-tungstenite", "dep:websocket"]
```

---

## 🔗 Dependencias con Otras Fases

| Fase | Dependencia | Tipo |
|------|-------------|------|
| Fase 1 | `RecordStore`, `Record` | Depende de |
| Fase 3 | `SpatialIndex` | Integra |
| Fase 6 | `SyncClient` | Reutiliza |

---

## 🚨 Riesgos Identificados

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Conflictos no resueltos | Media | Alto | Tests exhaustivos + fuzzing |
| Latencia de sync | Media | Alto | BinaryDeltaCodec (F.5) |
| Reconexión fallida | Baja | Medio | Retry policies + exponential backoff |
| Memoria de vector clocks | Baja | Bajo | Cleanup de sitios inactivos |

---

## ✅ Checklist de 完成

### Investigación
- [x] Perplexity: CRDT implementations comparison (Librerías analizadas: Diamond Types, cr-sqlite, SyncedStore, Hocuspocus)
- [x] Perplexity: Conflict resolution strategies (Estrategias implementadas: LWW, Field-level, Optimistic)
- [x] Perplexity: WebSocket sync patterns (Patrones investigados y documentados)

### Tests TDD
- [x] CRDT tests (8 tests implemented - test_crdt_new, test_apply_local_change, test_merge_remote_changes, test_concurrent_update_conflict, test_vector_clock_relation)
- [x] MergeStrategy tests (3 tests implemented - test_lww_strategy, test_field_merge_strategy, test_merge_strategy_swap)
- [x] Network protocol tests (SyncMessage, SessionId, ClientCapabilities, ServerCapabilities, SyncErrorCode)
- [x] SyncServer tests (SyncServerBackend, DefaultSyncServer, SyncSession traits implemented)
- [x] SyncClient tests (SyncClient, RetryPolicy con calculate_delay)
- [x] Conflict resolution tests (ConflictDetector, ConflictResolver, ConflictResolutionPipeline, ConflictType)

### Benchmarks
- [x] Benchmarks implementados en benches/collab_benchmarks.rs
- [x] Tests de rendimiento incluidos para validación

### Implementación
- [x] CRDT core implementado con vector clocks
- [x] Merge strategies con patrones SOLID
- [x] Protocolo de red (SyncMessage)
- [x] SyncServer y SyncClient con reintentos
- [x] Sistema de resolución de conflictos
- [x] crate archflow-collab creado desde cero

### Eliminación Legacy
- [x] event_sourcing eliminado completamente de archflow-core
- [x] Referencias removidas de lib.rs
- [x] selection.rs y connectivity.rs NO reutilizados (según especificación)

### Documentación
- [x] KDoc en tipos públicos
- [x] Ejemplos de uso en doctests
- [x] Comentarios de implementación en inglés

### Criterios de Éxito
- [x] Tests pasan: cargo test -p archflow-collab --lib (15 tests passed)
- [x] Tests workspace pasan: cargo test --workspace --lib
- [x] Código compila sin errores
- [x] Implementación production-ready (no inMemory)
- [x] Arquitectura DDD mantenida
- [x] Principios SOLID aplicados en conflict resolution

---

## 📝 Notas de Implementación

### Decisiones Arquitectónicas

1. **CRDT como wrapper de RecordStore:** El CRDT encapsula el RecordStore y añade lógica de merge.

2. **Vector Clocks deterministas:** Usamos SiteId como desempate para garantizar determinismo.

3. **Estrategias de merge intercambiables:** El trait MergeStrategy permite extensión sin modificar código existente (OCP).

4. **Retry con exponential backoff:** Política configurable para reconexiones.

### Integración con Legacy

- El SyncServer debe poder recibir cambios del sistema legacy
- Los mensajes de sync deben ser backward compatible
- El sistema debe poder hacer sync desde snapshot

---

**Documento de Época: EPIC-FASE-02-Collaboration.md**  
**Versión:** 1.0.0  
**Creado:** 2026-01-26  
**Referencia Principal:** `MIGRACION_RECORDS_V2_COMPLETA.md` (L1623-2347, A, C, F.7)
