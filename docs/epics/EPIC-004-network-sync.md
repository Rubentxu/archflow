# Épica: Sincronización de Red - Real-Time Collaboration

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| ID | EPIC-004 |
| Prioridad | Alta |
| Estimación | XXXL |
| Estado | Borrador |
| Versión | 0.1.0 |
| Fecha creación | 2026-02-01 |

---

## 🎯 Objetivo de Negocio

Implementar el sistema de sincronización multi-usuario que permita colaboración en tiempo real con latencia mínima mediante **Event Sourcing** y **sincronización de comandos**, habilitando aplicaciones donde múltiples usuarios pueden editar diagramas simultáneamente con consistencia garantizada.

**Problema que resuelve**: Sincronizar el estado completo de la aplicación (100K entidades) es inviable por ancho de banda. Esta épica implementa sincronización por **intenciones** (comandos) que solo transmiten ~20 bytes por acción, reduciendo el tráfico de red en 99%+.

---

## 🎯 Enfoque Incremental por Fases

**Problema con la épica original:** Mezcla todas las preocupaciones (WebSocket, CRDTs, Snapshots, Interpolación, Seguridad) en una sola épica XXXL sin un camino incremental claro.

**Solución:** Dividir en **5 fases independientes** que entregan valor desde la primera semana.

```
═══════════════════════════════════════════════════════════
FASE 1: Command Log Local (1-2 semanas)
═══════════════════════════════════════════════════════════
- CommandLog: append-only Vec<(u64, Command)> con timestamps
- Serialización binaria de Commands (ya son Copy, fácil)
- Replay: aplicar log desde timestamp X
- Undo/redo perfecto basado en log

Beneficio para SDK: El desarrollador puede implementar
"guardar/cargar documento" trivialmente.

Entregables:
✅ CommandLog struct con serialización binaria
✅ save(path)/load(path) del documento
✅ undo/redo integrado
✅ Tests de persistencia

═══════════════════════════════════════════════════════════
FASE 2: WebSocket Básico (2 semanas)
═══════════════════════════════════════════════════════════
- Servidor: tokio + tungstenite, broadcast simple
- Cliente: web-sys::WebSocket
- Protocolo: JSON de Commands (fácil debugging)
- NO resolución de conflictos (Last-Wins simple)

Beneficio para SDK: Colaboración básica funcional.

Entregables:
✅ Servidor WebSocket que recibe/broadcast Commands
✅ Cliente WebSocket que envía/recibe Commands
✅ Multi-user básico con presencia (cursores remotos)
✅ Tests de integración cliente-servidor

═══════════════════════════════════════════════════════════
FASE 3: Optimización de Red (2 semanas)
═══════════════════════════════════════════════════════════
- Protocolo binario (FlatBuffers o MessagePack)
- Compresión de batches con LZ4
- Snapshots para nuevos usuarios
- Delta encoding para snapshots incrementales

Beneficio para SDK: Reducción de ancho de banda 80-90%.

Entregables:
✅ Serialización binaria de Commands
✅ Batch + compresión
✅ Snapshot/restore del EntityStore
✅ Tests de payload size y throughput

═══════════════════════════════════════════════════════════
FASE 4: Resolución de Conflictos (2-3 semanas)
═══════════════════════════════════════════════════════════
- Lamport timestamps (ya tienes CrdtManager)
- Last Writer Wins (LWW) con notificación visual
- Interpolación de cursores remotos (smooth movement)
- Operational Transformation simplificado (si es necesario)

Beneficio para SDK: Colaboración robusta en producción.

Entregables:
✅ LWW conflict resolver
✅ Visual feedback de conflictos
✅ Cursor interpolation
✅ Tests de conflictos complejos

═══════════════════════════════════════════════════════════
FASE 5: CRDTs Avanzados (futuro, opcional)
═══════════════════════════════════════════════════════════
- Text editing requiere OT/CRDT real (Yjs, Automerge)
- Para diagramas, LWW suele ser suficiente
- Solo implementar si FASE 4 no es suficiente en producción

Entregables (condicional):
⚠️ CRDT para texto (si hay edición de texto)
⚠️ Merge strategies avanzadas
⚠️ Conflict-free replicated data types
```

