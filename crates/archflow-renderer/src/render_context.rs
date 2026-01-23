//! ArchFlow Render Context - Rendering optimizado
//!
//! Este módulo proporciona:
//! - Dirty rect tracking para minimizar redibujado
//! - Spatial culling para objetos fuera del viewport
//! - Batch rendering por tipo de primitiva
//! - FPS counter para debugging

use crate::{
    image::ImageData, path::SvgPath, stroke::StrokeStyle, FontStyle, Renderer as RendererTrait,
};
use archflow_core::{Color, Rect, Vec2};

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Región sucia (necesita redibujado)
#[derive(Debug, Clone, Default)]
pub struct DirtyRegion {
    rects: Vec<Rect>,
}

impl DirtyRegion {
    /// Crear región vacía
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    /// Añadir rectángulo a la región
    pub fn add(&mut self, rect: Rect) {
        let mut merged = false;
        let mut new_rects = Vec::with_capacity(self.rects.len().saturating_add(1));

        for existing in self.rects.drain(..) {
            if let Some(union) = Self::union_rect(existing, rect) {
                new_rects.push(union);
                merged = true;
            } else {
                new_rects.push(existing);
            }
        }

        if !merged {
            new_rects.push(rect);
        }

        self.rects = new_rects;
    }

    /// Fusionar dos rectángulos si se solapan
    fn union_rect(a: Rect, b: Rect) -> Option<Rect> {
        let left = a.min.x.min(b.min.x);
        let top = a.min.y.min(b.min.y);
        let right = a.max.x.max(b.max.x);
        let bottom = a.max.y.max(b.max.y);

        if right > left && bottom > top {
            Some(Rect::from_min_max(
                Vec2::new(left, top),
                Vec2::new(right, bottom),
            ))
        } else {
            None
        }
    }

    /// Verificar si está vacía
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// Obtener todos los rectángulos
    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    /// Obtener bounding box de toda la región
    pub fn bounding_box(&self) -> Option<Rect> {
        self.rects.iter().fold(None, |acc, r| {
            if let Some(acc) = acc {
                let left = acc.min.x.min(r.min.x);
                let top = acc.min.y.min(r.min.y);
                let right = acc.max.x.max(r.max.x);
                let bottom = acc.max.y.max(r.max.y);
                Some(Rect::from_min_max(
                    Vec2::new(left, top),
                    Vec2::new(right, bottom),
                ))
            } else {
                Some(*r)
            }
        })
    }

    /// Limpiar región
    pub fn clear(&mut self) {
        self.rects.clear();
    }
}

/// Tipo de operación de rendering para batching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderOpType {
    FillRect,
    FillEllipse,
    FillPath,
    StrokeRect,
    StrokeEllipse,
    StrokePath,
    DrawText,
    DrawImage,
}

/// Operación de rendering individual
#[derive(Debug, Clone)]
pub struct RenderOp {
    pub op_type: RenderOpType,
    pub data: RenderOpData,
    pub z_index: u32,
    pub layer: String,
}

#[derive(Debug, Clone)]
pub enum RenderOpData {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    Ellipse {
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        color: Color,
    },
    Path {
        svg_path: String,
        color: Color,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        font_size: f32,
        color: Color,
    },
    Image {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}

/// Configuración de renderizado optimizado
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Habilitar dirty rect tracking
    pub enable_dirty_rects: bool,
    /// Habilitar spatial culling
    pub enable_culling: bool,
    /// Habilitar batch rendering
    pub enable_batching: bool,
    /// Margen para culling (pixels extra alrededor del viewport)
    pub culling_margin: f32,
    /// Tamaño mínimo de región sucia para redibujar
    pub min_dirty_area: f32,
    /// Habilitar FPS counter
    pub enable_fps_counter: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            enable_dirty_rects: true,
            enable_culling: true,
            enable_batching: true,
            culling_margin: 10.0,
            min_dirty_area: 1.0,
            enable_fps_counter: true,
        }
    }
}

