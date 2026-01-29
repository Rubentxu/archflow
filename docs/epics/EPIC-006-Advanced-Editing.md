# EPIC-006: Advanced Canvas Editing Features
## User Interaction Refinement & Productivity Tools

---

## 📋 Metadatos

| Campo | Valor |
|-------|-------|
| **ID** | EPIC-006 |
| **Título** | Advanced Canvas Editing Features |
| **Prioridad** | 🟡 MEDIA |
| **Complejidad** | Media-Alta |
| **Estimación** | 3-4 semanas |
| **Depende de** | EPIC-001, EPIC-002, EPIC-003, EPIC-004, EPIC-005 |
| **Bloquea** | EPIC-007 (UI Polish), EPIC-008 (Export & IO) |
| **Estado** | 📝 Planeación - Listo para Implementación |
| **Fecha Creación** | 2025-01-28 |

---

## 🎯 Objetivo

Implementar las funcionalidades de edición avanzada pendientes que mejoran la productividad del usuario y completan el conjunto de herramientas de interacción básicas de un editor canvas profesional comparable a Figma/tldraw.

### Alcance

Este EPIC cubre las siguientes áreas funcionales pendientes del [USER-INTERACTION-STUDY](./USER-INTERACTION-STUDY.md):

1. **Keyboard Nudge & Precision Movement** - Movimiento preciso con teclado
2. **Layer Management** - Agrupación y ordenamiento de capas
3. **Alignment & Distribution** - Herramientas de alineación y distribución
4. **Properties Panel Foundation** - Sistema de propiedades editable
5. **Advanced Tools** - Texto, dibujo a mano alzada, y otros

### No Incluye (Futuros EPICs)

- Sistema de colaboración real-time (EPIC-FASE-02)
- Exportación a formatos externos (EPIC-008)
- UI/UX polishing avanzado (EPIC-007)
- Comentarios y anotaciones (EPIC-009)

---

## 📚 Investigación y Mejores Prácticas

### Fuentes Consultadas

#### 1. Keyboard Nudge & Precision Movement

**Referencia:** MDN KeyboardEvent API
- **keydown/keyup events**: Manejo de eventos de teclado con repeat detection
- **Modifier keys**: Alt, Shift, Ctrl detection para diferentes niveles de precisión
- **Auto-repeat handling**: Importante para nudge continuo (16ms intervals típicos)

**Best Practices Identificadas:**
- Usar `keydown` para iniciar movimiento, `keyup` para detener
- Implementar batching de comandos para undo/redo (agrupar movimientos continuos)
- Precisión levels: 1px (normal), 10px (Shift), 0.1px (Alt) - configurable
- Throttling a 60fps (16ms) para evitar saturación del event loop
- Debounce para detectar fin de secuencia de nudge y crear checkpoint de undo

**Patrón de Implementación Recomendado:**
```rust
// Estructura sugerida basada en investigación
pub struct KeyboardNudgeSystem {
    nudge_state: NudgeState,
    command_accumulator: Vec<NudgeCommand>,
    undo_batch_timer: Option<Timer>,
    precision_level: PrecisionLevel,
}

enum PrecisionLevel {
    Normal = 1,      // 1px
    Fast = 10,       // 10px (Shift)
    Precise = 1,     // 0.1px (Alt) - requiere transform decimal
}
```

#### 2. Layer Management (Group/Ungroup)

**Referencia:** Figma/tldraw Architecture
- **Parent-Child Relationships**: Árbol de escena con referencias padre-hijo
- **Z-Index Management**: Sistema de ordenamiento basado en índices numéricos
- **Group Transform Inheritance**: Transformaciones acumulativas en grupo

**Best Practices Identificadas:**
- Mantener flat array de entidades para renderizado (performance)
- Árbol separado para relaciones padre-hijo (query rápidas)
- Group como entidad especial que contiene children IDs
- Transformaciones locales vs globales (matriz de acumulación)
- Máximo nesting depth: 10 niveles (para evitar stack overflow en cálculos)

**Data Structure Pattern:**
```rust
// Entity-Component-System approach
pub struct GroupComponent {
    children: Vec<EntityId>,
    parent: Option<EntityId>,
    depth: u8,  // Nesting level
}

pub struct LayerSystem {
    z_index_counter: AtomicU64,
    groups: HashMap<EntityId, GroupComponent>,
}
```

#### 3. Alignment & Distribution Algorithms

**Referencia:** Computational Geometry - Algorithms and Applications