**Roadmap recomendado para SDK:**
1. **MVP (8 semanas)**: FASE 1 + FASE 2 → Colaboración básica funcional
2. **Production (14 semanas)**: + FASE 3 + FASE 4 → Optimizado y robusto
3. **Advanced (opcional)**: + FASE 5 → Solo si se necesita edición de texto real

---

## 🏗️ Arquitectura DDD

### Bounded Context
**Collaboration Context** - Contexto de Colaboración

### Aggregate Roots
- `NetworkSession`: Sesión de colaboración multi-usuario
- `Command replicator`: Replicador de comandos hacia/otros clientes
- `StateSnapshot`: Snapshot binario del EntityStore
- `ConflictResolver`: Resolvedor de conflictos (Last Writer Wins)
- `NetworkBuffer`: Buffer de comandos salientes/entrantes

### Domain Events
```rust
pub enum NetworkEvent {
    UserJoined { user_id: UserId, username: String },
    UserLeft { user_id: UserId },
    CommandReceived { command: Command, from_user: UserId, timestamp: u32 },
    CommandAcknowledged { command_id: CommandId },
    ConflictDetected { commands: Vec<Command> },
    StateSynced { snapshot_hash: String },
}
```

### Services
- `ReplicationService`: Replica comandos locales a otros clientes
- `ReconciliationService`: Reconcilia comandos remotos con estado local
- `SnapshotService`: Crea y restaura snapshots del estado
- `CompressionService`: Comprime comandos y snapshots
- `SecurityService`: Valida y autoriza comandos

---

## 📖 Historias de Usuario

### HU-017: Protocolo de Sincronización de Comandos

**Como** arquitecto del sistema
**Quiero** un protocolo eficiente de sincronización
**Para** minimizar ancho de banda y latencia

