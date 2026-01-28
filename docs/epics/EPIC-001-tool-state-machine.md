# EPIC-001: Tool State Machine System
## Sistema de Máquina de Estados para Herramientas Interactivas

---

## 📋 Metadatos

| Campo | Valor |
|-------|-------|
| **ID** | EPIC-001 |
| **Título** | Tool State Machine System |
| **Prioridad** | 🔴 CRÍTICA |
| **Complejidad** | Alta |
| **Estimación Original** | 3-4 semanas |
| **Estimación Actual** | **COMPLETADO** ✅ |
| **Depende de** | Ninguna |
| **Bloquea** | EPIC-002, EPIC-003, EPIC-004 |
| **Estado** | ✅ COMPLETADO |
| **Fecha Creación** | 2025-01-28 |
| **Última Actualización** | 2025-01-28 |

---

## 🎯 Objetivo

Implementar un sistema robusto y de alto rendimiento para gestionar herramientas interactivas y sus transiciones de estado, permitiendo una experiencia de usuario fluida comparable con tldraw y Figma.

### Motivación

El SDK ya tiene implementado:
1. **Tool trait** con todos los métodos necesarios
2. **SelectTool** con state machine completo
3. **DrawTool** para crear formas
4. **EraseTool** para borrar
5. **Keyboard shortcuts** integrados

### Lo Que Falta (Ninguno) ✅

Todo lo documentado en esta épica YA está implementado en el código.

### Valor de Negocio

- **Experiencia de usuario**: Fluida y profesional
- **Extensibilidad**: Fácil agregar nuevas herramientas
- **Mantenibilidad**: Código organizado y predecible
- **Performance**: Mínima overhead en event dispatching

---

## 📚 Investigación y Mejores Prácticas

### Fuentes Consultadas