**Alignment Mathematics:**
```
// Left alignment
for each selected entity:
    target_x = min(entity.bounds.x for entity in selection)
    entity.x = target_x

// Center alignment  
for each selected entity:
    target_center_x = average(entity.center.x for entity in selection)
    entity.x = target_center_x - entity.width / 2

// Distribution (horizontal)
sorted = sort_by_x(selection)
min_x = sorted[0].x
max_x = sorted[-1].x
total_width = max_x - min_x
spacing = total_width / (len(selection) - 1)

for i, entity in enumerate(sorted):
    entity.x = min_x + (spacing * i)
```

**Performance Considerations:**
- O(n log n) para ordenamiento (distribución)
- O(n) para alignment simple
- Cache de bounds calculados para evitar recálculos
- Batch update de entidades (single transaction)

#### 4. Properties Panel Architecture

**Referencia:** React/Electron Patterns (Figma, tldraw)

**Key Findings:**
- **Reactive Updates**: Sistema de observables para cambios en propiedades
- **Multi-selection Support**: Mostrar propiedades comunes + indicadores de valores mixtos
- **Type Safety**: Validación de tipos en tiempo de compilación
- **Serialization**: Debe integrarse con sistema existente de SerializedEntity

**Component Architecture:**
```rust
// Property System
pub trait PropertyEditor<T> {
    fn get_value(&self) -> Option<T>;  // None si multi-select con valores diferentes
    fn set_value(&mut self, value: T);
    fn is_mixed(&self) -> bool;
}

pub struct PropertiesPanel {
    selected_entities: Vec<EntityId>,
    property_editors: Vec<Box<dyn PropertyEditor>>,
    update_bus: EventBus,
}
```

#### 5. Text Tool Implementation

**Referencia:** Canvas Text Rendering & Rust Text Shaping

**Challenges Identified:**
- **Text Shaping**: HarfBuzz para text shaping complejo (ligatures, RTL)
- **Line Breaking**: UAX #14 Unicode Line Breaking Algorithm
- **Hit Testing**: Mapeo de coordenadas a posición de cursor en texto
- **Editing**: Manejo de IME (Input Method Editor) para CJK y otros idiomas

**Simplified Initial Approach:**
- Fase 1: Texto monolínea simple con canvas fillText
- Fase 2: Texto multilínea básico con word wrapping simple
- Fase 3: Full text shaping con HarfBuzz (si es necesario)

---

## 🏗️ Arquitectura Propuesta

### Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────────┐
│                     EPIC-006 Architecture                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐ │
│  │ Keyboard System │  │  Layer System   │  │ Alignment System │ │
│  │                 │  │                 │  │                  │ │
│  │ - NudgeHandler  │  │ - GroupManager  │  │ - AlignEngine    │ │
│  │ - KeyBinding    │  │ - ZIndexManager │  │ - Distribute     │ │
│  │ - CommandBatch  │  │ - TreeTraversal │  │ - SnapSystem     │ │
│  └────────┬────────┘  └────────┬────────┘  └────────┬─────────┘ │
│           │                    │                    │           │
│           └────────────────────┼────────────────────┘           │
│                                │                                │
│                     ┌──────────▼──────────┐                     │
│                     │   CommandExecutor   │                     │
│                     │   (EPIC-004)        │                     │
│                     └──────────┬──────────┘                     │
│                                │                                │
│  ┌─────────────────┐  ┌────────▼────────┐  ┌──────────────────┐ │
│  │ Properties      │  │   Canvas API    │  │ Text System      │ │
│  │ Panel           │  │   (EPIC-001)    │  │                  │ │
│  │                 │  │                 │  │ - TextEntity     │ │
│  │ - PropertyGrid  │  │ - Shape updates │  │ - LineBreaking   │ │
│  │ - Validation    │  │ - Transform     │  │ - HitTesting     │ │
│  │ - Type Safety   │  │ - Render        │  │ - Editing        │ │
│  └─────────────────┘  └─────────────────┘  └──────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Módulos

```
crates/archflow-sdk/src/
├── keyboard/
│   ├── mod.rs
│   ├── nudge_system.rs       # US-006.1
│   └── key_binding.rs
├── layer/
│   ├── mod.rs
│   ├── group_manager.rs      # US-006.2
│   ├── z_index_manager.rs
│   └── tree_traversal.rs
├── alignment/
│   ├── mod.rs
│   ├── align_engine.rs       # US-006.3
│   └── snap_system.rs
├── properties/
│   ├── mod.rs
│   ├── panel.rs              # US-006.4
│   ├── editors.rs
│   └── validation.rs
└── text/
    ├── mod.rs
    ├── text_entity.rs        # US-006.5
    └── text_tool.rs
```

