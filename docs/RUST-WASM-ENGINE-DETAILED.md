# ArchFlow: Motor Gráfico Rust/WASM - Análisis Técnico Profundo

**Versión:** 3.0 - Análisis Detallado  
**Fecha:** 2026-01-23  
**Objetivo:** Motor gráfico de alto rendimiento 10k+ elementos @ 60fps

---

## Índice

1. [Introducción y Objetivos](#1-introducción-y-objetivos)
2. [Análisis Profundo: tldraw](#2-análisis-profundo-tldraw)
3. [Análisis Profundo: React Flow](#3-análisis-profundo-react-flow)
4. [Análisis Profundo: Excalidraw](#4-análisis-profundo-excalidraw)
5. [Arquitectura del Motor Propuesto](#5-arquitectura-del-motor-propuesto)
6. [Rendering WebGPU](#6-rendering-webgpu)
7. [Sistema ECS Considerado](#7-sistema-ecs-considerado)
8. [Implementación en Rust](#8-implementación-en-rust)
9. [Roadmap de Implementación](#9-roadmap-de-implementación)

---

## 1. Introducción y Objetivos

### 1.1 El Problema

Los motores gráficos JavaScript actuales para diagramación tienen limitaciones fundamentales:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     LIMITACIONES DE MOTORES JS                          │
├─────────────────────────────────────────────────────────────────────────┤
│  Canvas 2D:                                                             │
│  • Límite práctico: ~5,000 elementos a 60fps                            │
│  • Rendering en CPU, no GPU                                             │
│  • Sin instancing ni batching eficiente                                │
│  • Text rendering especialmente costoso                                │
│                                                                         │
│  WebGL (Three.js, PixiJS):                                              │
│  • Overhead significativo de JavaScript                                │
│  • Garbage collection pauses                                            │
│  • Memory fragmentation                                                 │
│  • Dificultad para debuggear GPU                                        │
│                                                                         │
│  WebGPU (Bibliotecas JS):                                               │
│  • Bridge JS→WASM añade overhead                                        │
│  • Serialización de datos costosa                                       │
│  • Control fino de memoria limitado                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 La Solución: Rust + WebGPU

```
┌─────────────────────────────────────────────────────────────────────────┐
│                  VENTAJAS DE RUST + WEBGPU                             │
├─────────────────────────────────────────────────────────────────────────┤
│  Rust:                                                                  │
│  ✅ Zero-cost abstractions                                             │
│  ✅ Memory safety sin GC                                               │
│  ✅ Data-oriented design natural                                       │
│  ✅ Compile-time optimization                                          │
│  ✅ SIMD intrinsics                                                    │
│                                                                         │
│  WebGPU:                                                                │
│  ✅ Compute shaders para operaciones paralelas                         │
│  ✅ Instanced rendering                                                │
│  ✅ Bind groups para compartir recursos                                │
│  ✅ Control explícito de memoria GPU                                    │
│  ✅ Modern gráficos pipeline                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Objetivos Técnicos

| Métrica | Objetivo | Justificación |
|---------|----------|---------------|
| **Elementos visibles** | 10,000+ @ 60fps | Diagramas enterprise complejos |
| **Zoom fluido** | 0.1x - 100x | Navegación infinito canvas |
| **Animaciones** | 1,000+ simultáneas | Transiciones, microinteracciones |
| **Memoria WASM** | < 50MB | Móviles y navegadores modestos |
| **Startup time** | < 100ms | Percepción de instantaneidad |
| **Tamaño bundle** | < 500KB gz | Descarga rápida |

---

## 2. Análisis Profundo: tldraw

### 2.1 Arquitectura General de tldraw

tldraw tiene una arquitectura en capas bien definida:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        TLDRWA ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  PRESENTATION LAYER                                                │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • React Components (UI, tools, menus)                            │ │
│  │  • Event Handlers (pointer, keyboard, touch)                       │ │
│  │  • tldraw SDK (high-level API)                                     │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  STATE MANAGEMENT LAYER                                            │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Store (centralized state)                                      │ │
│  │  • Records (immutable data structures)                            │ │
│  │  • History (undo/redo)                                             │ │
│  │  • Transactions (atomic operations)                               │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  GEOMETRY & UTILS LAYER                                            │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Vec2, Vec3, Mat4 (math primitives)                             │ │
│  │  • Bounds, AABB (spatial queries)                                 │ │
│  │  • Intersection tests                                             │ │
│  │  • Fractional indexing (z-order)                                 │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  RENDERING LAYER                                                   │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Renderer (Canvas 2D)                                           │ │
│  │  • Renderer (WebGL - experimental)                                │ │
│  │  • Batching, culling, LOD                                          │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Sistema de Records - Análisis Detallado

**¿Por qué es tan importante?**

El sistema de records de tldraw es el corazón de su arquitectura. Es el patrón que más valor aporta para portar a Rust debido a:

1. **Inmutabilidad**: Perfecto para Rust's ownership model
2. **Type-safety**: TypeScript estricto que traduce naturalmente a Rust
3. **Performance**: Estructuras optimizadas para cache locality
4. **Serialización**: JSON nativo para persistencia

#### Estructura del Record System

```typescript
// TLBaseRecord - La base de TODO en tldraw
interface TLBaseRecord {
  id: string                    // Identificador único
  typeName: string              // Tipo del record ("shape", "arrow", etc.)
  index: string                 // Fractional index para z-order
}

// Ejemplo de record concreto
interface TLShape extends TLBaseRecord {
  id: string
  typeName: "shape"
  index: string
  x: number
  y: number
  width: number
  height: number
  rotation: number
  opacity: number
}
```

**Patrones Clave Identificados:**

1. **Fractional Indexing para Z-Order**
   - Algoritmo de `jittered-fractional-indexing`
   - Permite insertar elementos entre otros sin conflictos
   - Strings lexicográficos como índices (ej: "a0", "a1", "a1V", "a2")

2. **Inmutabilidad con Updates Eficientes**
   - Records nunca se mutan, se reemplazan
   - Structural sharing para memoria eficiente
   - Batch updates para performance

#### Puerto a Rust - Record System

```rust
// core/src/records/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Identificador único de un record
/// Usando newtype pattern para type-safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(String);

impl RecordId {
    pub fn new(id: String) -> Self {
        // Validar formato del ID (nanoid-style)
        assert!(id.len() >= 10, "Record ID too short");
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Fractional index para z-order sin conflictos
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FractionalIndex(String);

impl FractionalIndex {
    /// Genera un índice entre dos índices existentes
    pub fn between(a: Option<&Self>, b: Option<&Self>) -> Self {
        match (a, b) {
            (None, None) => Self("a0".to_string()),
            (None, Some(b)) => Self::decrement(b),
            (Some(a), None) => Self::increment(a),
            (Some(a), Some(b)) => Self::midpoint(a, b),
        }
    }
    
    fn increment(s: &Self) -> Self {
        // Implementación simplificada
        Self(format!("{}0", s.0))
    }
    
    fn decrement(s: &Self) -> Self {
        if s.0.ends_with('0') {
            Self(s.0[..s.0.len()-1].to_string())
        } else {
            Self(format!("{}0", s.0))
        }
    }
    
    fn midpoint(a: &Self, b: &Self) -> Self {
        // Algoritmo jittered-fractional-indexing
        // Implementación real usa base-26 arithmetic
        use jittered_fractional_indexing::generate_key_between;
        Self(generate_key_between(Some(&a.0), Some(&b.0)))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Trait base que todos los records deben implementar
pub trait Record: Send + Sync {
    fn id(&self) -> &RecordId;
    fn type_name(&self) -> &str;
    fn index(&self) -> &FractionalIndex;
    
    /// Clona el record actualizando el índice
    fn with_index(&self, index: FractionalIndex) -> Self where Self: Sized;
}

/// Record genérico que puede contener cualquier tipo serializable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericRecord {
    pub id: RecordId,
    pub type_name: String,
    pub index: FractionalIndex,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl Record for GenericRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }
    
    fn type_name(&self) -> &str {
        &self.type_name
    }
    
    fn index(&self) -> &FractionalIndex {
        &self.index
    }
    
    fn with_index(&self, index: FractionalIndex) -> Self {
        let mut clone = self.clone();
        clone.index = index;
        clone
    }
}

/// Store centralizado de records con undo/redo
pub struct Store<T: Record + Clone> {
    /// HashMap para lookup O(1) por ID
    records: HashMap<RecordId, T>,
    
    /// Vec ordenado por índice para rendering
    sorted_records: Vec<T>,
    
    /// Historia para undo/redo
    history: Vec<Snapshot<T>>,
    current_index: usize,
    
    /// Configuración
    max_history: usize,
    dirty: bool,
}

impl<T: Record + Clone> Store<T> {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            sorted_records: Vec::new(),
            history: vec![Snapshot::default()],
            current_index: 0,
            max_history: 100,
            dirty: false,
        }
    }
    
    /// Inserta o reemplaza un record
    pub fn put(&mut self, record: T) {
        let id = record.id().clone();
        let old_record = self.records.insert(id.clone(), record.clone());
        
        // Actualizar sorted_records
        if let Some(old) = old_record {
            // Remover versión anterior
            self.sorted_records.retain(|r| r.id() != &id);
        }
        
        // Insertar en orden correcto (binary search + insert)
        let index = self.sorted_records
            .binary_search_by(|r| r.index().as_str().cmp(record.index().as_str()))
            .unwrap_or_else(|x| x);
        
        self.sorted_records.insert(index, record);
        
        self.save_snapshot();
    }
    
    /// Obtiene un record por ID
    pub fn get(&self, id: &RecordId) -> Option<&T> {
        self.records.get(id)
    }
    
    /// Obtiene records ordenados por z-index (para rendering)
    pub fn iter_sorted(&self) -> impl Iterator<Item = &T> {
        self.sorted_records.iter()
    }
    
    /// Undo
    pub fn undo(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.restore_from_history();
            return true;
        }
        false
    }
    
    /// Redo
    pub fn redo(&mut self) -> bool {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            self.restore_from_history();
            return true;
        }
        false
    }
    
    fn save_snapshot(&mut self) {
        let snapshot = Snapshot {
            records: self.records.clone(),
            sorted_records: self.sorted_records.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        
        // Truncate redo history
        self.history.truncate(self.current_index + 1);
        self.history.push(snapshot);
        
        // Limit history size
        if self.history.len() > self.max_history {
            self.history.remove(0);
            self.current_index = self.current_index.saturating_sub(1);
        } else {
            self.current_index += 1;
        }
        
        self.dirty = true;
    }
    
    fn restore_from_history(&mut self) {
        if let Some(snapshot) = self.history.get(self.current_index) {
            self.records = snapshot.records.clone();
            self.sorted_records = snapshot.sorted_records.clone();
            self.dirty = true;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snapshot<T: Record + Clone> {
    records: HashMap<RecordId, T>,
    sorted_records: Vec<T>,
    timestamp: i64,
}

impl<T: Record + Clone> Default for Snapshot<T> {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            sorted_records: Vec::new(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
```

### 2.3 Sistema de Utilidades - Análisis

tldraw tiene un paquete de utilidades extremadamente optimizado. Analizamos 66 archivos del core:

#### Array Utilities (array.ts)

```typescript
// Funciones clave identificadas:

// 1. Rotación de arrays (usado para reordenamiento)
function rotateArray<T>(arr: T[], offset: number): T[]

// 2. Deduplicación con función de igualdad custom
function dedupe<T>(input: T[], equals?: (a: any, b: any) => boolean): T[]

// 3. Partición de arrays
function partition<T>(arr: T[], predicate: (item: T) => boolean): [T[], T[]]

// 4. Equality check optimizado
function areArraysShallowEqual<T>(arr1: readonly T[], arr2: readonly T[]): boolean
```

**Puerto a Rust:**

```rust
// core/src/utils/array.rs

use std::cmp::Ordering;

/// Rotación de array - O(n) tiempo, O(1) espacio extra
pub fn rotate<T: Clone>(arr: &[T], offset: isize) -> Vec<T> {
    if arr.is_empty() {
        return Vec::new();
    }
    
    let len = arr.len() as isize;
    let offset = ((offset % len) + len) % len; // Handle negative offset
    
    let mut result = Vec::with_capacity(arr.len());
    
    // Segunda mitad
    result.extend_from_slice(&arr[offset as usize..]);
    // Primera mitad
    result.extend_from_slice(&arr[..offset as usize]);
    
    result
}

/// Deduplicación preservando orden
pub fn dedupe<T: Clone, F: Fn(&T, &T) -> bool>(
    input: &[T],
    equals: F,
) -> Vec<T> {
    if input.is_empty() {
        return Vec::new();
    }
    
    let mut result = Vec::with_capacity(input.len());
    
    for item in input {
        if !result.iter().any(|existing| equals(existing, item)) {
            result.push(item.clone());
        }
    }
    
    result
}

/// Partición de array por predicado
pub fn partition<T, F: Fn(&T) -> bool>(arr: &[T], predicate: F) -> (Vec<&T>, Vec<&T>) {
    let mut matching = Vec::new();
    let mut non_matching = Vec::new();
    
    for item in arr {
        if predicate(item) {
            matching.push(item);
        } else {
            non_matching.push(item);
        }
    }
    
    (matching, non_matching)
}

/// Shallow equality check - O(n)
pub fn are_shallow_equal<T: PartialEq>(arr1: &[T], arr2: &[T]) -> bool {
    if arr1.len() != arr2.len() {
        return false;
    }
    
    // std::slice::cmp::Slice::contains es más optimizado
    arr1 == arr2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate() {
        let arr = vec![1, 2, 3, 4, 5];
        assert_eq!(rotate(&arr, 2), vec![3, 4, 5, 1, 2]);
        assert_eq!(rotate(&arr, -1), vec![2, 3, 4, 5, 1]);
    }

    #[test]
    fn test_dedupe() {
        let arr = vec![1, 2, 2, 3, 1];
        assert_eq!(dedupe(&arr, |a, b| a == b), vec![1, 2, 3]);
    }

    #[test]
    fn test_partition() {
        let arr = vec![1, 2, 3, 4, 5, 6];
        let (evens, odds) = partition(&arr, |x| x % 2 == 0);
        assert_eq!(evens, vec![&2, &4, &6]);
        assert_eq!(odds, vec![&1, &3, &5]);
    }
}
```

#### Object Utilities (object.ts)

```typescript
// Funciones clave:

// 1. Safe hasOwnProperty
function hasOwnProperty(obj: object, key: string): boolean

// 2. Type-preserving Object.entries
function objectMapEntries<Obj extends object>(obj: Obj): Array<[keyof Obj, Obj[keyof Obj]]>

// 3. Type-preserving Object.fromEntries
function objectMapFromEntries<Key extends string, Value>(
    entries: ReadonlyArray<readonly [Key, Value]>
): Record<Key, Value>

// 4. Filter entries efficiently
function filterEntries<Key extends string, Value>(
    object: { [K in Key]: Value },
    predicate: (key: Key, value: Value) => boolean
): { [K in Key]?: Value }

// 5. GroupBy optimizado
function groupBy<K extends string, V>(
    array: ReadonlyArray<V>,
    keySelector: (value: V) => K
): Record<K, V[]>
```

**Patrón importante:** tldraw usa `objectMapEntries` y `objectMapFromEntries` para preservar tipos de TypeScript. En Rust esto es nativo con `Iterator`.

#### Control Flow (control.ts)

```typescript
// Result type para error handling sin excepciones
export type Result<T, E> = OkResult<T> | ErrorResult<E>

interface OkResult<T> {
    readonly ok: true
    readonly value: T
}

interface ErrorResult<E> {
    readonly ok: false
    readonly error: E
}

// Promise con resolve/reduce expuestos
export function promiseWithResolve<T>(): Promise<T> & {
    resolve(value: T): void
    reject(reason?: any): void
}

// Exhaustive switch error
export function exhaustiveSwitchError(value: never, property?: string): never
```

**Insight:** Rust ya tiene `Result<T, E>` nativo, lo que hace esta traducción trivial. El `promiseWithResolve` se traduce a `oneshot` channel en Rust async.

#### Math Utilities (number.ts)

```typescript
// Interpolación lineal
export function lerp(a: number, b: number, t: number): number

// Interpolación inversa
export function invLerp(a: number, b: number, t: number): number

// Modulación entre rangos
export function modulate(
    value: number, 
    rangeA: number[], 
    rangeB: number[], 
    clamp = false
): number

// RNG con seed (xorshift)
export function rng(seed = '')
```

**Puerto a Rust:**

```rust
// core/src/math/interpolation.rs

/// Linear interpolation
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Inverse linear interpolation
#[inline]
pub fn inv_lerp(a: f32, b: f32, t: f32) -> f32 {
    (t - a) / (b - a)
}

/// Modulate value from range A to range B
pub fn modulate(value: f32, range_a: (f32, f32), range_b: (f32, f32), clamp: bool) -> f32 {
    let t = inv_lerp(range_a.0, range_a.1, value);
    let result = lerp(range_b.0, range_b.1, t);
    
    if clamp {
        result.min(range_b.1).max(range_b.0)
    } else {
        result
    }
}

/// Seeded RNG usando xorshift (determinístico para tests)
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: &str) -> Self {
        // Hash del seed para obtener estado inicial
        let mut hash = 0u32;
        for byte in seed.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte);
        }
        Self { state: hash }
    }
    
    /// Genera número f32 entre -1 y 1
    pub fn gen(&mut self) -> f32 {
        // Xorshift algorithm
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        
        // Normalizar a [-1, 1]
        (self.state as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}
```

### 2.4 ExecutionQueue - Async Orchestration

tldraw tiene un `ExecutionQueue` muy interesante para ejecutar tareas secuenciales:

```typescript
export class ExecutionQueue {
    private queue: Array<() => Promise<void>> = []
    private isExecuting = false
    
    constructor(private readonly timeout?: number) {}
    
    async push<T>(task: () => T): Promise<Awaited<T>> {
        return new Promise((resolve, reject) => {
            this.queue.push(async () => {
                try {
                    const result = await task()
                    resolve(result)
                } catch (e) {
                    reject(e)
                }
            })
            this.process()
        })
    }
    
    close(): void {
        this.queue = []
    }
}
```

**Caso de uso:** Rate limiting de operaciones, prevenir race conditions en updates, controlar flujo de animaciones.

**Puerto a Rust:**

```rust
// core/src/async/queue.rs

use std::sync::{Arc, Mutex};
use std::future::Future;
use tokio::sync::{Semaphore, oneshot};
use tokio::task::JoinSet;

pub struct ExecutionQueue {
    queue: Arc<Mutex<Vec<TaskEntry>>>,
    semaphore: Arc<Semaphore>,
    is_executing: Arc<Mutex<bool>>,
}

struct TaskEntry {
    task: Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send>,
    tx: oneshot::Sender<Result<(), Error>>,
}

impl ExecutionQueue {
    pub fn new(concurrency: usize, timeout_ms: Option<u64>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(concurrency)),
            is_executing: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn push<F, Fut>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        
        {
            let mut queue = self.queue.lock().await;
            queue.push(TaskEntry {
                task: Box::new(move || Box::pin(f())),
                tx,
            });
        }
        
        self.process().await?;
        rx.await.map_err(|_| Error::Canceled)?
    }
    
    async fn process(&self) -> Result<(), Error> {
        let mut is_executing = self.is_executing.lock().await;
        if *is_executing {
            return Ok(());
        }
        *is_executing = true;
        drop(is_executing);
        
        loop {
            let task = {
                let mut queue = self.queue.lock().await;
                queue.pop()
            };
            
            match task {
                Some(entry) => {
                    let result = (entry.task)().await;
                    let _ = entry.tx.send(result);
                }
                None => break,
            }
        }
        
        let mut is_executing = self.is_executing.lock().await;
        *is_executing = false;
        Ok(())
    }
}
```

### 2.5 PerformanceTracker - Métricas

tldraw tiene un `PerformanceTracker` específico para medir FPS durante operaciones:

```typescript
export class PerformanceTracker {
    private rafId: number | null = null
    private startTime: number = 0
    private frames: number = 0
    
    start(name: string): void {
        this.startTime = performance.now()
        this.frames = 0
        this.rafId = requestAnimationFrame(this.recordFrame)
    }
    
    stop(): void {
        const duration = performance.now() - this.startTime
        const fps = Math.round((this.frames / duration) * 1000)
        console.log(`Perf ${this.name} ${fps} fps`)
    }
    
    private recordFrame = () => {
        this.frames++
        if (this.isStarted()) {
            this.rafId = requestAnimationFrame(this.recordFrame)
        }
    }
}
```

**Insight:** Pattern de medición que debemos copiar - mide FPS durante operaciones críticas (drag, zoom, pan).

---

## 3. Análisis Profundo: React Flow

### 3.1 Arquitectura de React Flow

React Flow está especializado en diagramas basados en nodos (flowcharts, pipelines):

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      REACT FLOW ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  NODES LAYER                                                       │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Node: { id, type, position, data }                             │ │
│  │  • Handle: puntos de conexión (source/target)                     │ │
│  │  • Draggable, selectable                                          │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  EDGES LAYER                                                       │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Edge: { id, source, target, sourceHandle, targetHandle }      │ │
│  │  • Path: Bezier, Straight, Step, SmoothStep                       │ │
│  │  • Animated, labelable                                            │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  STATE MANAGEMENT                                                  │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Zustand store                                                  │ │
│  │  • onNodesChange, onEdgesChange                                   │ │
│  │  • NodeChange, EdgeChange (discriminated unions)                  │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Sistema de Cambios (Changes)

**Patrón brillante** de React Flow: eventos de cambio tipados:

```typescript
// Cambios que pueden ocurrir en un nodo
export type NodeChange<NodeType extends NodeBase = NodeBase> =
  | NodePositionChange      // { id, type: 'position', position, dragging }
  | NodeDimensionChange     // { id, type: 'dimensions', dimensions, resizing }
  | NodeSelectionChange     // { id, type: 'select', selected }
  | NodeRemoveChange        // { id, type: 'remove' }
  | NodeAddChange<NodeType> // { item, type: 'add', index }
  | NodeReplaceChange<NodeType> // { id, item, type: 'replace' }
```

**Insight:** Este patrón es perfecto para nuestro sistema de records - cada cambio es un evento inmutable que podemos almacenar, reproducir, o deshacer.

### 3.3 Cálculo de Conexiones (Edges)

React Flow tiene algoritmos muy optimizados para calcular paths de conexiones:

```typescript
// Bezier curve para edges
export function getBezierPath({
  sourceX, sourceY,
  targetX, targetY,
  sourcePosition = Position.Left,
  targetPosition = Position.Right,
  curvature = 0.25
}): string {
  
  // Calcular puntos de control basados en posiciones
  const [sourceControlX, sourceControlY] = getControlPoints(
    sourceX, sourceY, sourcePosition, curvature
  )
  const [targetControlX, targetControlY] = getControlPoints(
    targetX, targetY, targetPosition, curvature
  )
  
  return `M ${sourceX} ${sourceY} C ${sourceControlX} ${sourceControlY}, ${targetControlX} ${targetControlY}, ${targetX} ${targetY}`
}
```

**Puerto a Rust:**

```rust
// core/src/connections/bezier.rs

use crate::geometry::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandlePosition {
    Top,
    Right,
    Bottom,
    Left,
}

/// Calcula curva de Bézier cúbica para conexión
pub fn bezier_path(
    source: Vec2,
    target: Vec2,
    source_pos: HandlePosition,
    target_pos: HandlePosition,
    curvature: f32,
) -> [Vec2; 4] {
    // Offset para puntos de control
    let control_offset = 50.0 * curvature;
    
    let source_ctrl = match source_pos {
        HandlePosition::Top => Vec2::new(source.x, source.y - control_offset),
        HandlePosition::Right => Vec2::new(source.x + control_offset, source.y),
        HandlePosition::Bottom => Vec2::new(source.x, source.y + control_offset),
        HandlePosition::Left => Vec2::new(source.x - control_offset, source.y),
    };
    
    let target_ctrl = match target_pos {
        HandlePosition::Top => Vec2::new(target.x, target.y - control_offset),
        HandlePosition::Right => Vec2::new(target.x + control_offset, target.y),
        HandlePosition::Bottom => Vec2::new(target.x, target.y + control_offset),
        HandlePosition::Left => Vec2::new(target.x - control_offset, target.y),
    };
    
    [source, source_ctrl, target_ctrl, target]
}

/// Genera string de path SVG (para debugging o export)
pub fn bezier_path_svg(bezier: &[Vec2; 4]) -> String {
    format!(
        "M {} {} C {} {}, {} {}, {} {}",
        bezier[0].x, bezier[0].y,
        bezier[1].x, bezier[1].y,
        bezier[2].x, bezier[2].y,
        bezier[3].x, bezier[3].y,
    )
}

/// Evalúa punto en curva de Bézier en t [0, 1]
pub fn eval_bezier(bezier: &[Vec2; 4], t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    Vec2::new(
        mt3 * bezier[0].x + 3.0 * mt2 * t * bezier[1].x + 3.0 * mt * t2 * bezier[2].x + t3 * bezier[3].x,
        mt3 * bezier[0].y + 3.0 * mt2 * t * bezier[1].y + 3.0 * mt * t2 * bezier[2].y + t3 * bezier[3].y,
    )
}
```

---

## 4. Análisis Profundo: Excalidraw

### 4.1 Arquitectura de Excalidraw

Excalidraw usa una técnica de **doble canvas** que es clave para su performance:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    EXCALIDRAW DUAL CANVAS                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  STATIC LAYER (Canvas 1)                                          │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Elementos que no cambian frecuentemente                        │ │
│  │  • Se cachea y solo se re-renderiza cuando hay cambios           │ │
│  │  • ~70% del contenido en diagramas típicos                       │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  INTERACTIVE LAYER (Canvas 2)                                     │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • Elemento siendo editado/dibujado                              │ │
│  │  • Selection boxes                                                │ │
│  │  • Guides, handles                                                │ │
│  │  • Se re-renderiza cada frame (60fps)                             │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  RESULTADO: Solo ~30% del canvas se re-renderiza cada frame           │
│             Reducción de 71% en operaciones de dibujo                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Viewport Culling

Excalidraw tiene un viewport culling muy agresivo:

```typescript
// Elementos fuera del viewport NO se renderizan
function getVisibleElements(
    elements: ExcalidrawElement[],
    appState: AppState
): ExcalidrawElement[] {
    
    const { scrollX, scrollY, width, height, zoom } = appState
    
    // Convertir viewport a world coordinates
    const viewportBounds = {
        x: -scrollX / zoom,
        y: -scrollY / zoom,
        w: width / zoom,
        h: height / zoom
    }
    
    // Filtrar elementos que intersectan con viewport
    return elements.filter(el => 
        !el.isDeleted && 
        !el.isLocked &&
        elementIntersectsBounds(el, viewportBounds)
    )
}
```

**Puerto a Rust con Spatial Index:**

```rust
// core/src/scene/viewport.rs

use crate::geometry::Bounds;
use crate::spatial::QuadTree;

pub struct Viewport {
    /// Posición del viewport en world coordinates
    pub x: f32,
    pub y: f32,
    /// Nivel de zoom
    pub zoom: f32,
    /// Tamaño del viewport en screen pixels
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            width,
            height,
        }
    }
    
    /// Convierte screen coordinates a world coordinates
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        (
            (screen_x - self.x) / self.zoom,
            (screen_y - self.y) / self.zoom,
        )
    }
    
    /// Convierte world coordinates a screen coordinates
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (
            world_x * self.zoom + self.x,
            world_y * self.zoom + self.y,
        )
    }
    
    /// Obtiene bounds del viewport en world coordinates
    pub fn world_bounds(&self) -> Bounds {
        let (min_x, min_y) = self.screen_to_world(0.0, 0.0);
        let (max_x, max_y) = self.screen_to_world(self.width, self.height);
        
        Bounds {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
    
    /// Filtra elementos visibles usando QuadTree
    pub fn filter_visible<T: SpatialObject>(
        &self,
        elements: &[T],
        quadtree: &QuadTree<T>,
    ) -> Vec<&T> {
        let world_bounds = self.world_bounds();
        
        // Query al QuadTree - O(log n + k) donde k = elementos encontrados
        quadtree.query(&world_bounds)
    }
}

/// Trait para objetos con bounds espaciales
pub trait SpatialObject {
    fn bounds(&self) -> Bounds;
    fn is_visible(&self) -> bool {
        true
    }
}
```

### 4.3 Optimizaciones de Rendering de Excalidraw

1. **Dirty Flag Pattern**
   - Solo se re-renderiza lo que cambió
   - Cada elemento tiene un flag `version`
   - Se cachea la última versión renderizada

2. **Canvas Caching**
   - Elementos complejos se cachean en offscreen canvas
   - Text especialmente costoso se cachea
   - Imágenes se cachean en múltiples escalas

3. **Spatial Indexing**
   - Grid espacial para hit testing
   - Reduce O(n) a O(1) para clicks
   - Se reconstruye incrementalmente

---

## 5. Arquitectura del Motor Propuesto

Basándonos en el análisis, proponemos esta arquitectura:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ARCHFLOW ENGINE ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  JAVASCRIPT BRIDGE LAYER                                           │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │  • wasm-bindgen exports                                           │ │
│  │  • Event handling (DOM events → WASM)                             │ │
│  │  • Canvas element management                                      │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  CORE LAYER (Rust)                                                 │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │                                                                   │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  RECORD SYSTEM (from tldraw)                                │ │ │
│  │  │  • Immutable records                                        │ │ │
│  │  │  • Store with undo/redo                                     │ │ │
│  │  │  • Fractional indexing                                      │ │ │
│  │  │  • Transactions                                             │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  GEOMETRY LAYER                                              │ │ │
│  │  │  • Vec2, Vec3, Vec4, Mat4                                   │ │ │
│  │  │  • Bounds, AABB, Circle                                     │ │ │
│  │  │  • Intersection tests                                       │ │ │
│  │  │  • Spatial index (QuadTree)                                 │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  SCENE GRAPH (ECS-based)                                    │ │ │
│  │  │  • Entity: nodos, conexiones, grupos                        │ │ │
│  │  │  • Components: Transform, Renderable, Style                │ │ │
│  │  │  • Systems: Update, Render, Animate                        │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  ANIMATION LAYER                                             │ │ │
│  │  │  • Easing functions                                         │ │ │
│  │  │  • Tween engine                                              │ │ │
│  │  │  • Timeline                                                  │ │ │
│  │  │  • Keyframe system                                           │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                                                                   │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                ↕                                        │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │  RENDERING LAYER (WebGPU)                                          │ │
│  │  ────────────────────────────────────────────────────────────────│ │
│  │                                                                   │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  RENDERER                                                    │ │ │
│  │  │  • WebGPU device + queue                                     │ │ │
│  │  │  • Pipeline management                                       │ │ │
│  │  │  • Command buffer encoding                                   │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  BATCHING SYSTEM                                             │ │ │
│  │  │  • Batch by type (rectangles, ellipses, etc.)               │ │ │
│  │  │  • Batch by material (color, texture)                        │ │ │
│  │  │  • Instanced rendering                                       │ │ │
│  │  │  • Vertex/index buffers                                      │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  TEXT RENDERING                                              │ │ │
│  │  │  • SDF text atlas                                            │ │ │
│  │  │  • Glyph cache                                               │ │ │
│  │  │  • Dynamic font loading                                      │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                          ↕                                        │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │  SHADERS (WGSL)                                             │ │ │
│  │  │  • shape.wgsl (SDF rendering)                               │ │ │
│  │  │  • text.wgsl (SDF text)                                     │ │ │
│  │  │  • connection.wgsl (bezier curves)                          │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  │                                                                   │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Rendering WebGPU

### 6.1 Pipeline de Rendering

WebGPU permite un pipeline muy eficiente:

```rust
// renderer/src/webgpu/pipeline.rs

use wgpu::*;

pub struct ShapePipeline {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
}

impl ShapePipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        // Shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shape Shader"),
            source: ShaderSource::Wgsl(include_str!("shape.wgsl").into()),
        });
        
        // Vertex buffer layout
        let vertex_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2, // position
                1 => Float32x4, // color
                2 => Float32,   // corner_radius
                3 => Float32x2, // size
            ],
        };
        
        // Pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        // ...
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ShapeVertex {
    position: [f32; 2],
    color: [f32; 4],
    corner_radius: f32,
    size: [f32; 2],
}
```

### 6.2 SDF Rendering para Shapes

**Idea clave:** Usar Signed Distance Fields en el fragment shader para rendering de shapes. Esto permite:

- Rounded rectangles perfectos a cualquier resolución
- Anti-aliasing de alta calidad
- Zoom sin pixelación
- Shapes procedurales sin geometría compleja

```wgsl
// shaders/shape.wgsl

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) corner_radius: f32,
    @location(3) size: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) corner_radius: f32,
    @location(3) size: vec2<f32>,
}

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.projection * vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.corner_radius = input.corner_radius;
    output.size = input.size;
    output.uv = input.position;
    return output;
}