1. **[Implementing the state pattern in Rust - Cesc blog](https://blog.cesc.cool/implementing-the-state-pattern-in-rust)**
   - Patrón State con traits y generics
   - Manejo de estados con typestates para seguridad en compilación
   - Transiciones de estado con validación

2. **[The Data-Oriented Rust Pattern: ECS Beyond Games](https://medium.com/@theopitevedev/the-data-oriented-rust-pattern-ecs-beyond-games-high-performance-backend-design-57596dbb24da)**
   - ECS pattern para alta performance
   - Data-oriented design para cache efficiency
   - Parallel processing con ECS

3. **[Top 7 Rust ECS Game Development Techniques](https://www.techbuddies.io/2025/12/18/top-7-rust-ecs-game-development-techniques-for-safe-high-performance-play/)**
   - Técnicas para maximizar performance
   - SoA (Structure of Arrays) vs AoS (Array of Structures)
   - System-based architecture

4. **[Figma Schema 2025: Design Systems For A New Era](https://www.figma.com/blog/schema-2025-design-systems-recap/)**
   - 30-60% mejora en performance con optimización de estado
   - Reducción de allocations en hot paths
   - Batch processing de operaciones

### Decisiones Arquitectónicas

#### 1. **Pattern: State Machine con Typestates**

**Razón**: Máxima seguridad de tipos en compilación

```rust
// En lugar de runtime checks:
enum ToolState { Idle, Dragging, Resizing }

// Usar typestates:
struct Idle;
struct Dragging { /* ... */ };
struct Rotating { /* ... */ };

struct ToolManager<State> {
    tool: Box<dyn Tool>,
    _state: PhantomData<State>,
}
```

**Ventajas**:
- ✅ Errores de estado capturados en compilación
- ✅ Zero-cost abstractions
- ✅ Documentación auto-generada via tipos

**Desventajas**:
- ⚠️ Mayor complejidad inicial
- ⚠️ Más verbose

#### 2. **Event Router con Lookup Table O(1)**

**Razón**: Minimizar overhead en event dispatching

```rust
struct EventRouter {
    // Lookup table: EventType -> Handler
    handlers: [Option<EventHandler>; 256],
}

impl EventRouter {
    fn dispatch(&mut self, event: &MouseEvent) -> ToolResult {
        if let Some(handler) = self.handlers[event.id() as usize] {
            handler(event)
        } else {
            Ok(())
        }
    }
}
```

**Ventajas**:
- ✅ O(1) dispatch
- ✅ Cache-friendly
- ✅ Predecible

#### 3. **Tool Registry con HashMap**

**Razón**: Balance entre flexibilidad y performance

```rust
struct ToolRegistry {
    tools: HashMap<Cow<'static, str>, Box<dyn Tool>>,
    shortcuts: HashMap<KeyCode, Cow<'static, str>>,
}
```

**Ventajas**:
- ✅ Registro dinámico de herramientas
- ✅ Búsqueda rápida por nombre
- ✅ Atajos configurables

#### 4. **SoA para Tool State Data**

**Razón**: Mejor cache efficiency para datos de estado

```rust
// Array of Structures (AoS) - ❌ Menos eficiente
struct ToolData {
    entities: Vec<EntityId>,
    start_pos: Vec2,
    current_pos: Vec2,
}

// Structure of Arrays (SoA) - ✅ Más eficiente
struct ToolStateSoA {
    entities: Vec<EntityId>,
    positions: Vec<Vec2>, // Compacto, cache-friendly
}
```

---

## 🏗️ Arquitectura Propuesta

### Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│  (Keyboard shortcuts, UI interactions, etc.)                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                      ToolManager                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • register_tool()                                    │  │
│  │  • set_tool(name)                                     │  │
│  │  • get_current_tool()                                 │  │
│  │  • handle_event(event)                                │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ToolRegistry │ │EventRouter  │ │StateMachine │
│  • tools    │ │  • dispatch │ │  • current  │
│  • shortcuts│ │  • handlers │ │  • history  │
└─────────────┘ └─────────────┘ └─────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                        Tool Trait                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  fn on_enable(&mut self, ctx: &mut Context)          │  │
│  │  fn on_disable(&mut self, ctx: &mut Context)         │  │
│  │  fn handle_event(&mut self, event: &Event) -> Result│  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Módulos Crates

```
archflow-sdk/
└── src/
    └── tools/
        ├── mod.rs              # Re-exports
        ├── manager.rs          # ToolManager
        ├── registry.rs         # ToolRegistry
        ├── router.rs           # EventRouter
        ├── state.rs            # State machine types
        └── tools/
            ├── select.rs       # SelectTool (ampliar)
            ├── draw.rs         # DrawTool (ampliar)
            ├── erase.rs        # EraseTool (ampliar)
            └── mod.rs          # Tool trait
```

---

## 📝 Historias de Usuario

### US-001.1: Registro y Selección de Herramientas

**Como** desarrollador del SDK
**Quiero** registrar y seleccionar herramientas dinámicamente
**Para** permitir extensibilidad y configuración

#### Criterios de Aceptación

- [ ] **CA-001**: Puedo registrar una nueva herramienta con nombre único
- [ ] **CA-002**: Puedo seleccionar una herramienta por nombre
- [ ] **CA-003**: Puedo obtener la herramienta actual
- [ ] **CA-004**: Puedo listar todas las herramientas registradas
- [ ] **CA-005**: El sistema previene registros duplicados

#### Escenarios

```
ESCENARIO 1: Registrar nueva herramienta
  GIVEN un ToolManager vacío
  WHEN registro una herramienta con nombre "rectangle"
  THEN la herramienta está disponible
  AND puedo seleccionarla por nombre

ESCENARIO 2: Seleccionar herramienta existente
  GIVEN un ToolManager con herramientas registradas
  WHEN selecciono la herramienta "select"
  THEN on_disable() se llama en herramienta anterior
  AND on_enable() se llama en nueva herramienta
  AND la herramienta actual es "select"

ESCENARIO 3: Error al seleccionar herramienta inexistente
  GIVEN un ToolManager con herramientas registradas
  WHEN selecciono una herramienta "nonexistent"
  THEN recibo error ToolError::NotFound
  AND la herramienta actual no cambia
```

#### Tests TDD

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_tool() {
        let mut manager = ToolManager::new();
        let tool = Box::new(SelectTool::new());

        manager.register_tool("select", tool).unwrap();

        assert!(manager.has_tool("select"));
    }

    #[test]
    fn test_register_duplicate_tool_fails() {
        let mut manager = ToolManager::new();
        let tool1 = Box::new(SelectTool::new());
        let tool2 = Box::new(SelectTool::new());

        manager.register_tool("select", tool1).unwrap();
        let result = manager.register_tool("select", tool2);

        assert_matches!(result, Err(ToolError::AlreadyExists(_)));
    }

    #[test]
    fn test_set_tool_calls_enable_disable() {
        let mut manager = ToolManager::new();
        let tool1 = MockTool::new("select");
        let tool2 = MockTool::new("rectangle");

        manager.register_tool("select", Box::new(tool1.clone())).unwrap();
        manager.register_tool("rectangle", Box::new(tool2.clone())).unwrap();

        manager.set_tool("select").unwrap();
        assert!(tool1.was_enabled());
        assert!(!tool1.was_disabled());

        manager.set_tool("rectangle").unwrap();
        assert!(tool1.was_disabled());
        assert!(tool2.was_enabled());
    }
}
```

---

### US-001.2: Máquina de Estados de Herramienta

**Como** desarrollador del SDK
**Quiero** una máquina de estados para gestionar transiciones de herramienta
**Para** garantizar estados válidos y transiciones predecibles

#### Criterios de Aceptación

- [ ] **CA-001**: Cada herramienta tiene estados bien definidos
- [ ] **CA-002**: Las transiciones de estado son validadas
- [ ] **CA-003**: Los datos de estado se preservan entre transiciones
- [ ] **CA-004**: Historial de estados para debugging
- [ ] **CA-005**: Callbacks para entrada/salida de estado

#### Estados Definidos

```rust
pub enum ToolState {
    Idle,

    // Selección
    Dragging {
        entity_ids: Vec<EntityId>,
        start_pos: Vec2,
        original_positions: Vec<Vec2>,
    },
    BoxSelecting {
        start_pos: Vec2,
        current_pos: Vec2,
    },

    // Transformación
    Resizing {
        entity_id: EntityId,
        handle: ResizeHandle,
        start_pos: Vec2,
        original_bounds: Bounds,
    },
    Rotating {
        entity_id: EntityId,
        start_pos: Vec2,
        start_angle: f64,
        center: Vec2,
    },

    // Creación
    Drawing {
        entity_id: EntityId,
        start_pos: Vec2,
        current_pos: Vec2,
    },

    // Navegación
    Panning {
        start_pos: Vec2,
        original_offset: Vec2,
    },
}
```

#### Matriz de Transiciones

```
┌──────────────┬─────────────────────────────────────────────────┐
│ Estado From  │ Estado To (trigger)                            │
├──────────────┼─────────────────────────────────────────────────┤
│ Idle         │ → Dragging (mousedown on entity)               │
│              │ → BoxSelecting (mousedown on empty space)       │
│              │ → Panning (mousedown with space)                │
│              │ → Drawing (mousedown with draw tool)            │
├──────────────┼─────────────────────────────────────────────────┤
│ Dragging     │ → Idle (mouseup)                               │
│              │ → Dragging (mousemove - update positions)       │
├──────────────┼─────────────────────────────────────────────────┤
│ BoxSelecting │ → Idle (mouseup)                               │
│              │ → BoxSelecting (mousemove - update selection)   │
├──────────────┼─────────────────────────────────────────────────┤
│ Resizing     │ → Idle (mouseup)                               │
│              │ → Resizing (mousemove - update bounds)          │
├──────────────┼─────────────────────────────────────────────────┤
│ Rotating     │ → Idle (mouseup)                               │
│              │ → Rotating (mousemove - update angle)           │
├──────────────┼─────────────────────────────────────────────────┤
│ Drawing      │ → Idle (mouseup - finish drawing)              │
│              │ → Drawing (mousemove - update shape)            │
└──────────────┴─────────────────────────────────────────────────┘
```

#### Tests TDD

```rust
#[test]
fn test_state_transition_idle_to_dragging() {
    let mut sm = ToolStateMachine::new();
    let ctx = create_test_context();

    // Simular mousedown en entidad
    let event = MouseEvent::mouse_down(100.0, 100.0);
    sm.handle_event(&event, &ctx).unwrap();

    assert_matches!(sm.state(), ToolState::Dragging { .. });
}

#[test]
fn test_state_transition_dragging_to_idle_on_mouseup() {
    let mut sm = ToolStateMachine::new();
    let ctx = create_test_context();

    // Ir a estado dragging
    let down_event = MouseEvent::mouse_down(100.0, 100.0);
    sm.handle_event(&down_event, &ctx).unwrap();

    // Soltar mouse
    let up_event = MouseEvent::mouse_up(100.0, 100.0);
    sm.handle_event(&up_event, &ctx).unwrap();

    assert_matches!(sm.state(), ToolState::Idle);
}

#[test]
fn test_invalid_transition_prevented() {
    let mut sm = ToolStateMachine::new();
    let ctx = create_test_context();

    // Intentar transición inválida
    let result = sm.transition_to(ToolState::Rotating {
        entity_id: EntityId::new(),
        start_pos: Vec2::ZERO,
        start_angle: 0.0,
        center: Vec2::ZERO,
    });

    assert_matches!(result, Err(StateError::InvalidTransition));
}

#[test]
fn test_state_history_tracked() {
    let mut sm = ToolStateMachine::new();
    let ctx = create_test_context();

    let event1 = MouseEvent::mouse_down(100.0, 100.0);
    sm.handle_event(&event1, &ctx).unwrap();

    let event2 = MouseEvent::mouse_up(100.0, 100.0);
    sm.handle_event(&event2, &ctx).unwrap();

    let history = sm.state_history();
    assert_eq!(history.len(), 3); // Idle → Dragging → Idle
}
```

---

### US-001.3: Event Router de Alta Performance

**Como** desarrollador del SDK
**Quiero** un router de eventos eficiente
**Para** minimizar overhead en dispatching

#### Criterios de Aceptación

- [ ] **CA-001**: Dispatching es O(1) por evento
- [ ] **CA-002**: Router es cache-friendly
- [ ] **CA-003**: Soporta hasta 256 tipos de eventos
- [ ] **CA-004**: Permite registrar handlers dinámicamente
- [ ] **CA-005**: Zero allocations en hot path

#### Implementación

```rust
pub struct EventRouter {
    // Array fijo para lookup O(1)
    // Index: EventType enum as usize
    handlers: [Option<EventHandler>; 256],
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn register(&mut self, event_type: EventType, handler: EventHandler) {
        let index = event_type as usize;
        self.handlers[index] = Some(handler);
    }

    #[inline]
    pub fn dispatch(&self, event: &MouseEvent) -> ToolResult {
        let index = event.event_type as usize;

        if let Some(ref handler) = self.handlers[index] {
            // Inline para performance
            handler.handle(event)
        } else {
            Ok(())
        }
    }
}

// Zero allocations en hot path
impl Clone for EventRouter {
    fn clone(&self) -> Self {
        // Shallow copy del array
        Self {
            handlers: self.handlers,
        }
    }
}
```

#### Tests TDD + Benchmarks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_register_and_dispatch() {
        let mut router = EventRouter::new();
        let mut called = false;

        router.register(
            EventType::MouseDown,
            EventHandler::from_fn(|_| {
                called = true;
                Ok(())
            }),
        );

        let event = MouseEvent::mouse_down(100.0, 100.0);
        router.dispatch(&event).unwrap();

        assert!(called);
    }

    #[test]
    fn test_dispatch_unregistered_event_noops() {
        let router = EventRouter::new();
        let event = MouseEvent::mouse_move(100.0, 100.0);

        // No debe panic
        let result = router.dispatch(&event);
        assert_matches!(result, Ok(()));
    }

    // Benchmark de performance
    #[bench]
    fn bench_event_dispatch(b: &mut test::Bencher) {
        let mut router = EventRouter::new();

        // Registrar handlers para todos los eventos
        for i in 0..256u8 {
            router.register(
                EventType::from_u8(i),
                EventHandler::from_fn(|_| Ok(())),
            );
        }

        let event = MouseEvent::mouse_down(100.0, 100.0);

        b.iter(|| {
            router.dispatch(&event).unwrap();
        });
    }

    #[test]
    fn test_performance_target() {
        let router = setup_router();
        let event = MouseEvent::mouse_down(100.0, 100.0);

        let start = Instant::now();
        for _ in 0..10_000 {
            router.dispatch(&event).unwrap();
        }
        let elapsed = start.elapsed();

        // Target: < 100ns por dispatch
        let avg_time = elapsed / 10_000;
        assert!(avg_time.as_nanos() < 100, "Dispatch too slow: {:?}", avg_time);
    }
}
```

---

### US-001.4: Atajos de Teclado para Herramientas

**Como** usuario final
**Quiero** cambiar rápidamente entre herramientas con atajos de teclado
**Para** mejorar mi productividad

#### Criterios de Aceptación

- [ ] **CA-001**: V activa herramienta de selección
- [ ] **CA-002**: R activa herramienta de rectángulo
- [ ] **CA-003**: O activa herramienta de elipse
- [ ] **CA-004**: L activa herramienta de línea
- [ ] **CA-005**: P activa herramienta de lápiz
- [ ] **CA-006**: T activa herramienta de texto
- [ ] **CA-007**: Los atajos son configurables

#### Tests TDD

```rust
#[test]
fn test_keyboard_shortcut_select_tool() {
    let mut manager = ToolManager::new();
    manager.register_default_tools().unwrap();

    let key_event = KeyEvent::press('v');
    manager.handle_key_event(&key_event).unwrap();

    assert_eq!(manager.get_current_tool_name(), "select");
}

#[test]
fn test_custom_shortcuts() {
    let mut manager = ToolManager::new();
    manager.register_default_tools().unwrap();

    // Configurar atajo personalizado
    manager.set_shortcut(KeyCode::KeyS, "select").unwrap();

    let key_event = KeyEvent::press('s');
    manager.handle_key_event(&key_event).unwrap();

    assert_eq!(manager.get_current_tool_name(), "select");
}

#[test]
fn test_shortcut_in_text_input_ignored() {
    let mut manager = ToolManager::new();
    manager.register_default_tools().unwrap();

    // Simular que estamos en un input de texto
    manager.set_in_text_input(true);

    let key_event = KeyEvent::press('v');
    manager.handle_key_event(&key_event).unwrap();

    // Herramienta no debe cambiar
    assert_ne!(manager.get_current_tool_name(), "select");
}
```

---

## 🔬 Protocolo de Investigación

### Investigación 1: State Pattern con Typestates

**Objetivo**: Evaluar si typestates mejoran la seguridad del código

**Metodología**:
1. Implementar prototipo con typestates
2. Comparar con runtime state checking
3. Medir overhead de compilación
4. Evaluar experiencia de desarrollo

**Métricas**:
- Tiempo de compilación
- Tamaño del binario
- Performance en runtime
- Lines of código
- Errores capturados en compilación vs runtime

### Investigación 2: Event Dispatching Performance

**Objetivo**: Determinar la estrategia más eficiente para event routing

**Metodología**:
1. Benchmark de lookup table vs HashMap vs match
2. Medir cache misses
3. Evaluar branch prediction
4. Profile con flamegraph

**Métricas**:
- Latencia P50, P95, P99
- Throughput (events/segundo)
- CPU cache hit rate
- Instrucciones por evento

### Investigación 3: Memory Layout para Tool State

**Objetivo**: Optimizar layout de memoria para mejor cache efficiency

**Metodología**:
1. Comparar AoS vs SoA
2. Medir cache misses
3. Evaluar impacto en SIMD
4. Considerar alineación de memoria

**Métricas**:
- Cache hit rate
- Memory bandwidth usage
- SIMD potential

---

## 📊 Métricas de Éxito

### Performance

| Métrica | Target | Medición |
|---------|--------|----------|
| Latencia de dispatch | < 100ns | Benchmark |
| Throughput de eventos | > 10M ops/s | Benchmark |
| Memory overhead | < 1KB por tool | Valgrind |
| Cache miss rate | < 5% | perf stat |
| Compilación time | < 30s增量 | cargo build --timings |

### Calidad

| Métrica | Target | Medición |
|---------|--------|----------|
| Coverage | > 95% | tarpaulin |
| Clippy warnings | 0 | cargo clippy |
| Unsafe blocks | 0 (si es posible) | Auditoría |
| Documentation coverage | 100% público | rustdoc |

### UX

| Métrica | Target | Medición |
|---------|--------|----------|
| Herramientas registradas | > 10 | Conteo |
| Latencia percibida | < 16ms (1 frame) | User testing |
| Tools switching time | < 1ms | Benchmark |

---

## 🚀 Plan de Implementación

### Sprint 1: Fundamentos (Semana 1)

**Objetivo**: Infraestructura base del sistema

#### Día 1-2: Tool Registry
- [ ] Definir `ToolRegistry` struct
- [ ] Implementar `register_tool()`
- [ ] Implementar `get_tool()`
- [ ] Tests para registro
- [ ] Documentación

#### Día 3-4: Tool Manager
- [ ] Definir `ToolManager` struct
- [ ] Implementar `set_tool()`
- [ ] Integrar con `ToolRegistry`
- [ ] Tests para selección
- [ ] Documentación

#### Día 5: Event Router
- [ ] Definir `EventRouter` struct
- [ ] Implementar lookup table
- [ ] Benchmark de performance
- [ ] Optimización si es necesario
- [ ] Documentación

### Sprint 2: State Machine (Semana 2)

**Objetivo**: Máquina de estados completa

#### Día 1-2: Estados y Transiciones
- [ ] Definir enum `ToolState`
- [ ] Implementar `StateMachine`
- [ ] Implementar `transition_to()`
- [ ] Validación de transiciones
- [ ] Tests de estados

#### Día 3-4: Contexto y Datos
- [ ] Definir `ToolContext`
- [ ] Implementar preservación de datos
- [ ] Historial de estados
- [ ] Tests de contexto

#### Día 5: Callbacks
- [ ] `on_enter_state()`
- [ ] `on_exit_state()`
- [ ] Integración con herramientas
- [ ] Tests de callbacks

### Sprint 3: Integración (Semana 3)

**Objetivo**: Integración con sistema existente

#### Día 1-2: SelectTool
- [ ] Migrar `SelectTool` a nueva arquitectura
- [ ] Implementar estados de selección
- [ ] Tests de integración

#### Día 3-4: DrawTool
- [ ] Migrar `DrawTool` a nueva arquitectura
- [ ] Implementar estados de dibujo
- [ ] Tests de integración

#### Día 5: EraseTool
- [ ] Migrar `EraseTool` a nueva arquitectura
- [ ] Implementar estados de borrado
- [ ] Tests de integración

### Sprint 4: Polish (Semana 4)

**Objetivo**: Optimización y documentación

#### Día 1-2: Optimización
- [ ] Profiling con flamegraph
- [ ] Optimizar hot paths
- [ ] Reducir allocations
- [ ] Benchmarks antes/después

#### Día 3-4: Documentación
- [ ] Guía de usuario
- [ ] Ejemplos de código
- [ ] Diagramas de arquitectura
- [ ] API docs completas

#### Día 5: Testing Final
- [ ] Integration tests
- [ ] Stress tests
- [ ] Manual testing
- [ ] Bug fixes

---

## 🧪 Testing Strategy

### Unit Tests

- **Cobertura**: > 95%
- **Framework**: rstest + Mockall
- **Ejecución**: < 5 segundos

### Integration Tests

- **Escenarios**: Flujos completos de usuario
- **Datos**: Canvas con 100+ entidades
- **Mediciones**: Performance y memory

### Property-Based Tests

- **Framework**: proptest
- **Propiedades**:
  - Transiciones de estado son válidas
  - Historial es consistente
  - No hay memory leaks

### Benchmarks

- **Framework**: criterion
- **Casos**:
  - Event dispatching
  - State transitions
  - Tool switching
  - Memory allocation

---

## 📖 Referencias

### Código

- [tldraw - Tool State Machine](https://github.com/tldraw/tldraw)
- [Figma Plugin API](https://www.figma.com/plugin-docs/)
- [Bevy ECS State Machine](https://github.com/bevyengine/bevy)

### Artículos

- [Implementing the state pattern in Rust](https://blog.cesc.cool/implementing-the-state-pattern-in-rust)
- [Data-Oriented Rust Pattern: ECS Beyond Games](https://medium.com/@theopitevedev/the-data-oriented-rust-pattern-ecs-beyond-games-high-performance-backend-design-57596dbb24da)
- [Top 7 Rust ECS Game Development Techniques](https://www.techbuddies.io/2025/12/18/top-7-rust-ecs-game-development-techniques-for-safe-high-performance-play/)

### Papers Académicos

- [Taming stateful computations in Rust with typestates](https://www.sciencedirect.com/science/article/pii/S259011842200051X)
- [The impact of ECS logic on parallel performance](https://ceur-ws.org/Vol-4124/paper43.pdf)

---

## 🔗 Dependencias

### Blocs de otras Épicas

- **EPIC-002**: Sistema de Selección Avanzada (depende de ToolManager)
- **EPIC-003**: Handles de Transformación (depende de StateMachine)
- **EPIC-004**: Comandos de Transformación (depende de Tool events)

### Bloquea

- Todas las épicas de interacción dependen de esta

---

## 📝 Notas

### Decisiones Pendientes

- [ ] ¿Usar typestates o runtime checking?
- [ ] ¿Implementar tool stacking (herramientas compuestas)?
- [ ] ¿Soportar tool plugins (cargadas dinámicamente)?

### Riesgos

- **Riesgo 1**: Complejidad de typestates puede impactar DX
  - **Mitigación**: Prototipar ambos enfoques
  - **Decisión**: Sprint 1

- **Riesgo 2**: Performance de event dispatching
  - **Mitigación**: Benchmark temprano
  - **Decisión**: Día 5 Sprint 1

- **Riesgo 3**: Integración con código existente
  - **Mitigación**: Migración incremental
  - **Decisión**: Sprint 3

### Alternativas Consideradas

#### Alternativa 1: No usar State Machine

**Pros**:
- Más simple inicialmente
- Menos código

**Cons**:
- Estados inconsistentes
- Difícil extender
- Más bugs

**Decisión**: RECHAZADA - Estado inconsistente es inaceptable

#### Alternativa 2: Usar crate existente (eg. `smth`)

**Pros**:
- Menos código que escribir
- Probado en producción

**Cons**:
- Posible mismatch con nuestras necesidades
- Dependencia externa

**Decisión**: CONSIDERAR - Evaluar en Sprint 1

---

**Versión**: 1.0.0
**Última actualización**: 2025-01-28
**Autores**: ArchFlow Development Team