---

## 📝 Historias de Usuario

### US-006.1: Keyboard Nudge & Precision Movement ✅ **IMPLEMENTED**

#### Descripción
Como usuario, quiero mover objetos seleccionados usando las teclas de flecha con diferentes niveles de precisión para posicionar elementos con exactitud.

#### Estado
✅ **COMPLETADO** - Implementado en `crates/archflow-sdk/src/keyboard/mod.rs`

#### Criterios de Aceptación

- [x] **Nudge básico**: Flechas mueven 1px por pulsación
- [x] **Nudge rápido**: Shift + Flechas mueven 10px por pulsación
- [x] **Nudge preciso**: Alt + Flechas mueven 0.1px por pulsación
- [x] **Soporte multi-selección**: Todas las entidades seleccionadas se mueven juntas
- [x] **Undo/Redo**: Secuencias de nudge se agrupan en un solo comando
- [x] **Auto-repeat**: Mantener presionada la tecla continúa el movimiento
- [x] **Detección de fin**: Finalizar batch de undo al soltar la tecla

#### Implementación

**Módulo**: `keyboard`

**Componentes principales**:
- `KeyboardNudgeSystem`: Sistema principal de control de nudge
- `NudgeCommand`: Comando implementando trait `Command` para undo/redo
- `PrecisionLevel`: Enum con niveles Normal (1px), Fast (10px), Precise (0.1px)
- `NudgeDirection`: Enum con direcciones Up, Down, Left, Right
- `CanvasNudgeExt`: Extensión del Canvas para operaciones de nudge

**Características**:
- Batching automático de comandos con timeout configurable (300ms por defecto)
- Soporte para merge de comandos consecutivos
- Integración completa con el sistema de comandos existente
- API fluent para configuración de auto-repeat

#### Tests Implementados (23 tests - TDD ✅)

**Test Obligatorios (Red → Green → Refactor):**

```rust
#[test]
fn test_nudge_basic_1px() {
    // Arrange: Crear rectángulo en (100, 100)
    // Act: Presionar flecha derecha una vez
    // Assert: Rectángulo debe estar en (101, 100)
}

#[test]
fn test_nudge_fast_shift_10px() {
    // Arrange: Crear rectángulo en (100, 100)
    // Act: Presionar Shift + flecha derecha
    // Assert: Rectángulo debe estar en (110, 100)
}

#[test]
fn test_nudge_precise_alt_0_1px() {
    // Arrange: Crear rectángulo en (100.0, 100.0)
    // Act: Presionar Alt + flecha derecha
    // Assert: Rectángulo debe estar en (100.1, 100.0)
}

#[test]
fn test_nudge_multi_selection() {
    // Arrange: Crear 3 rectángulos en diferentes posiciones
    // Act: Seleccionar todos y presionar flecha derecha
    // Assert: Todos los rectángulos deben moverse 1px a la derecha
}

#[test]
fn test_nudge_undo_batching() {
    // Arrange: Crear rectángulo, iniciar nudge continuo
    // Act: Mover 10 veces con flecha, luego undo
    // Assert: Un solo undo debe revertir todas las operaciones
}

#[test]
fn test_nudge_auto_repeat() {
    // Arrange: Crear rectángulo en (0, 0)
    // Act: Mantener flecha derecha presionada por 1 segundo
    // Assert: Rectángulo debe haberse movido aproximadamente 60px (60fps)
}

#[test]
fn test_nudge_no_movement_on_boundary() {
    // Arrange: Crear rectángulo en canvas boundary
    // Act: Intentar mover fuera de límites
    // Assert: Posición debe permanecer dentro de bounds del canvas
}
```

**Cobertura Requerida:**
- Unit tests: 8+ tests
- Integration tests: 4+ tests (con Canvas real)
- Performance tests: 2+ tests (60fps bajo carga)

#### Investigación Requerida

**Antes de implementar, investigar:**

1. **Keyboard Event Handling in Rust/WASM:**
   - Usar perplexity: "Rust wasm keyboard event handling best practices 60fps"
   - Investigar: Prevent default behavior, key repeat rates
   - Context7: Buscar en crates de input handling (winit, web-sys)