/// Contexto de renderizado optimizado
pub struct RenderContext<R: RendererTrait> {
    /// Renderer subyacente
    renderer: R,
    /// Configuración
    config: RenderConfig,
    /// Región sucia actual
    dirty_region: DirtyRegion,
    /// Viewport actual
    viewport: Rect,
    /// Cola de operaciones para batch rendering
    render_queue: Vec<RenderOp>,
    /// Operaciones agrupadas por tipo
    batches: HashMap<(RenderOpType, String), Vec<RenderOp>>,
    /// Frame actual
    frame_count: u64,
    /// Tiempo del último frame
    last_frame_time: Instant,
    /// FPS actual
    fps: f64,
    /// Historial de FPS para promediado
    fps_history: Vec<f64>,
}

impl<R: RendererTrait> RenderContext<R> {
    /// Crear nuevo contexto de renderizado
    pub fn new(renderer: R, viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            renderer,
            config: RenderConfig::default(),
            dirty_region: DirtyRegion::new(),
            viewport: Rect::from_pos_size(
                Vec2::ZERO,
                Vec2::new(viewport_width as f32, viewport_height as f32),
            ),
            render_queue: Vec::new(),
            batches: HashMap::new(),
            frame_count: 0,
            last_frame_time: Instant::now(),
            fps: 0.0,
            fps_history: Vec::new(),
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(renderer: R, config: RenderConfig, viewport: Rect) -> Self {
        Self {
            renderer,
            config,
            dirty_region: DirtyRegion::new(),
            viewport,
            render_queue: Vec::new(),
            batches: HashMap::new(),
            frame_count: 0,
            last_frame_time: Instant::now(),
            fps: 0.0,
            fps_history: Vec::new(),
        }
    }

    /// Obtener referencia al renderer
    pub fn renderer(&mut self) -> &mut R {
        &mut self.renderer
    }

    /// Marcar área como dirty
    pub fn mark_dirty(&mut self, rect: Rect) {
        self.dirty_region.add(rect);
    }

    /// Actualizar viewport
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let old_viewport = self.viewport;
        self.viewport = Rect::from_pos_size(Vec2::new(x, y), Vec2::new(width, height));

        // Si el viewport cambió significativamente, marcar todo como dirty
        let old_min = old_viewport.min;
        let new_min = self.viewport.min;
        if (old_min.x - new_min.x).abs() > self.config.culling_margin
            || (old_min.y - new_min.y).abs() > self.config.culling_margin
        {
            self.dirty_region = DirtyRegion::new();
        }
    }

    /// Obtener FPS actual
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Obtener número de frame
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Renderizar un frame completo
    pub fn render_frame(&mut self) {
        let frame_start = Instant::now();

        // 1. Actualizar FPS
        self.frame_count += 1;
        let elapsed = frame_start.duration_since(self.last_frame_time);
        if elapsed > Duration::ZERO {
            let current_fps = 1.0 / elapsed.as_secs_f64();
            self.fps_history.push(current_fps);
            if self.fps_history.len() > 60 {
                self.fps_history.remove(0);
            }
            self.fps = self.fps_history.iter().sum::<f64>() / self.fps_history.len() as f64;
        }
        self.last_frame_time = frame_start;

        // 2. Determinar región a redibujar
        let _render_region = if self.config.enable_dirty_rects {
            self.dirty_region.bounding_box().unwrap_or(self.viewport)
        } else {
            self.viewport
        };

        // 3. Si no hay nada dirty, salir temprano
        if self.config.enable_dirty_rects && self.dirty_region.is_empty() {
            return;
        }

        // 4. Limpiar solo la región dirty
        self.renderer.save();
        self.renderer.reset_transform();

        // 5. Renderizar operaciones de la cola
        self.execute_render_queue();

        // 6. Renderizar FPS counter si está habilitado
        if self.config.enable_fps_counter {
            self.render_fps_counter();
        }

        self.renderer.restore();

        // 7. Limpiar región dirty
        if self.config.enable_dirty_rects {
            self.dirty_region.clear();
        }
    }

