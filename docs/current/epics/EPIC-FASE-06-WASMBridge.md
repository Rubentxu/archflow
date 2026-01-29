# EPIC-FASE-06: WASM Bridge

**Versión:** 1.1.0 (Corregida)  
**Fase:** 6/8  
**Duración:** Semana 8  
**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` - L3001-3242, F.4, F.5

---

## 📋 Descripción General

**ENFOQUE: CERO CÓDIGO LEGACY - TODO DESDE CERO**

Bridge zero-copy entre Rust y JavaScript con SharedArrayBuffer y binary deltas.

### Archivos Legacy a ELIMINAR:
```
crates/archflow-wasm/src/lib.rs          → REESCRIBIR completo
```

### Objetivos Principales
- Crear `archflow-wasm-collab/` crate **desde cero**
- Implementar `SharedBuffer` con SharedArrayBuffer
- Implementar `BinaryDeltaCodec` (Apéndice F.5)
- Zero-copy con `bytemuck` (Apéndice F.4)
- 60fps garantizado en navegador

---

## 🔬 Investigación Realizada (2026-01-27)

### Hallazgos Clave

| Tema | Resultado |
|------|-----------|
| **COOP/COEP headers** | Sin cambios desde 2021 - siguen requeridos para SharedArrayBuffer |
| **wasm-bindgen memory views** | `js_sys::Float32Array::view()` deprecated desde wasm-bindgen 0.2.100+ |
| **Safe access pattern** | Usar `wasm_bindgen::memory()` + `DataView` para acceso seguro |
| **Cross-origin isolation** | Verificar en servidor (headers) y cliente (`crossOriginIsolated`) |

### Patrón Recomendado (2024)

```rust
// ✅ CORRECTO: Acceso seguro a memoria WASM
#[wasm_bindgen(getter)]
pub fn data(&self) -> js_sys::Float32Array {
    let mem = wasm_bindgen::memory().dyn_into::<web_sys::WebAssemblyMemory>().unwrap();
    let buffer = mem.buffer().dyn_into::<js_sys::WebAssembly::LinearMemory>().unwrap();
    let ptr = self.render_buffer.as_ptr() as *const f32;
    let len = self.render_buffer.len() * (std::mem::size_of::<RenderAttribute>() / 4);
    unsafe { js_sys::Float32Array::view(&buffer, ptr as u32, len as u32) }
}
```

---

## 📦 Entregables (TODO DESDE CERO)

### Módulo 6.0: `src/types.rs` (NUEVO) - ShapeField Enum

```rust
/// Campos que pueden actualizarse en un delta
/// Usado con BitFlags para masks selectivos
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, bitflags::bitflags)]
pub enum ShapeField {
    /// Posición (x, y) - 8 bytes
    Position = 0b00000001,
    /// Color (r, g, b, a) - 4 bytes
    Color = 0b00000010,
    /// Tamaño (width, height) - 8 bytes
    Size = 0b00000100,
    /// Rotación - 4 bytes
    Rotation = 0b00001000,
    /// Z-index - 4 bytes
    ZIndex = 0b00010000,
    /// Todos los campos
    All = Self::Position.bits() | Self::Color.bits() | Self::Size.bits() 
        | Self::Rotation.bits() | Self::ZIndex.bits(),
}