2. **Command Batching Patterns:**
   - Usar perplexity: "Command pattern batching undo redo editor implementation"
   - Investigación: Cómo Figma/tldraw agrupan comandos de nudge
   - Buscar: Debounce vs Throttle para detectar fin de secuencia

3. **Performance Optimization:**
   - Usar perplexity: "Canvas editor 60fps keyboard movement optimization"
   - Investigar: RAF (RequestAnimationFrame) integration
   - Context7: Spatial indexing updates during movement

**Entregables de Investigación:**
- [ ] Documento con findings de keyboard handling
- [ ] Prototipo de command batching
- [ ] Benchmark de performance objetivo (< 16ms por frame)

---

### US-006.2: Layer Management (Group/Ungroup) ✅ **IMPLEMENTED**

#### Descripción
Como usuario, quiero agrupar múltiples objetos para moverlos y transformarlos como una unidad, y gestionar el orden de capas (z-index).

#### Estado
✅ **COMPLETADO** - Implementado en `crates/archflow-sdk/src/group/mod.rs`

#### Criterios de Aceptación

- [x] **Group**: Ctrl/Cmd + G agrupa selección actual
- [x] **Ungroup**: Ctrl/Cmd + Shift + G desagrupa grupo seleccionado
- [x] **Nested groups**: Soporte para grupos anidados (hasta 10 niveles)
- [x] **Visual indicator**: Bounding box calculada automáticamente
- [x] **Transform inheritance**: Soporte estructural para herencia de transforms
- [x] **Selection**: API para obtener grupo de una forma
- [x] **Persistence**: Estructura serializable con serde

#### Implementación

**Módulo**: `group`

**Componentes principales**:
- `GroupManager`: Gestiona todos los grupos y sus relaciones
- `Group`: Estructura de grupo con children, parent, bounds, transforms
- `GroupBounds`: Bounds calculados del grupo basado en children
- `ShapeTransform`: Almacena transform originales para undo
- `GroupCommand`/`UngroupCommand`: Comandos para undo/redo

**Características**:
- Soporte para grupos anidados (MAX_GROUP_DEPTH = 10)
- Cálculo automático de bounds del grupo
- Mapeo bidireccional shape ↔ grupo
- Serialización completa con serde
- Tests exhaustivos (18 tests TDD)

#### Tests Implementados (18 tests - TDD ✅)

**Tests Obligatorios (todos pasando):

```rust
#[test]
fn test_group_creates_group_entity() {
    // Arrange: Seleccionar 3 rectángulos
    // Act: Presionar Ctrl+G
    // Assert: Debe crearse 1 entidad Group con 3 children
}

#[test]
fn test_ungroup_restores_children() {
    // Arrange: Crear grupo con 3 rectángulos
    // Act: Presionar Ctrl+Shift+G
    // Assert: Grupo eliminado, 3 rectángulos independientes
}

#[test]
fn test_nested_groups_max_depth() {
    // Arrange: Crear grupos anidados hasta 10 niveles
    // Act: Intentar crear nivel 11
    // Assert: Error o prevención de nivel 11
}

#[test]
fn test_group_transform_inheritance() {
    // Arrange: Grupo con rectángulo en (100, 100)
    // Act: Mover grupo a (200, 200)
    // Assert: Rectángulo debe estar en (200, 200) en coordenadas globales
}

#[test]
fn test_layer_ordering_forward() {
    // Arrange: 3 rectángulos con z-index 1, 2, 3
    // Act: Seleccionar z=2, presionar ]
    // Assert: Z-index debe ser 3 (y anterior 3 debe ser 2)
}

#[test]
fn test_layer_ordering_to_front() {
    // Arrange: 5 rectángulos, seleccionar primero
    // Act: Presionar Shift+]
    // Assert: Seleccionado debe tener z-index máximo
}

#[test]
fn test_selection_group_vs_child() {
    // Arrange: Grupo con rectángulo interno
    // Act 1: Click simple → seleccionar grupo
    // Act 2: Doble click → seleccionar rectángulo
    // Assert: Dos estados de selección distintos
}

#[test]
fn test_group_serialization() {
    // Arrange: Grupo anidado con transformaciones
    // Act: Serializar a JSON y deserializar
    // Assert: Estructura y posiciones deben ser idénticas
}
```

**Cobertura Requerida:**
- Unit tests: 10+ tests
- Integration tests: 6+ tests
- Stress tests: 1 test (1000+ entidades anidadas)

#### Investigación Requerida

