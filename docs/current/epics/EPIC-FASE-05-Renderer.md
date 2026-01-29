# EPIC-FASE-05: Renderer

**Versión:** 1.0.0  
**Fase:** 5/8  
**Duración:** Semana 7  
**Referencia:** `MIGRACION_RECORDS_V2_COMPLETA.md` - L2793-3001, F.9

---

## 📋 Descripción General

**ENFOQUE: CERO CÓDIGO LEGACY - TODO DESDE CERO**

Sistema de rendering con batch rendering para WebGPU con máximo rendimiento.

### Archivos Legacy a ELIMINAR:
```
crates/archflow-renderer/src/lib.rs      → ELIMINAR
crates/archflow-renderer/src/path.rs     → ELIMINAR
crates/archflow-renderer-canvas/src/lib.rs → ELIMINAR
crates/archflow-renderer-rough/src/lib.rs → ELIMINAR
```

### Objetivos Principales
- Crear `archflow-renderers/` crate **desde cero**
- Implementar `Renderable` trait optimizado
- Implementar `BatchRenderer2D` (Apéndice F.9)
- Instancing para WebGPU con zero-copy
- 60fps garantizado con 10k+ elementos

---

## 🔬 Investigación Perplexity Requerida

Antes de implementar, realizar investigación con Perplexity sobre:
- WebGPU instanced rendering best practices 2024
- bytemuck + wgpu integration patterns
- Batch rendering GPU optimization techniques
- Vertex buffer streaming patterns
- Shader specialization for 2D rendering

**Prompt de investigación:**
```
Research WebGPU instanced rendering patterns for 2D batch rendering 2024.
Focus on: 1) bytemuck pod transformation for vertex buffers, 
2) wgpu render pass instancing, 3) dynamic vertex buffer updates,
4) GPU memory optimization for 10k+ sprites. Include code examples.
```

---

## 📦 Entregables (TODO DESDE CERO)

### Módulo 5.1: `src/traits/renderable.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod renderable_tests {
    use super::*;

    #[test]
    fn test_renderable_bounds() {
        let renderable = TestRenderable::new(Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        let bounds = renderable.bounds();
        assert_eq!(bounds.min(), Vec2::ZERO);
        assert_eq!(bounds.max(), Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_renderable_contains_point() {
        let renderable = TestRenderable::new(Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        assert!(renderable.contains_point(Vec2::new(50.0, 50.0)));
        assert!(!renderable.contains_point(Vec2::new(150.0, 150.0)));
    }

    #[test]
    fn test_renderable_priority_ordering() {
        let low = TestRenderable::with_priority(0);
        let high = TestRenderable::with_priority(100);
        assert!(low.render_priority() < high.render_priority());
    }
}
```

**Implementación:**
```rust
// CÓDIGO NUEVO - SIN LEGACY
use glam::Vec2;

/// Trait para objetos renderizables
pub trait Renderable: Send + Sync {
    fn bounds(&self) -> Option<Bounds>;
    fn contains_point(&self, point: Vec2) -> bool;
    fn render_priority(&self) -> i32;
    fn material_id(&self) -> u64;
    fn color(&self) -> RgbaColor;
}

/// Bounds 2D
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

/// Color RGBA
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }
}
```

### Módulo 5.2: `src/batch_renderer.rs` (NUEVO)

**TDD Test First:**
```rust
#[cfg(test)]
mod batch_renderer_tests {
    use super::*;

    #[test]
    fn test_instance_raw_pod() {
        let instance = InstanceRaw {
            model_matrix: glam::Mat4::IDENTITY.to_cols_array_2d(),
            color: [1.0, 0.0, 0.0, 1.0],
        };
        // Verificar que es Pod
        let bytes = bytemuck::bytes_of(&instance);
        assert_eq!(bytes.len(), std::mem::size_of::<InstanceRaw>());
    }

    #[test]
    fn test_batch_clear() {
        let mut renderer = BatchRenderer2D::new(1000);
        renderer.batches.insert(1, vec![]);

        renderer.clear();
        assert!(renderer.batches.is_empty());
    }

    #[test]
    fn test_prepare_frame_empty() {
        let mut renderer = BatchRenderer2D::new(100);
        renderer.prepare_frame(&[], &TestRecordStore::new());
        assert!(renderer.batches.is_empty());
    }
}
```

**Implementación:**
```rust
// F.9: Batch rendering con WebGPU y bytemuck zero-copy
use bytemuck::{Pod, Zeroable, bytes_of};
use glam::{Mat4, Vec4};
use std::collections::HashMap;

/// Instancia cruda para GPU (must be POD for bytemuck)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct InstanceRaw {
    /// Matriz de modelo 4x4 (column-major para GPU)
    pub model_matrix: [[f32; 4]; 4],
    /// Color en RGBA float
    pub color: [f32; 4],
}