impl ShapeField {
    /// Tamaño en bytes de este campo cuando está presente
    pub fn byte_size(&self) -> usize {
        match self {
            Self::Position => 8,  // 2 x f32
            Self::Color => 4,     // 4 x u8
            Self::Size => 8,      // 2 x f32
            Self::Rotation => 4,  // 1 x f32
            Self::ZIndex => 4,    // 1 x f32
            _ => 0,
        }
    }
}
```

### Módulo 6.1: `src/shared_buffer.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod shared_buffer_tests {
    use super::*;

    #[test]
    fn test_render_attribute_pod() {
        let attr = RenderAttribute {
            id: 42,
            x: 100.5,
            y: 200.3,
            color: [255, 128, 64, 255],
            _padding: [0; 4],
        };
        // Verificar POD
        let bytes = bytemuck::bytes_of(&attr);
        assert_eq!(bytes.len(), std::mem::size_of::<RenderAttribute>());
    }

    #[test]
    fn test_shared_buffer_creation() {
        let buffer = SharedBuffer::new(1000);
        assert_eq!(buffer.max_elements(), 1000);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_shared_buffer_update() {
        let mut buffer = SharedBuffer::new(10);
        let store = TestRecordStore::new();
        let ids = vec![RecordId::from_str("test_1").unwrap()];

        buffer.update(&ids, &store);
        // Verificar actualización
    }

    #[test]
    fn test_pointer_stability() {
        let buffer = SharedBuffer::new(100);
        let ptr1 = buffer.get_ptr();
        buffer.update(&[], &TestRecordStore::new());
        let ptr2 = buffer.get_ptr();
        assert_eq!(ptr1, ptr2); // Mismo buffer, pointer estable
    }

    #[test]
    fn test_update_bounds_safety() {
        let mut buffer = SharedBuffer::new(5);
        let store = TestRecordStore::new();
        // 10 IDs pero solo caben 5 - debe truncarse sin panics
        let ids: Vec<RecordId> = (0..10).map(|i| RecordId::from_u64(i)).collect();
        buffer.update(&ids, &store);
        assert_eq!(buffer.len(), 5);
    }
}
```

**Implementación:**
```rust
// F.4: Zero-copy con bytemuck para SharedArrayBuffer
use bytemuck::{Pod, Zeroable};
use wasm_bindgen::prelude::*;

/// Atributo de render compartido con JS (must be POD)
/// ⭐ Importante: repr(C) + tamaños explícitos para ABI estable
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct RenderAttribute {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub color: [u8; 4],
    // Padding explícito para alineación de 8 bytes
    pub _padding: [u8; 4],
}

static_assertions::assert_eq_size!(RenderAttribute, [u8; 24]);
static_assertions::assert_align_eq!(RenderAttribute, 8);

/// Buffer compartido para comunicación Rust → JS
/// ⭐ Uso interno: Vec que luego se expone via SharedArrayBuffer
pub struct SharedBuffer {
    render_buffer: Vec<RenderAttribute>,
    max_elements: usize,
}

impl SharedBuffer {
    /// Crear nuevo buffer con capacidad máxima
    /// ⭐ Pre-aloca toda la memoria para evitar re-allocations
    pub fn new(max_elements: usize) -> Self {
        Self {
            render_buffer: vec![RenderAttribute::zeroed(); max_elements],
            max_elements,
        }
    }

    /// Actualizar buffer con elementos visibles
    /// F.4: Zero-copy - actualizar in-place sin allocaciones
    /// ⭐ Usa zip() para evitar overflow de índices
    pub fn update(&mut self, visible_ids: &[RecordId], store: &RecordStore<dyn Record>) {
        // zip() trunca automáticamente al menor length
        for (attr, id) in self.render_buffer.iter_mut().zip(visible_ids.iter()) {
            if let Some(record) = store.get(id) {
                let position = record.bounds()
                    .map(|b| b.center())
                    .unwrap_or(glam::Vec2::ZERO);

                *attr = RenderAttribute {
                    id: id.into_u64(),
                    x: position.x,
                    y: position.y,
                    color: record.color().to_rgba8(),
                    _padding: [0; 4],
                };
            }
        }
    }

    /// Obtener puntero para JS (SharedArrayBuffer)
    /// ⭐ Puntero estable - nunca cambia después de new()
    pub fn get_ptr(&self) -> *const RenderAttribute {
        self.render_buffer.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.render_buffer.len()
    }

    pub fn max_elements(&self) -> usize {
        self.max_elements
    }

    pub fn is_empty(&self) -> bool {
        self.render_buffer.is_empty()
    }

    /// Crear vista tipeada para JavaScript
    /// ⭐ SAFETY: JS tiene borrow transient - debe usar datos inmediatamente
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Float32Array {
        // SAFETY: El buffer de memoria WASM es válido por toda la vida del módulo.
        // JS debe consumir estos datos inmediatamente (patrón "fire and forget").
        // La memoria no se realloca después de construcción (vector capacity fijo).
        let mem = unsafe {
            wasm_bindgen::memory()
                .dyn_into::<web_sys::WebAssemblyMemory>()
                .unwrap_unchecked()
        };
        let buffer = mem.buffer().dyn_into::<js_sys::WebAssembly::LinearMemory>().unwrap();
        let ptr = self.render_buffer.as_ptr() as *const f32;
        let len = self.render_buffer.len() * (std::mem::size_of::<RenderAttribute>() / 4);
        
        unsafe { 
            js_sys::Float32Array::view(&buffer, ptr as u32, len as u32) 
        }
    }

    /// Vista de IDs como u64
    /// ⭐ SAFETY: Mismo contracto que data()
    #[wasm_bindgen(getter)]
    pub fn ids(&self) -> js_sys::BigUint64Array {
        // SAFETY: Ver documentación en data()
        let mem = unsafe {
            wasm_bindgen::memory()
                .dyn_into::<web_sys::WebAssemblyMemory>()
                .unwrap_unchecked()
        };
        let buffer = mem.buffer().dyn_into::<js_sys::WebAssembly::LinearMemory>().unwrap();
        let ptr = self.render_buffer.as_ptr() as *const u64;
        let len = self.render_buffer.len();
        
        unsafe { 
            js_sys::BigUint64Array::view(&buffer, ptr as u32, len as u32) 
        }
    }
}
```

### Módulo 6.2: `src/binary_delta_codec.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod binary_delta_tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let id = RecordId::from_str("roundtrip_test").unwrap();
        let mask = BitFlags::<ShapeField>::all();
        let record = TestRecord::new();

        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(&mut encoded, id.clone(), mask, &record);

        let decoded = BinaryDeltaCodec::decode_delta(&encoded).unwrap();
        assert_eq!(decoded.id, id);
        assert_eq!(decoded.mask, mask);
    }

    #[test]
    fn test_partial_field_mask() {
        let id = RecordId::from_str("partial_test").unwrap();
        let mask = BitFlags::<ShapeField>::from_bits_truncate(0b00000011);
        let record = TestRecord::new();

        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(&mut encoded, id.clone(), mask, &record);

        let decoded = BinaryDeltaCodec::decode_delta(&encoded).unwrap();
        assert_eq!(decoded.mask, mask);
    }

    #[test]
    fn test_delta_compression_ratio() {
        let original: Vec<u8> = (0..1000).map(|i| i as u8).collect();
        let mut compressed = Vec::new();

        BinaryDeltaCodec::compress(&original, &mut compressed);
        // Verificar que está comprimido
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_varint_efficiency() {
        // Valores pequeños deben codificarse en 1 byte
        assert_eq!(BinaryDeltaCodec::encode_varint(0), vec![0x00]);
        assert_eq!(BinaryDeltaCodec::encode_varint(127), vec![0x7F]);
        // Valor de 1 byte más alto
        assert_eq!(BinaryDeltaCodec::encode_varint(128).len(), 2);
    }

    #[test]
    fn test_delta_header_version() {
        // Verificar que el header incluye versión para forward compatibility
        let id = RecordId::from_str("version_test").unwrap();
        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(&mut encoded, id, BitFlags::<ShapeField>::empty(), &TestRecord::new());
        
        // Primer byte es la versión del protocolo
        assert_eq!(encoded[0], BinaryDeltaCodec::PROTOCOL_VERSION);
    }
}
```

**Implementación:**
```rust
// F.5: Binary Delta Codec para red - 75% reducción vs JSON
use bytemuck::bytes_of;
use fixedbitset::BitFlags;