1. **Scene Graph Architectures:**
   - Usar perplexity: "Scene graph vs flat entity list game engine performance"
   - Investigar: FlatArray + ParentIndex vs TreeStructure
   - Context7: Bevy ECS parenting, specs hierarchy

2. **Group Transform Mathematics:**
   - Usar perplexity: "2D transform matrix composition parent child coordinate spaces"
   - Investigación: Local vs World coordinate systems
   - Buscar: Matrix acumulación para nested transforms

3. **Z-Index Management:**
   - Usar perplexity: "Canvas z-index management 1000+ layers algorithm"
   - Investigar: Sparse vs Dense indexing
   - Buscar: Stable sort para layer ordering

**Entregables de Investigación:**
- [ ] Benchmark: Scene graph vs flat array (10K, 100K entidades)
- [ ] Documento: Transform matrix composition math
- [ ] Prototipo: Z-index management con sorting estable

---

### US-006.3: Alignment & Distribution Tools

#### Descripción
Como usuario, quiero alinear y distribuir múltiples objetos seleccionados horizontal y verticalmente para crear layouts precisos.

#### Criterios de Aceptación

1. **Align Left**: Alinear al borde izquierdo más extremo
2. **Align Center**: Alinear al centro horizontal promedio
3. **Align Right**: Alinear al borde derecho más extremo
4. **Align Top/Middle/Bottom**: Equivalentes verticales
5. **Distribute Horizontal**: Espaciado equidistante horizontal
6. **Distribute Vertical**: Espaciado equidistante vertical
7. **Smart guides**: Líneas guía visuales durante alineación (bonus)
8. **Multi-select**: Funciona con 2+ objetos seleccionados

#### Requisitos TDD

**Test Obligatorios:**

```rust
#[test]
fn test_align_left() {
    // Arrange: Rectángulos en x: [100, 150, 200]
    // Act: Alinear a la izquierda
    // Assert: Todos deben tener x = 100 (mínimo)
}

#[test]
fn test_align_center_horizontal() {
    // Arrange: Rectángulos de diferente ancho centrados en diferentes puntos
    // Act: Alinear al centro
    // Assert: Todos deben compartir el mismo centro X promedio
}

#[test]
fn test_align_right() {
    // Arrange: Rectángulos con bordes derechos en [200, 250, 300]
    // Act: Alinear a la derecha
    // Assert: Todos los bordes derechos deben estar en x = 300
}

#[test]
fn test_distribute_horizontal() {
    // Arrange: 3 rectángulos en x: [0, 100, 300]
    // Act: Distribuir horizontalmente
    // Assert: Espaciado uniforme: [0, 150, 300]
}

#[test]
fn test_distribute_vertical() {
    // Arrange: 4 rectángulos en y: [0, 50, 100, 400]
    // Act: Distribuir verticalmente
    // Assert: Espaciado uniforme con gaps iguales
}

#[test]
fn test_align_requires_min_2_selections() {
    // Arrange: 1 rectángulo seleccionado
    // Act: Intentar alinear
    // Assert: Operación no debe ejecutarse (no-op o error)
}

#[test]
fn test_align_undo_redo() {
    // Arrange: Múltiples rectángulos desalineados
    // Act: Alinear, undo, redo
    // Assert: Estados deben alternar correctamente
}

#[test]
fn test_align_preserves_other_properties() {
    // Arrange: Rectángulos con diferentes rotaciones, colores, tamaños
    // Act: Alinear a la izquierda
    // Assert: Solo posición X cambia, demás propiedades intactas
}
```

**Cobertura Requerida:**
- Unit tests: 8+ tests
- Edge case tests: 4+ tests (objetos con tamaño 0, negativo, etc.)

#### Investigación Requerida

1. **Alignment Algorithms:**
   - Usar perplexity: "2D alignment algorithm computational geometry implementation"
   - Investigar: Bounding box calculation optimization
   - Buscar: Canvas editor alignment implementation tldraw figma

2. **Distribution Mathematics:**
   - Usar perplexity: "Equal spacing distribution algorithm graphic design tools"
   - Investigación: Edge-based vs Center-based distribution
   - Buscar: Sorting algorithms performance (10K+ objects)

3. **Smart Guides Implementation:**
   - Usar perplexity: "Smart guides snap visual indicators canvas implementation"
   - Investigar: Spatial indexing para detección rápida de alineaciones
   - Context7: Buscar crates de geometría computacional