/// Lotes de instancing por material
pub struct BatchRenderer2D {
    batches: HashMap<u64, Vec<InstanceRaw>>,
    max_instances: usize,
}

impl BatchRenderer2D {
    pub fn new(max_instances: usize) -> Self {
        Self {
            batches: HashMap::new(),
            max_instances,
        }
    }

    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// F.9: Preparar frame con solo elementos visibles
    pub fn prepare_frame(
        &mut self,
        visible_ids: &[RecordId],
        store: &RecordStore<dyn Record>,
    ) {
        self.clear();

        for (batch_idx, id) in visible_ids.iter().enumerate().take(self.max_instances) {
            if let Some(record) = store.get(id) {
                let instance = InstanceRaw {
                    model_matrix: Self::compute_model_matrix(record),
                    color: record.color().to_f32_array(),
                };
                self.batches
                    .entry(record.material_id())
                    .or_default()
                    .push(instance);
            }
        }
    }

    fn compute_model_matrix(record: &dyn Record) -> [[f32; 4]; 4] {
        let bounds = match record.bounds() {
            Some(b) => b,
            return Mat4::IDENTITY.to_cols_array_2d(),
        };

        let center = bounds.center();
        let size = Vec2::new(bounds.width(), bounds.height());

        Mat4::from_translation(Vec3::new(center.x, center.y, 0.0))
            .to_cols_array_2d()
    }

    /// Obtener tamaño total del buffer de instancias
    pub fn total_instance_buffer_size(&self) -> usize {
        self.batches.values()
            .map(|instances| instances.len() * std::mem::size_of::<InstanceRaw>())
            .sum()
    }

    /// Iterar batches para render pass
    pub fn iter_batches(&self) -> impl Iterator<Item = (&u64, &[InstanceRaw])> {
        self.batches.iter().map(|(k, v)| (k, v.as_slice()))
    }
}
```

### Módulo 5.3: `src/render_context.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
pub struct RenderContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl RenderContext {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("batch_renderer shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/batch.wgsl").into()),
        };

        let pipeline = Self::create_pipeline(device, surface_format, &shader);

        Self {
            device: device.clone(),
            queue: queue.clone(),
            pipeline,
            uniform_buffer: Self::create_uniform_buffer(device),
            instance_buffer: Self::create_instance_buffer(device),
        }
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        batches: &BatchRenderer2D,
    ) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("batch render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            for (material_id, instances) in batches.iter_batches() {
                if instances.is_empty() { continue; }

                // Zero-copy: pasar slice directamente a GPU
                render_pass.set_vertex_buffer(1, instances.as_bytes());
                render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
```

### Módulo 5.4: `src/lib.rs` (NUEVO)

```rust
// CÓDIGO NUEVO - SIN LEGACY
pub mod traits;
pub mod batch_renderer;
pub mod render_context;

pub use traits::{Renderable, Bounds, RgbaColor};
pub use batch_renderer::{BatchRenderer2D, InstanceRaw};
pub use render_context::RenderContext;
```

---

## 🎯 Criterios de Aceptación

| Criterio | Target | Método |
|----------|--------|--------|
| 60fps | 10k elementos a 60fps | Benchmark |
| Zero-copy | bytemuck para todos los buffers | Code review |
| Batch size | < 1ms para preparar batches | Profiling |
| Memory | < 1KB per 100 elementos | Memory profiler |

---

## 🗑️ Eliminación Legacy

```bash
#!/bin/bash
# Eliminar código legacy de Renderer

echo "🗑️ Eliminando archflow-renderer/ legacy..."
rm -rf crates/archflow-renderer/

echo "🗑️ Eliminando archflow-renderer-canvas/ legacy..."
rm -rf crates/archflow-renderer-canvas/

echo "🗑️ Eliminando archflow-renderer-rough/ legacy..."
rm -rf crates/archflow-renderer-rough/

echo "✅ Renderer Legacy eliminado"
```

---

## 📊 Referencias al Documento de Migración

| Sección | Contenido | Referencia |
|---------|-----------|------------|
| F.9 | Batch rendering | L2850-2920 |
| 5.1 | Renderable trait | L2860-2880 |
| 5.2 | InstanceRaw POD | L2880-2900 |
| 5.3 | WebGPU render | L2900-2950 |

---

**Documento de Época: EPIC-FASE-05-Renderer.md**  
**Versión:** 1.0.0  
**Creado:** 2026-01-26