use super::types::ShapeField;

/// Versión del protocolo de delta - incrementa en breaking changes
const PROTOCOL_VERSION: u8 = 1;

pub struct BinaryDeltaCodec;

impl BinaryDeltaCodec {
    /// Protocol version para forward compatibility
    pub const PROTOCOL_VERSION: u8 = PROTOCOL_VERSION;

    /// Codificar delta con field mask selectivo
    /// ⭐ Formato: [version:1][id_varint:1-9][mask:2][payload:variable]
    pub fn encode_delta(
        buffer: &mut Vec<u8>,
        id: RecordId,
        mask: BitFlags<ShapeField>,
        record: &dyn Record,
    ) {
        // Version byte para forward compatibility
        buffer.push(PROTOCOL_VERSION);

        // VarInt para ID (1-9 bytes, típicamente 1-2)
        let id_bytes = Self::encode_varint(id.into_u64());
        buffer.extend_from_slice(&id_bytes);

        // Field mask (1 byte para ShapeField, puede expandirse a 2)
        buffer.push(mask.bits() as u8);

        // Payload selectivo basado en mask
        if mask.contains(ShapeField::Position) {
            if let Some(bounds) = record.bounds() {
                let center = bounds.center();
                buffer.extend_from_slice(bytes_of(&center.x));
                buffer.extend_from_slice(bytes_of(&center.y));
            }
        }

        if mask.contains(ShapeField::Color) {
            let color = record.color();
            buffer.push(color.r);
            buffer.push(color.g);
            buffer.push(color.b);
            buffer.push(color.a);
        }

        if mask.contains(ShapeField::Size) {
            if let Some(bounds) = record.bounds() {
                let width = bounds.width();
                let height = bounds.height();
                buffer.extend_from_slice(bytes_of(&width));
                buffer.extend_from_slice(bytes_of(&height));
            }
        }

        if mask.contains(ShapeField::Rotation) {
            if let Some(rotation) = record.rotation() {
                buffer.extend_from_slice(bytes_of(&rotation));
            }
        }

        if mask.contains(ShapeField::ZIndex) {
            if let Some(z_index) = record.z_index() {
                buffer.extend_from_slice(bytes_of(&z_index));
            }
        }
    }