**Entregables de Investigación:**
- [ ] Implementación de referencia: Algoritmos de alineación
- [ ] Benchmark: Sorting performance para distribución
- [ ] Documento: Decisiones edge-based vs center-based

---

### US-006.4: Properties Panel Foundation

#### Descripción
Como usuario, quiero ver y editar las propiedades de los objetos seleccionados (posición, tamaño, color, etc.) en un panel de propiedades.

#### Criterios de Aceptación

1. **Display properties**: Mostrar posición (x, y), tamaño (w, h), rotación
2. **Edit properties**: Editar valores numéricos con validación
3. **Multi-selection**: Mostrar valores comunes; indicar valores mixtos
4. **Color editing**: Selector de color para fill y stroke
5. **Opacity**: Slider 0-100%
6. **Stroke width**: Input numérico con unidades
7. **Corner radius**: Input para rectángulos (0+ px)
8. **Validation**: Prevenir valores inválidos (negativos donde no aplica)
9. **Live update**: Cambios reflejados inmediatamente en canvas
10. **Undo/Redo**: Cada cambio es un comando deshacible

#### Requisitos TDD

**Test Obligatorios:**

```rust
#[test]
fn test_properties_display_single_selection() {
    // Arrange: Seleccionar rectángulo con propiedades conocidas
    // Act: Abrir panel de propiedades
    // Assert: Panel debe mostrar x=100, y=100, w=50, h=50, etc.
}

#[test]
fn test_properties_edit_position() {
    // Arrange: Rectángulo en (100, 100)
    // Act: Cambiar X a 200 en panel de propiedades
    // Assert: Rectángulo debe moverse a (200, 100)
}

#[test]
fn test_properties_multi_selection_mixed_values() {
    // Arrange: Seleccionar 2 rectángulos con diferentes X
    // Act: Abrir panel de propiedades
    // Assert: Campo X debe mostrar "Mixed" o indicador visual
}

#[test]
fn test_properties_validation_negative_width() {
    // Arrange: Rectángulo con w=50
    // Act: Intentar cambiar width a -10
    // Assert: Validación debe rechazar; valor permanece en 50
}

#[test]
fn test_properties_color_picker() {
    // Arrange: Rectángulo con fill rojo
    // Act: Cambiar fill a azul usando color picker
    // Assert: Rectángulo debe renderizar azul
}

#[test]
fn test_properties_undo_single_change() {
    // Arrange: Rectángulo en (100, 100)
    // Act: Cambiar X a 200, luego undo
    // Assert: Rectángulo debe volver a (100, 100)
}

#[test]
fn test_properties_live_update() {
    // Arrange: Rectángulo visible en canvas
    // Act: Escribir nuevo valor X (sin presionar Enter aún)
    // Assert: Canvas debe actualizar en tiempo real (debounce 100ms)
}
```

**Cobertura Requerida:**
- Unit tests: 7+ tests
- Integration tests: 5+ tests (con Canvas real)
- UI tests: 3+ tests (renderizado correcto)

#### Investigación Requerida

1. **Property Editor Patterns:**
   - Usar perplexity: "Property panel architecture react figma immediate mode"
   - Investigar: Immediate vs retained mode para UI
   - Buscar: Observable pattern para sincronización modelo-vista

2. **Validation Strategies:**
   - Usar perplexity: "Rust type safe validation patterns user input"
   - Investigar: Parse, don't validate vs Validation types
   - Context7: crates de validación (validator, garde)

3. **Color Management:**
   - Usar perplexity: "Color picker implementation RGBA HSL conversion"
   - Investigación: Color space conversions (sRGB, HSL, HSV)
   - Buscar: Color picker UI patterns accessibility

**Entregables de Investigación:**
- [ ] Prototipo: Sistema de validación type-safe
- [ ] Documento: Architecture decision - Immediate vs Retained
- [ ] Benchmark: Observable pattern overhead (1K+ objetos)

---

### US-006.5: Text Tool Implementation

#### Descripción
Como usuario, quiero añadir y editar texto en el canvas para crear anotaciones y labels.

#### Criterios de Aceptación

1. **Create text**: Herramienta T para crear textos
2. **Click to place**: Click en canvas coloca cursor de texto
3. **Type to edit**: Escribir modifica el texto
4. **Move text**: Arrastrar mueve el texto como cualquier shape
5. **Basic styling**: Font family, size, weight, color
6. **Edit mode**: Doble click entra modo edición
7. **Exit edit**: Click fuera o Escape sale de edición
8. **Text bounds**: Caja delimitadora visible durante edición
9. **Serialization**: Texto guardado en JSON