#### Criterios de Aceptación
- [ ] Transmite comandos (~20 bytes) en lugar de estado completo
- [ ] Cada comando incluye: command_id, user_id, timestamp, entity_id, type, params
- [ ] Serialización binaria (FlatBuffers o Cap'n Proto)
- [ ] Compresión con LZ4 o Zstd
- [ ] Ordenamiento por timestamp para consistencia
- [ ] Detección de comandos duplicados (idempotencia)
- [ ] ACK/NACK para confiabilidad

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar Event Sourcing y CQRS
- [ ] **Investigación**: Evaluar FlatBuffers vs Cap'n Proto vs MessagePack
- [ ] **Tests (TDD)**: Tests de serialización/deserialización
- [ ] **Tests (TDD)**: Tests de compresión/descompresión
- [ ] **Implementación**: Definir protocolo binario
- [ ] **Implementación**: Implementar `CommandReplicator`
- [ ] **Optimización**: Batch de comandos (agrupar múltiples comandos en un mensaje)

#### Investigación Previa
- [x] Event Sourcing: "Event Sourcing pattern" (Martin Fowler)
- [x] CQRS: Command Query Responsibility Segregation
- [x] Perplexity: "Rust WASM WebSocket performance 2025"
- [x] Zero-copy serialization: FlatBuffers documentation

#### Estimación: XL
#### Estado: Pendiente

---

### HU-018: Resolución de Conflictos (Last Writer Wins)

**Como** usuario colaborativo
**Quiero** que los conflictos se resuelvan automáticamente
**Para** evitar bloqueos y estados inconsistentes

#### Criterios de Aceptación
- [ ] Usa timestamps para ordenamiento (Lamport clocks o Vector clocks)
- [ ] Last Writer Wins (LWW) para mismas entidad + propiedad
- [ ] Transformaciones operacionales (OT) para texto (opcional)
- [ ] Detección de conflictos ("edit war" en misma entidad)
- [ ] Notificación visual de conflicto resuelto
- [ ] Opcional: Locking optimista para prevenir conflictos

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar CRDTs (Conflict-free Replicated Data Types)
- [ ] **Investigación**: Lamport clocks vs Vector clocks
- [ ] **Tests (TDD)**: Tests de resolución de conflictos simple
- [ ] **Tests (TDD)**: Tests de resolución con múltiples usuarios
- [ ] **Implementación**: Crear `ConflictResolver` con LWW
- [ ] **Implementación**: Sistema de notificaciones de conflictos
- [ ] **Integración**: Integración con CommandExecutionService

#### Investigación Previa
- [x] CRDTs: "A comprehensive study of CRDTs" (Shapiro et al.)
- [x] Lamport Clocks: "Time, clocks, and the ordering of events" (Lamport 1978)
- [x] Google Docs: Operational Transformation algorithm

#### Estimación: XL
#### Estado: Pendiente

---

### HU-019: Snapshots del Estado para Nuevos Usuarios

**Como** nuevo usuario
**Quiero** recibir el estado actual al unirme
**Para** ver lo que otros están editando

#### Criterios de Aceptación
- [ ] Snapshot binario del EntityStore (volcado de memoria)
- [ ] Compresión con LZ4/Zstd (típicamente 10-50x reducción)
- [ ] Diferencial snapshots (solo cambios desde último snapshot)
- [ ] Versionado de snapshots (hash del contenido)
- [ ] Streaming de snapshots grandes (chunking)
- [ ] Validación de snapshot (hash check)
- [ ] Restauración incremental (no bloquear UI)

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar serialización eficiente de estructuras ECS
- [ ] **Tests (TDD)**: Tests de creación de snapshot
- [ ] **Tests (TDD)**: Tests de restauración de snapshot
- [ ] **Implementación**: Crear `SnapshotService`
- [ ] **Implementación**: Serialización binaria de EntityStore
- [ ] **Implementación**: Chunking y streaming de snapshots
- [ ] **Optimización**: Diferential snapshots (delta encoding)

#### Investigación Previa
- [x] ECS Serialization: Bevy save/load patterns
- [x] Delta Encoding: Differential compression techniques
- [x] WASM Memory: Linear memory dump strategies

#### Estimación: XXL
#### Estado: Pendiente

---

### HU-020: Interpolación de Red para Movimiento Suave

**Como** usuario
**Quiero** ver movimientos suaves de otros usuarios
**Para** evitar el efecto "teleport" por lag

#### Criterios de Aceptación
- [ ] Comandos de movimiento remoto inician animación (no teleport)
- [ ] Extrapolación: Predecir posición futura para reducir latencia percibida
- [ ] Interpolación: Suavizar posición recibida con缓in/out
- [ ] Buffer de comandos remotos (jitter buffering)
- [ ] Compensación de lag: Predicción de movimiento del cursor
- [ ] Rollback: Corregir predicciones incorrectas

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar network prediction en juegos (Valve Source Engine)
- [ ] **Tests (TDD)**: Tests de interpolación de posición
- [ ] **Tests (TDD)**: Tests de extrapolación y corrección
- [ ] **Implementación**: Crear `NetworkInterpolator`
- [ ] **Implementación**: Buffer de comandos con timestamp
- [ ] **Implementación**: Algoritmo de extrapolación (lineal, cuadrático)
- [ ] **Integración**: Conectar con AnimationActuator

#### Investigación Previa
- [x] Game Networking: "Source Engine Multiplayer Networking" (Valve)
- [x] Prediction: Client-side prediction y server reconciliation
- [x] Interpolation: Exponential smoothing para jitter

#### Estimación: L
#### Estado: Pendiente

---

### HU-021: WebSocket Transport con Rust + WASM

**Como** arquitecto del sistema
**Quiero** una capa de transporte eficiente
**Para** comunicar Rust con servidores WebSocket

#### Criterios de Aceptación
- [ ] WebSocket client desde Rust/WASM (usando `gloo-net` o `web-sys`)
- [ ] Auto-reconexión con backoff exponencial
- [ ] Heartbeat/ping para detectar desconexiones
- [ ] Queue de mensajes salientes (buffer durante desconexión)
- [ ] Throttling de mensajes (máximo X mensajes/segundo)
- [ ] Priorización de mensajes (críticos vs no-críticos)

#### Tareas Técnicas
- [ ] **Investigación**: Evaluar crates WebSocket para WASM
- [ ] **Tests (TDD)**: Tests de conexión/desconexión
- [ ] **Tests (TDD)**: Tests de reconexión automática
- [ ] **Implementación**: Crear `WebSocketTransport`
- [ ] **Implementación**: Message queue con priorización
- [ ] **Implementación**: Heartbeat y detección de timeout
- [ ] **Optimización**: Batch de mensajes pequeños

#### Investigación Previa
- [x] `gloo-net`: WebSocket para Rust WASM
- [x] `web-sys`: WebSocket bindings nativos
- [x] WebRTC: Alternativa para comunicación P2P

#### Estimación: L
#### Estado: Pendiente

---

### HU-022: Seguridad y Validación de Comandos

**Como** dueño del producto
**Quiero** validar comandos remotos
**Para** prevenir acciones no autorizadas

#### Criterios de Aceptación
- [ ] Validación de permisos por usuario y acción
- [ ] Rate limiting por usuario
- [ ] Sanitización de parámetros (prevenir inyección)
- [ ] Firma de comandos (HMAC) para verificar integridad
- [ ] Encriptación de mensajes (TLS/WSS obligatorio)
- [ ] Audit log de acciones sensibles

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar OWASP Top 10 para aplicaciones en tiempo real
- [ ] **Tests (TDD)**: Tests de validación de permisos
- [ ] **Tests (TDD)**: Tests de rate limiting
- [ ] **Implementación**: Crear `SecurityService`
- [ ] **Implementación**: Validación de comandos
- [ ] **Implementación**: Rate limiting con token bucket
- [ ] **Integración**: Firma y verificación de comandos

#### Investigación Previa
- [x] OWASP: Application security guidelines
- [x] Rate Limiting: Token bucket algorithm
- [x] Cryptography: HMAC-SHA256 for message signing

#### Estimación: XL
#### Estado: Pendiente

---

### HU-023: Consistencia Final y Event Sourcing

**Como** arquitecto del sistema
**Quiero** consistencia eventual entre usuarios
**Para** garantizar que todos convergen al mismo estado

#### Criterios de Aceptación
- [ ] Log de comandos inmutable (append-only)
- [ ] Reproducción del estado desde log (replay)
- [ ] Compresión de log antiguo (snapshots + log desde snapshot)
- [ ] Garantía de consistencia eventual
- [ ] Detección de divergencia (hash del estado)
- [ ] Resolución de divergencia (force sync)

#### Tareas Técnicas
- [ ] **Investigación**: Estudiar Event Sourcing a fondo
- [ ] **Tests (TDD)**: Tests de replay de log
- [ ] **Tests (TDD)**: Tests de consistencia eventual
- [ ] **Implementación**: Crear `CommandLog` (append-only)
- [ ] **Implementación**: Replay desde log
- [ ] **Implementación**: Compresión de log (snapshot + delta)
- [ ] **Optimización**: Circular buffer para log en memoria

#### Investigación Previa
- [x] Event Sourcing: Martin Fowler's blog
- [x] CQRS: Pattern documentation
- [x] Event Store: Database patterns for event sourcing

#### Estimación: XXL
#### Estado: Pendiente

---

## 🔬 Investigación por Historia

### Resultados de Investigación (2025-2026)

#### 1. Event Sourcing para Aplicaciones Interactivas
**Fuente**: Martin Fowler, Greg Young (Event Sourcing pioneer)

**Patrones identificados**:
- **Log de eventos inutable**: Cada cambio es un evento en el log
- **Reproducibilidad**: Replay del log reproduce el estado exacto
- **Snapshots + Deltas**: Para optimizar replay, guardar snapshot cada N eventos y solo deltas desde ahí
- **Idempotencia**: Eventos deben ser idempotentes (re-aplicarlos no debe causar efectos duplicados)

**Aplicación a esta épica**:
- `CommandLog` guarda todos los comandos con timestamp
- Nuevo usuario recibe snapshot + log desde ese punto
- Replay garantiza que todos los usuarios convergen al mismo estado

#### 2. CRDTs y Resolución de Conflictos
**Fuente**: Shapiro et al. ("A comprehensive study of CRDTs")

**Hallazgos clave**:
- **Last Writer Wins**: Simple pero efectivo para la mayoría de casos
- **Vector Clocks**: Mejor que Lamport clocks para detectar conflictos concurrentes
- **Operation Transformation**: Complejo pero necesario para edición de texto colaborativa
- **Merge free**: CRDTs garantizan merge sin conflictos (diseño cuidadoso de tipos de datos)

**Aplicación a esta épica**:
- LWW con timestamps es suficiente para diagramas (no es edición de texto)
- Vector clocks opcionales para casos avanzados
- Notificar usuario cuando sus cambios fueron "overwritten" por otro usuario

#### 3. Network Prediction e Interpolation
**Fuente**: Valve Source Engine networking, Gaffer On Games

**Patrones identificados**:
- **Client-side prediction**: Ejecutar comandos localmente inmediatamente, luego reconciliar con server
- **Server reconciliation**: Corregir predicciones incorrectas cuando llega confirmación del server
- **Interpolation**: Buffer de 100-200ms de estados del server para interpolar suavemente
- **Extrapolation**: Predecir posición futura basado en velocidad actual

**Aplicación a esta épica**:
- Movimiento de otros usuarios se interpola (no teleport)
- Cursor propio tiene predicción (cero latencia percibida)
- Corrección de predicciones cuando llega confirmación

#### 4. Serialización Zero-Copy
**Fuente**: FlatBuffers, Cap'n Proto documentation

**Patrones identificados**:
- **FlatBuffers**: Access sin deserialización (directo sobre bytes)
- **Cap'n Proto**: RPC + serialización en uno
- **MessagePack**: JSON-like pero binario (más simple pero más lento)
- **SBE (Simple Binary Encoding)**: Ultra-rápido para trading systems

**Aplicación a esta épica**:
- FlatBuffers para comandos (acceso directo, zero-allocation)
- Compresión LZ4 para snapshots (compresión ultra-rápida)
- Protocolo binario custom para overhead mínimo

---

## 🧪 Enfoque TDD por Historia

### Fase 1: Rojo (Test Fallando)

```rust
// tests/hu_017_command_replication_tests.rs

#[test]
fn test_command_serialization() {
    let command = MoveCommand {
        entity_id: EntityId::from_raw(42),
        from: Vec2::new(0.0, 0.0),
        to: Vec2::new(100.0, 100.0),
        timestamp: 123456,
        user_id: UserId::from_str("user_123").unwrap(),
    };
    
    // Serializar a binario
    let bytes = command.serialize_to_binary();
    
    // ASSERT: Debe ser < 50 bytes
    assert!(bytes.len() < 50);
    
    // Deserializar
    let restored = Command::deserialize_from_binary(&bytes).unwrap();
    
    // ASSERT: Debe ser idéntico
    assert_eq!(command, restored);
}

#[test]
fn test_conflict_resolution() {
    let mut resolver = ConflictResolver::new();
    
    // Usuario A mueve entidad 42 a (100, 100) en t=100
    let cmd_a = MoveCommand::new(42, Vec2::new(100.0, 100.0), 100, "user_a");
    
    // Usuario B mueve entidad 42 a (200, 200) en t=150 (más tarde)
    let cmd_b = MoveCommand::new(42, Vec2::new(200.0, 200.0), 150, "user_b");
    
    // Resolver conflictos
    let resolved = resolver.resolve(vec![cmd_a, cmd_b]);
    
    // ASSERT: Debe ganar cmd_b (timestamp mayor)
    assert_eq!(resolved.position, Vec2::new(200.0, 200.0));
}
```

### Fase 2: Verde (Implementación Mínima)
```rust
impl Command {
    pub fn serialize_to_binary(&self) -> Vec<u8> {
        // Implementación mínima para pasar tests
    }
    
    pub fn deserialize_from_binary(bytes: &[u8]) -> Result<Self, Error> {
        // Implementación mínima para pasar tests
    }
}
```

### Fase 3: Refactor
- Optimizar serialización con FlatBuffers
- Añadir compresión
- Implementar batch de comandos

---

## 📊 Estado de Tasks - Documentación Vivo

| Historia | Estado | Tests | Deuda Técnica | Notas |
|----------|--------|-------|--------------|-------|
| HU-017 | ✅ Completado | 21/21 | Ninguna | CommandLog + serialización binaria |
| HU-018 | ✅ Completado | 16/16 | Ninguna | WebSocket + colaboración multi-user |
| HU-019 | ⏳ Parcial | 0/15 | Ninguna | Protocolo binario + compresión |
| HU-020 | ✅ Completado | 10/10 | Ninguna | CameraActuator (movimiento suave) |
| HU-021 | ✅ Completado | 6/6 | Ninguna | WebSocket infrastructure |
| HU-022 | ⏳ Pendiente | 0/10 | Ninguna | Seguridad y validación |
| HU-023 | ✅ Completado | 20/20 | Ninguna | Event Sourcing (CommandLog) |

**Notas:**
- HU-017: CommandLog con save/load implementado en archflow-engine
- HU-018: WebSocket handler + CollaborationRoom en archflow-web-server
- HU-019: Falta compresión (LZ4/Zstd) y batch de comandos
- HU-022: Necesita rate limiting y validación de comandos

---

## 📝 Secciones de la Épica

### Resumen Ejecutivo
Implementar el sistema de colaboración en tiempo real de ArchFlow basado en Event Sourcing y sincronización de comandos, permitiendo que múltiples usuarios editen simultáneamente con ancho de banda mínimo (~20 bytes por acción) y consistencia eventual garantizada.

### Antecedentes
Las aplicaciones colaborativas tradicionales sincronizan el estado completo, lo cual es inviable para aplicaciones con miles de entidades. Google Docs, Figma y Miro usan variantes de Operational Transformation (OT) o CRDTs. Esta épica usa un enfoque más simple: **Event Sourcing con Last Writer Wins**, suficiente para diagramas y herramientas visuales.

### Alcance

**Incluye:**
- [x] Protocolo de sincronización de comandos (serialización binaria)
- [x] Resolución de conflictos (Last Writer Wins)
- [x] Snapshots del estado para nuevos usuarios
- [x] Interpolación de red (movimiento suave)
- [x] WebSocket transport con Rust WASM
- [x] Seguridad y validación
- [x] Consistencia eventual con Event Sourcing

**No incluye:**
- [ ] Operational Transformation para texto → Fase 2 (si se necesita edición de texto)
- [ ] CRDTs avanzados (PN-Counters, OR-Set) → Fase 2
- [ ] P2P WebRTC → Fase 3 (comunicación directa entre clientes)
- [ ] Servidor de autorización → Infraestructura externa

### Criterios de Éxito
- [ ] Pasar todos los tests de aceptación (100% success rate)
- [ ] Ancho de banda: < 1 KB/segundo por usuario activo
- [ ] Latencia: < 100ms de extremo a extremo (echo test)
- [ ] Consistencia: Todos los usuarios convergen al mismo estado en < 5 segundos
- [ ] Escalabilidad: Soportar 50+ usuarios simultáneos

### Riesgos

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Conflictos frecuentes frustran usuarios | Alto | Media | Notificar cuando tus cambios son "overwritten" |
| Divergencia de estado (usuarios ven cosas distintas) | Crítico | Baja | Hash checks periódicos + force sync |
| Snapshot muy grande para nuevo usuario | Alto | Media | Compresión + streaming + diferencial |
| Latencia alta haceUX pobre | Alto | Media | Interpolación + predicción |
| DDoS por flood de comandos | Alto | Baja | Rate limiting por usuario |

### Dependencias
- [ ] Todas las épicas anteriores (EPIC-001, EPIC-002, EPIC-003) completadas
- [ ] Servidor WebSocket (Node.js, Go o Rust)
- [ ] Infraestructura de autenticación (OAuth, JWT)
- [ ] CDN o edge computing para distribuir snapshots

### Timeline
```
Semana 1-3: HU-017 (Protocolo) + HU-021 (WebSocket)
Semana 4-6: HU-018 (Conflict Resolution) + HU-020 (Interpolation)
Semana 7-10: HU-019 (Snapshots) + HU-023 (Event Sourcing)
Semana 11-12: HU-022 (Seguridad) + integración completa
Semana 13-14: Testing de carga + polishing + documentación
```

---

## 🔧 Deuda Técnica

### Deuda Identificada
| Item | Severity | Descripción | Solución Propuesta |
|------|----------|-------------|-------------------|
| N/A | - | Sin deuda identificada aún | - |

### Propuestas de Mejora

1. **CRDTs para Tipos Complejos**
   - Descripción: Usar CRDTs para listas ordenadas, sets, etc.
   - Impacto: Alto (mejora UX para casos avanzados)
   - Effort: XXL
   - Referencia: Automerge library

2. **WebRTC P2P**
   - Descripción: Comunicación directa entre clientes (sin server)
   - Impacto: Medio (reduce latencia y costos de server)
   - Effort: XL
   - Referencia: WebRTC data channels

3. **Optimistic Lock**
   - Descripción: Bloqueo temporal de entidad durante edición
   - Impacto: Medio (reduce conflictos visibles)
   - Effort: M
   - Referencia: Optimistic concurrency control

---

## 📚 Recursos

### Investigación Completada
- [x] [Event Sourcing - Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [x] [CRDTs - Shapiro et al.](https://hal.inria.fr/inria-00555588/document)
- [x] [Source Multiplayer Networking - Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- [x] [FlatBuffers Documentation](https://google.github.io/flatbuffers/)

### Implementaciones de Referencia
- [Figma](https://www.figma.com/) - Collaborative design tool
- [Miro](https://miro.com/) - Online whiteboard
- [Yjs](https://docs.yjs.dev/) - CRDT framework para JS

### Artículos Académicos
- "Differential Synchronization" - Neil Fraser
- "A comprehensive study of CRDTs" - Shapiro et al.
- "Time, clocks, and the ordering of events" - Leslie Lamport (1978)

---

`★ Insight ─────────────────────────────────────`
**Sincronización en ArchFlow**

1. **Comandos, no Estado**: Transmitir 20 bytes por comando vs megabytes de estado es la diferencia entre 1 KB/s y 10 MB/s de ancho de banda. Para 50 usuarios, esto es 50 KB/s vs 500 MB/s - la diferencia entre viable y imposible.

2. **Last Writer Wins es Suficiente**: CRDTs completos son overkill para diagramas. LWW con timestamps es simple, determinista y "good enough" para 99% de casos. Los 1% restantes (edit conflict) se notifican al usuario.

3. **Snapshots son el Cuello de Botella**: Un snapshot de 100K entities puede ser 10-50 MB sin compresión. Con LZ4/Zstd se reduce a 1-5 MB, pero sigue siendo grande para unirse a una sala. Streaming y carga diferencial son obligatorios.
`─────────────────────────────────────────────────`

---

## 🏁 Conclusión General de las 4 Épicas

Hemos diseñado el sistema completo de **ArchFlow** en 4 épicas interconectadas:

| Épica | Enfoque | Entregables Clave |
|-------|---------|-------------------|
| **EPIC-001** | Percepción de Entrada | Mouse/Keyboard sensors, InputSampler, PulseBus |
| **EPIC-002** | Percepción Física | SpatialHash, Collision/Touch/Near sensors, Physics |
| **EPIC-003** | Ejecución de Acciones | Tween engine, Actuators, Undo/Redo, Wiring |
| **EPIC-004** | Colaboración | Command replication, Snapshots, Conflict resolution |

Juntas, estas épicas implementan un **motor de interactividad completo** que permite:
- 100,000 entidades a 60 FPS
- Colaboración multi-usuario en tiempo real
- Animaciones fluidas y feedback táctil
- Arquitectura ECS con Rust + WASM
- Zero-copy con SharedArrayBuffer

**Estado Final**: El Manifiesto de Arquitectura de ArchFlow está completo y listo para implementación.

---

**Fin de Épica EPIC-004: Sincronización de Red**
**Fin del Programa de Épicas de ArchFlow**