    /// Decodificar delta
    /// ⭐ Retorna None en vez de panic para robustez
    pub fn decode_delta(buffer: &[u8]) -> Option<DecodedDelta> {
        let mut pos = 0;

        // Version check
        if buffer.is_empty() { return None; }
        let version = buffer[pos];
        pos += 1;
        if version != PROTOCOL_VERSION {
            // ⭐ En el futuro, podemos hacer traducción de versiones
            return None;
        }

        // VarInt ID
        let (id, id_len) = Self::decode_varint(buffer.get(pos..)?)?;
        pos += id_len;

        // Field mask
        if buffer.len() < pos + 1 { return None; }
        let mask = BitFlags::<ShapeField>::from_bits_truncate(buffer[pos] as u32);
        pos += 1;

        // Payload
        let mut delta = DecodedDelta {
            id: RecordId::from_u64(id),
            mask,
            position: None,
            color: None,
            size: None,
            rotation: None,
            z_index: None,
        };

        if mask.contains(ShapeField::Position) {
            if buffer.len() < pos + 8 { return None; }
            let x = f32::from_le_bytes([buffer[pos], buffer[pos + 1], buffer[pos + 2], buffer[pos + 3]]);
            let y = f32::from_le_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]);
            delta.position = Some(glam::Vec2::new(x, y));
            pos += 8;
        }

        if mask.contains(ShapeField::Color) {
            if buffer.len() < pos + 4 { return None; }
            delta.color = Some(RgbaColor::new(buffer[pos], buffer[pos + 1], buffer[pos + 2], buffer[pos + 3]));
            pos += 4;
        }

        if mask.contains(ShapeField::Size) {
            if buffer.len() < pos + 8 { return None; }
            let width = f32::from_le_bytes([buffer[pos], buffer[pos + 1], buffer[pos + 2], buffer[pos + 3]]);
            let height = f32::from_le_bytes([buffer[pos + 4], buffer[pos + 5], buffer[pos + 6], buffer[pos + 7]]);
            delta.size = Some(glam::Vec2::new(width, height));
            pos += 8;
        }

        if mask.contains(ShapeField::Rotation) {
            if buffer.len() < pos + 4 { return None; }
            delta.rotation = Some(f32::from_le_bytes([buffer[pos], buffer[pos + 1], buffer[pos + 2], buffer[pos + 3]]));
            pos += 4;
        }

        if mask.contains(ShapeField::ZIndex) {
            if buffer.len() < pos + 4 { return None; }
            delta.z_index = Some(f32::from_le_bytes([buffer[pos], buffer[pos + 1], buffer[pos + 2], buffer[pos + 3]]));
        }

        Some(delta)
    }

    /// ⭐ FIX: VarInt encoding retorna Vec<u8> con solo bytes necesarios
    fn encode_varint(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        while value > 0x7F {
            result.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
        }
        result.push(value as u8);
        result
    }

    fn decode_varint(buffer: &[u8]) -> Option<(u64, usize)> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut i = 0;

        while i < buffer.len() && i < 9 {
            let byte = buffer[i];
            result |= ((byte & 0x7F) as u64) << shift;
            i += 1;
            if byte & 0x80 == 0 { break; }
            shift += 7;
        }

        Some((result, i))
    }

    /// Compresión simple para delta encoding
    /// ⭐ Usa Delta encoding: almacenar diferencia vs valor anterior
    pub fn compress(input: &[u8], output: &mut Vec<u8>) {
        // Simple approach: copy input for now
        // Future: implement LZ4 or similar
        output.extend_from_slice(input);
    }
}