#### Requisitos TDD

**Test Obligatorios:**

```rust
#[test]
fn test_text_tool_creates_text_entity() {
    // Arrange: Seleccionar herramienta T
    // Act: Click en canvas en (100, 100)
    // Assert: Debe crearse entidad Text en (100, 100)
}

#[test]
fn test_text_edit_type_changes_content() {
    // Arrange: Texto existente con "Hello"
    // Act: Doble click para editar, escribir "World"
    // Assert: Contenido debe ser "World"
}

#[test]
fn test_text_move_like_shape() {
    // Arrange: Texto en (100, 100)
    // Act: Arrastrar a (200, 200)
    // Assert: Texto debe estar en (200, 200)
}

#[test]
fn test_text_styling_font_size() {
    // Arrange: Texto con font_size = 16
    // Act: Cambiar a font_size = 24
    // Assert: Bounding box debe actualizarse; render más grande
}

#[test]
fn test_text_edit_mode_exit_on_click_outside() {
    // Arrange: En modo edición de texto
    // Act: Click en espacio vacío del canvas
    // Assert: Modo edición terminado; shape seleccionada
}

#[test]
fn test_text_serialization() {
    // Arrange: Texto con contenido, posición, estilo
    // Act: Serializar a JSON y deserializar
    // Assert: Todo debe preservarse exactamente
}

#[test]
fn test_text_tool_undo_create() {
    // Arrange: Crear texto con herramienta T
    // Act: Undo
    // Assert: Texto eliminado del canvas
}
```

**Cobertura Requerida:**
- Unit tests: 7+ tests
- Integration tests: 4+ tests

#### Investigación Requerida

1. **Text Rendering in Canvas:**
   - Usar perplexity: "HTML5 canvas text rendering performance fillText measureText"
   - Investigar: Caching de textos renderizados
   - Buscar: Font loading strategies (FOIT vs FOUT)

2. **Text Editing State Machine:**
   - Usar perplexity: "Canvas text editing state machine implementation"
   - Investigar: Focus management, cursor position
   - Buscar: IME composition handling for CJK

3. **Font Management:**
   - Usar perplexity: "Web font loading canvas text tool system fonts"
   - Investigación: System fonts vs Web fonts performance
   - Context7: web-sys font APIs, font-kit crate

**Entregables de Investigación:**
- [ ] Prototipo: Text rendering con canvas 2D context
- [ ] Documento: State machine para modo edición
- [ ] Benchmark: Text measureText performance (1K+ textos)

---

## 🔬 Protocolo de Investigación

### Investigación 1: Performance de Keyboard Nudge

**Pregunta:** ¿Cómo manejar 60fps nudge con 1000+ entidades seleccionadas?

**Metodología:**
1. Implementar versión naive (actualización inmediata de cada entidad)
2. Medir FPS con diferentes cantidades de entidades (100, 500, 1000, 5000)
3. Implementar optimizaciones:
   - Batch updates (single render pass)
   - Spatial index lazy updates
   - Transform matrix composition caching
4. Comparar resultados y documentar trade-offs

**Criterio de Éxito:** 60fps sostenido con 1000+ entidades en movimiento

### Investigación 2: Scene Graph vs Flat Array

**Pregunta:** ¿Qué arquitectura es mejor para grupos anidados?

**Metodología:**
1. Implementar ambos approaches:
   - FlatArray + ParentIndexMap
   - TreeNode structure with recursion
2. Benchmarks:
   - Insert: 10K entidades
   - Query: "find all children of group X"
   - Update: move group with 100 nested children
   - Render: traverse to generate draw calls
3. Medir memoria y CPU

**Criterio de Éxito:** Elección basada en datos, no suposiciones

### Investigación 3: Alignment Algorithm Efficiency

**Pregunta:** ¿Cómo escala el alineamiento con 10K objetos?

**Metodología:**
1. Implementar alignment O(n) naive
2. Generar datasets: 100, 1000, 10000 objetos aleatorios
3. Medir tiempo de ejecución
4. Optimizar si es necesario (paralelismo, SIMD)
5. Documentar complejidad real vs teórica

**Criterio de Éxito:** < 16ms para cualquier operación de alignment

---

## 📊 Métricas de Éxito

### Performance