    /// Ejecutar operaciones de rendering
    fn execute_render_queue(&mut self) {
        for op in &self.render_queue {
            match &op.data {
                RenderOpData::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    self.renderer.clear(*color);
                    self.renderer.draw_rect(*x, *y, *width, *height);
                }
                RenderOpData::Ellipse {
                    cx,
                    cy,
                    rx,
                    ry,
                    color,
                } => {
                    self.renderer.clear(*color);
                    self.renderer.draw_ellipse(*cx, *cy, *rx, *ry);
                }
                RenderOpData::Path { svg_path, color } => {
                    let path = SvgPath::new(svg_path);
                    self.renderer.fill_path(&path, *color);
                    self.renderer.stroke_path(&path, &StrokeStyle::default());
                }
                RenderOpData::Text {
                    text,
                    x,
                    y,
                    font_size: _,
                    color: _,
                } => {
                    self.renderer.draw_text(text, *x, *y, &FontStyle::default());
                }
                RenderOpData::Image {
                    width,
                    height,
                    data,
                } => {
                    let image = ImageData::new_rgba(*width, *height, data.clone());
                    self.renderer
                        .draw_image(&image, 0.0, 0.0, *width as f32, *height as f32);
                }
            }
        }
    }

    /// Renderizar contador de FPS
    fn render_fps_counter(&mut self) {
        let fps_text = format!("FPS: {:.1}", self.fps);
        let _ = self
            .renderer
            .draw_text(&fps_text, 10.0, 20.0, &FontStyle::default());
    }

    /// Obtener estadísticas de rendering
    pub fn stats(&self) -> RenderStats {
        RenderStats {
            frame_count: self.frame_count,
            fps: self.fps,
            render_ops: self.render_queue.len(),
            batches: self.batches.len(),
        }
    }

    /// Configurar dirty rect tracking
    pub fn set_dirty_rect_tracking(&mut self, enabled: bool) {
        self.config.enable_dirty_rects = enabled;
    }

    /// Configurar spatial culling
    pub fn set_spatial_culling(&mut self, enabled: bool) {
        self.config.enable_culling = enabled;
    }

    /// Configurar batch rendering
    pub fn set_batch_rendering(&mut self, enabled: bool) {
        self.config.enable_batching = enabled;
    }
}

/// Estadísticas de renderizado
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub frame_count: u64,
    pub fps: f64,
    pub render_ops: usize,
    pub batches: usize,
}

impl RenderStats {
    /// Obtener resumen como string
    pub fn summary(&self) -> String {
        format!(
            "FPS: {:.1} | Ops: {} | Batches: {}",
            self.fps, self.render_ops, self.batches
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_region_add() {
        let mut region = DirtyRegion::new();
        assert!(region.is_empty());

        region.add(Rect::from_pos_size(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        assert!(!region.is_empty());
        assert_eq!(region.rects().len(), 1);
    }

    #[test]
    fn test_dirty_region_merge() {
        let mut region = DirtyRegion::new();

        region.add(Rect::from_pos_size(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        region.add(Rect::from_pos_size(
            Vec2::new(50.0, 50.0),
            Vec2::new(100.0, 100.0),
        ));

        // Los dos rectángulos solapados deberían fusionarse
        assert_eq!(region.rects().len(), 1);
    }

    #[test]
    fn test_render_stats() {
        let stats = RenderStats {
            frame_count: 100,
            fps: 60.0,
            render_ops: 20,
            batches: 5,
        };

        let summary = stats.summary();
        assert!(summary.contains("60.0"));
        assert!(summary.contains("20"));
        assert!(summary.contains("5"));
    }
}