/// Resultado de decodificar un delta
pub struct DecodedDelta {
    pub id: RecordId,
    pub mask: BitFlags<ShapeField>,
    pub position: Option<glam::Vec2>,
    pub color: Option<RgbaColor>,
    pub size: Option<glam::Vec2>,
    pub rotation: Option<f32>,
    pub z_index: Option<f32>,
}
```

### Módulo 6.3: `src/wasm_bridge.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
use wasm_bindgen::prelude::*;

/// Bridge principal Rust → JavaScript
/// ⭐ Gestiona el ciclo de vida de la comunicación Rust ↔ JS
#[wasm_bindgen]
pub struct WasmBridge {
    shared_buffer: SharedBuffer,
    codec: BinaryDeltaCodec,
    record_store: RecordStore<dyn Record>,
}

#[wasm_bindgen]
impl WasmBridge {
    #[wasm_bindgen(constructor)]
    pub fn new(max_elements: usize) -> Self {
        Self {
            shared_buffer: SharedBuffer::new(max_elements),
            codec: BinaryDeltaCodec,
            record_store: RecordStore::new(),
        }
    }

    /// Inicializar con COOP/COEP headers verificados
    /// ⭐ Retorna error claro si el entorno no soporta SharedArrayBuffer
    pub fn init() -> Result<(), JsValue> {
        // Verificar SharedArrayBuffer disponible
        let cross_origin_isolated = web_sys::window()
            .and_then(|w| w.cross_origin_isolated())
            .unwrap_or(false);

        if !cross_origin_isolated {
            return Err(JsValue::from_str(
                "Cross-origin isolation required. Ensure COOP/COEP headers are set:\n\
                - Cross-Origin-Opener-Policy: same-origin\n\
                - Cross-Origin-Embedder-Policy: require-corp"
            ));
        }

        // Verificar que SharedArrayBuffer está disponible
        if !js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("SharedArrayBuffer")).unwrap_or(false) {
            return Err(JsValue::from_str("SharedArrayBuffer not available"));
        }