// Signed Distance Function para rounded rectangle
fn sd_rounded_box(p: vec2<f32>, size: vec2<f32>, r: f32) -> f32 {
    let half_size = size * 0.5;
    let q = abs(p) - half_size + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calcular distancia desde centro
    let center_offset = input.uv - input.size * 0.5;
    let dist = sd_rounded_box(center_offset, input.size, input.corner_radius);
    
    // Anti-aliasing con smoothstep
    let aa_width = fwidth(dist);
    let alpha = 1.0 - smoothstep(-aa_width, aa_width, dist);
    
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
```

### 6.3 Instanced Rendering

Para maximizar performance con 10k+ elementos:

```rust
// renderer/src/webgpu/instanced.rs

pub struct InstancedRenderer {
    pipeline: RenderPipeline,
    instance_buffer: Buffer,
    vertex_buffer: Buffer,
}

impl InstancedRenderer {
    pub fn render_instances(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        instances: &[ShapeInstance],
    ) {
        // Actualizar buffer de instancias
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Instanced Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color { r: 0.95, g: 0.95, b: 0.95, a: 1.0 }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        
        // Draw instanced
        render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ShapeInstance {
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    corner_radius: f32,
    rotation: f32,
}
```

---

## 7. Sistema ECS Considerado

### 7.1 ¿Por qué ECS?

ECS (Entity Component System) es ideal para motores gráficos porque:

- **Cache locality:** Componentes del mismo tipo se almacenan contiguamente
- **Paralelización:** Systems pueden ejecutar en paralelo
- **Data-oriented:** Mejor performance que OOP para motores
- **Composibilidad:** Fácil añadir funcionalidad

### 7.2 Librerías ECS en Rust

| Librería | Pros | Contras | Veredicto |
|----------|------|---------|-----------|
| **Bevy** | Feature-rich, ECS + Renderer | Mucho boilerplate | ❌ Overkill |
| **Hecs** | Simple, rápido | Sin paralelización | ⚠️ MVP ok |
| **Legion** | Paralelización, estable | Más complejo | ✅ **Recomendado** |
| **Specs** | Probado en juegos | Descontinuado | ❌ No usar |

**Recomendación:** `Legion` para balance entre performance y usabilidad.

### 7.3 Diseño ECS para ArchFlow

```rust
// core/src/ecs/mod.rs

use legion::*;

/// Entity: Identificador único
pub type Entity = legion::Entity;

/// Components

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

#[derive(Debug, Clone)]
pub struct Renderable {
    pub shape_type: ShapeType,
    pub color: Color,
    pub corner_radius: f32,
}

#[derive(Debug, Clone)]
pub struct Connections {
    pub sources: Vec<Entity>,
    pub targets: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub tween: Tween,
    pub elapsed: f32,
}

/// Systems

pub fn update_transforms(world: &mut World) {
    let mut query = <(Entity, &mut Transform, &Animation)>::query();
    
    for (entity, transform, animation) in query.iter_mut(world) {
        // Actualizar transform basado en animación
        if let Some(value) = animation.tween.sample(animation.elapsed) {
            transform.position = value.position;
        }
    }
}

pub fn cull_invisible(world: &mut World, viewport: &Viewport) {
    let mut query = <(Entity, &Transform, &mut Visible)>::query();
    
    for (entity, transform, mut visible) in query.iter_mut(world) {
        let in_viewport = viewport.contains(transform.position);
        visible.0 = in_viewport;
    }
}
```

---

## 8. Implementación en Rust

### 8.1 Estructura del Proyecto

```
archflow-engine/
├── Cargo.toml
├── packages/
│   ├── core/              # Librería core (no WASM)
│   │   ├── src/
│   │   │   ├── records/   # Record system
│   │   │   ├── geometry/  # Math
│   │   │   ├── ecs/       # ECS setup
│   │   │   ├── scene/     # Scene graph
│   │   │   └── animation/ # Animation
│   │   └── Cargo.toml
│   │
│   ├── renderer/          # WebGPU renderer
│   │   ├── src/
│   │   │   ├── webgpu/
│   │   │   ├── shaders/
│   │   │   └── batching/
│   │   └── Cargo.toml
│   │
│   └── wasm/              # WASM bindings
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
│
└── examples/
    └── simple/
        └── index.html
```

### 8.2 Cargo.toml

```toml
[workspace]
members = ["packages/*"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
wgpu = "0.19"
legion = "0.4"
nalgebra = "0.32"
bytemuck = "1.14"
```

---

## 9. Roadmap de Implementación

### Fase 1: Fundamentos (4 semanas)

| Semana | Tareas | Entregable |
|--------|--------|------------|
| 1 | Record system + Store | Records con undo/redo |
| 2 | Geometry + Math utils | Vec2, Bounds, Intersection |
| 3 | ECS setup (Legion) | Components + Systems básicos |
| 4 | Scene graph básico | Entities, Transform, Visible |

### Fase 2: Rendering (6 semanas)

| Semana | Tareas | Entregable |
|--------|--------|------------|
| 5 | WebGPU setup + Pipeline | Clear color background |
| 6 | Shape rendering (SDF) | Rectangles, Ellipses |
| 7 | Instanced rendering | 1k shapes @ 60fps |
| 8 | Text rendering (SDF atlas) | Glyph cache + rendering |
| 9 | Batching system | Batch by type/material |
| 10 | Viewport culling | Solo visible elements |

### Fase 3: Features Avanzadas (4 semanas)

| Semana | Tareas | Entregable |
|--------|--------|------------|
| 11 | Connections (Bezier) | Arrows/Edges |
| 12 | Animation system | Tween + Easing |
| 13 | WASM bindings | JavaScript API |
| 14 | Optimización + Benchmark | 10k elements @ 60fps |

---

**FIN DEL DOCUMENTO**