| Métrica | Target | Medición |
|---------|--------|----------|
| Nudge FPS | 60fps | Chrome DevTools |
| Group creation | < 50ms | 1000 entities |
| Alignment | < 16ms | 10000 entities |
| Properties panel load | < 100ms | 100 selected |
| Text render | < 2ms | 100 text entities |

### Calidad

| Métrica | Target | Medición |
|---------|--------|----------|
| Test coverage | > 90% | tarpaulin |
| User tasks | 100% | Manual testing |
| Undo reliability | 100% | Property tests |
| Accessibility | WCAG 2.1 AA | axe-core |

---

## 🚀 Plan de Implementación

### Sprint 1: Keyboard Nudge & Foundation (Semana 1)

**Objetivo:** Implementar sistema de nudge con alto rendimiento

**Tareas:**
- [ ] US-006.1: Investigación keyboard handling (2 días)
- [ ] US-006.1: Implementación NudgeSystem (2 días)
- [ ] US-006.1: Tests TDD (1 día)
- [ ] Benchmark performance (1 día)

**Entregables:**
- NudgeSystem funcional con 60fps
- 8+ tests pasando
- Documento de investigación

### Sprint 2: Layer Management (Semana 2)

**Objetivo:** Implementar grupos y ordenamiento de capas

**Tareas:**
- [ ] US-006.2: Investigación scene graph (1 día)
- [ ] US-006.2: Implementar GroupManager (2 días)
- [ ] US-006.2: Implementar ZIndexManager (1 día)
- [ ] US-006.2: Tests TDD (1 día)

**Entregables:**
- Group/Ungroup funcionando
- Layer ordering (], [)
- 10+ tests pasando

### Sprint 3: Alignment & Distribution (Semana 3)

**Objetivo:** Herramientas de alineación profesionales

**Tareas:**
- [ ] US-006.3: Investigación algoritmos (1 día)
- [ ] US-006.3: Implementar AlignEngine (2 días)
- [ ] US-006.3: Implementar Distribute (1 día)
- [ ] US-006.3: Tests TDD (1 día)

**Entregables:**
- 6 tipos de alineación
- Distribución horizontal/vertical
- 8+ tests pasando

### Sprint 4: Properties Panel & Text (Semana 4)

**Objetivo:** Panel de propiedades y herramienta de texto

**Tareas:**
- [ ] US-006.4: Investigación property patterns (1 día)
- [ ] US-006.4: Implementar PropertiesPanel (2 días)
- [ ] US-006.5: Implementar TextTool (1 día)
- [ ] Tests TDD ambos (1 día)

**Entregables:**
- PropertiesPanel funcional
- TextTool básico
- 14+ tests pasando

---

## 📖 Referencias

### Implementación
- [tldraw Editor Architecture](https://tldraw.dev/docs/editor)
- [Figma Plugin API](https://www.figma.com/plugin-docs/)
- [Canvas API Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API)

### Investigación
- [Computational Geometry - de Berg et al.](https://link.springer.com/book/10.1007/978-3-540-77974-2)
- [Game Engine Architecture - Gregory](https://www.gameenginebook.com/)
- [Data-Oriented Design - Acton](https://www.dataorienteddesign.com/dodbook/)

### Código Referencia
- [tldraw/tldraw](https://github.com/tldraw/tldraw)
- [excalidraw/excalidraw](https://github.com/excalidraw/excalidraw)
- [bevyengine/bevy](https://github.com/bevyengine/bevy) - ECS & hierarchy

---

## 🔗 Dependencias

### Requiere
- ✅ EPIC-001: Tool State Machine (completado)
- ✅ EPIC-002: Advanced Selection (completado)
- ✅ EPIC-003: Transform Controls (completado)
- ✅ EPIC-004: Commands & Clipboard (completado)
- ✅ EPIC-005: Transformation Matrix (completado)

### Provee a
- EPIC-007: UI Polish & Refinement
- EPIC-008: Export & IO Operations

---

## ✅ Checklist de Completitud

Antes de marcar este EPIC como completado, verificar:

- [ ] Todos los user stories implementados
- [ ] Todos los tests TDD pasando (coverage > 90%)
- [ ] Investigación documentada para cada US
- [ ] Benchmarks de performance realizados
- [ ] Documentación de API actualizada
- [ ] Manual de usuario actualizado
- [ ] `cargo test --workspace` pasa 100%
- [ ] No hay warnings de clippy críticos
- [ ] Revisión de código completada

---

*Documento creado: 2025-01-28*
*Última actualización: 2025-01-29*
*Estado: En desarrollo - US-006.1 Completada*