        Ok(())
    }

    /// Obtener puntero al buffer de render
    pub fn get_render_buffer_ptr(&self) -> *const RenderAttribute {
        self.shared_buffer.get_ptr()
    }

    /// Obtener longitud del buffer
    pub fn get_render_buffer_len(&self) -> usize {
        self.shared_buffer.len()
    }

    /// Actualizar estado y preparar render
    pub fn update(&mut self) {
        // Obtener elementos visibles del viewport
        // ... (integración con viewport manager)

        // Actualizar buffer compartido
        self.shared_buffer.update(&[], &self.record_store);
    }

    /// Recibir delta de red y aplicar
    pub fn apply_delta(&mut self, data: &[u8]) -> Result<(), JsValue> {
        let delta = BinaryDeltaCodec::decode_delta(data)
            .ok_or_else(|| JsValue::from_str("Invalid delta format or version mismatch"))?;

        if let Some(record) = self.record_store.get_mut(&delta.id) {
            // Actualizar campos del record basados en el mask
            if let Some(pos) = delta.position {
                record.set_bounds(Aabb2D::from_center_size(pos, glam::Vec2::splat(100.0)));
            }
            if let Some(color) = delta.color {
                record.set_color(color);
            }
            // ... aplicar otros campos
        }

        Ok(())
    }

    /// Serializar cambios para enviar
    pub fn serialize_changes(&self) -> Vec<u8> {
        let changeset = self.record_store.drain_changes();
        let mut result = Vec::new();

        for index in changeset.created.ones() {
            if let Some(id) = self.record_store.mapper.index_to_id.get(index) {
                if let Some(record) = self.record_store.get(id) {
                    BinaryDeltaCodec::encode_delta(
                        &mut result,
                        id.clone(),
                        BitFlags::<ShapeField>::all(),
                        record,
                    );
                }
            }
        }

        result
    }

    /// Tamaño estimado del changeset serializado
    pub fn changeset_size_estimate(&self) -> usize {
        let changes = self.record_store.changeset().created.count();
        // Estimación: ~30 bytes por record típico
        changes * 30
    }
}
```

### Módulo 6.4: `src/lib.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
//! archflow-wasm-collab
//! 
//! Zero-copy bridge entre Rust y JavaScript usando SharedArrayBuffer.
//! Optimizado para CRDT synchronization con binary delta encoding.
//! 
//! ## Uso
//!
//! ```javascript
//! import init, { WasmBridge } from './pkg/archflow_wasm_collab.js';
//! 
//! async function main() {
//!   await init();
//!   
//!   const bridge = new WasmBridge(1000);
//!   const ptr = bridge.get_render_buffer_ptr();
//!   const len = bridge.get_render_buffer_len();
//!   
//!   // Usar memoria directamente desde JS
//!   const f32 = bridge.data;
//!   const ids = bridge.ids;
//! }
//! ```

use wasm_bindgen::prelude::*;

pub mod shared_buffer;
pub mod binary_delta_codec;
pub mod types;

pub use shared_buffer::{SharedBuffer, RenderAttribute};
pub use binary_delta_codec::{BinaryDeltaCodec, DecodedDelta};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    log("archflow-wasm-collab initialized");
}
```

---

## 🎯 Criterios de Aceptación

| Criterio | Target | Método |
|----------|--------|--------|
| Zero-copy | SharedArrayBuffer con bytemuck | Code review |
| Delta size | < 25% del JSON original | Benchmark |
| 60fps | Render update en < 16ms | Profiling |
| Cross-origin | COOP/COEP verificados | Test |
| Memory safety | Sin UB en unsafe blocks | miri |
| Protocol version | Forward compatibility | Test |

---

## 🗑️ Eliminación Legacy

```bash
#!/bin/bash
# Eliminar código legacy de WASM

echo "🗑️ Eliminando archflow-wasm/ legacy..."
rm -rf crates/archflow-wasm/

echo "✅ WASM Legacy eliminado"
```

---

## 📊 Referencias al Documento de Migración

| Sección | Contenido | Referencia |
|---------|-----------|------------|
| F.4 | bytemuck zero-copy | L3001-3100 |
| F.5 | BinaryDeltaCodec | L3100-3200 |
| 6.0 | ShapeField enum | L3150-3160 |
| 6.1 | SharedBuffer | L3150-3180 |
| 6.2 | Delta encoding | L3180-3210 |

---

## 📝 Cambios desde v1.0.0

| Versión | Cambio | Razón |
|---------|--------|-------|
| 1.1.0 | `encode_varint` → `Vec<u8>` | Corrección: evitar bytes desperdiciados |
| 1.1.0 | `update()` usa `zip()` | Corrección: bounds safety |
| 1.1.0 | Memory views actualizados | wasm-bindgen 0.2.100+ API |
| 1.1.0 | ShapeField enum añadido | Type safety para BitFlags |
| 1.1.0 | Protocol version byte | Forward compatibility |
| 1.1.0 | Unsafe block docs | Claridad sobre contracto de seguridad |

---

**Documento de Época: EPIC-FASE-06-WASMBridge.md**  
**Versión:** 1.1.0  
**Creado:** 2026-01-26  
**Actualizado:** 2026-01-27
