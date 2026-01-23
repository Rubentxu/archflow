//! ArchFlow Renderer - Traits de rendering
//!
//! Este crate define los traits abstractos para el sistema de rendering.
//! Los paths usan un trait Path abstracto para permitir diferentes implementaciones.

pub mod render_context;
pub mod selection_renderer;

pub use render_context::{
    DirtyRegion, RenderConfig, RenderContext, RenderOp, RenderOpData, RenderOpType, RenderStats,
};

pub use selection_renderer::SelectionRenderer;

use archflow_core::Color;

/// Trait principal para renderers
pub trait Renderer {
    /// Limpiar el canvas con un color
    fn clear(&mut self, color: Color);

    /// Guardar estado del renderer
    fn save(&mut self);

    /// Restaurar estado del renderer
    fn restore(&mut self);

    /// Aplicar transformación de traslación
    fn translate(&mut self, x: f32, y: f32);

    /// Aplicar transformación de rotación
    fn rotate(&mut self, angle: f32);

    /// Aplicar transformación de escala
    fn scale(&mut self, sx: f32, sy: f32);

    /// Resetear transformaciones a identidad
    fn reset_transform(&mut self);

    // Métodos de dibujo primitivos
    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32);
    fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32);

    // Métodos de dibujo con paths
    fn draw_path(&mut self, path: &dyn Path);
    fn fill_path(&mut self, path: &dyn Path, color: Color);
    fn stroke_path(&mut self, path: &dyn Path, style: &StrokeStyle);

    // Métodos de texto
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font: &FontStyle);

    // Métodos de imagen
    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32);
    fn draw_image_slice(
        &mut self,
        image: &dyn Image,
        src_x: f32,
        src_y: f32,
        src_width: f32,
        src_height: f32,
        dst_x: f32,
        dst_y: f32,
        dst_width: f32,
        dst_height: f32,
    );
}

/// Trait abstracto para paths
pub trait Path {
    /// Convertir a SVG path string
    fn to_svg_path(&self) -> String;

    /// Obtener bounding box (x, y, width, height)
    fn bounds(&self) -> (f32, f32, f32, f32);

    /// Verificar si está vacío
    fn is_empty(&self) -> bool;

    /// Obtener longitud aproximada
    fn length(&self) -> f64;
}

/// Trait para imágenes
pub trait Image {
    /// Ancho de la imagen en pixels
    fn width(&self) -> u32;

    /// Alto de la imagen en pixels
    fn height(&self) -> u32;

    /// Datos de píxeles (RGBA)
    fn data(&self) -> &[u8];

    /// Formato de píxeles
    fn pixel_format(&self) -> PixelFormat;
}

/// Formato de píxeles de imagen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8,
    Rgb8,
    Gray8,
}

/// Estilo de texto
#[derive(Debug, Clone)]
pub struct FontStyle {
    /// Familia de fuente
    pub family: FontFamily,
    /// Tamaño en pixels
    pub size: f32,
    /// Peso de la fuente
    pub weight: FontWeight,
    /// Estilo adicional
    pub style: FontStyleType,
    /// Color del texto
    pub color: Color,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            family: FontFamily::SansSerif,
            size: 16.0,
            weight: FontWeight::Normal,
            style: FontStyleType::Normal,
            color: Color::BLACK,
        }
    }
}

/// Familia de fuente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    SansSerif,
    Serif,
    Monospace,
    Cursive,
    Fantasy,
    SystemUi,
}

/// Peso de fuente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

/// Estilo de fuente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyleType {
    Normal,
    Italic,
    Oblique,
}

/// Estilo de stroke para paths
#[derive(Debug, Clone)]
pub struct StrokeStyle {
    /// Color de la línea
    pub color: Color,
    /// Ancho de línea en pixels
    pub width: f32,
    /// Estilo de terminación de línea
    pub line_cap: LineCap,
    /// Estilo de unión entre segmentos
    pub line_join: LineJoin,
    /// Patrón de dash (vacío = línea continua)
    pub dash_pattern: Option<Vec<f32>>,
    /// Límite de miter (para LineJoin::Miter)
    pub miter_limit: f32,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        }
    }
}

/// Estilo de terminación de línea
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Estilo de unión entre segmentos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Modo de composición
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeOperation {
    SourceOver,
    SourceAtop,
    SourceIn,
    SourceOut,
    DestinationOver,
    DestinationAtop,
    DestinationIn,
    DestinationOut,
    Lighter,
    Copy,
    Xor,
}

/// Configuración global del renderer
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Ancho del viewport
    pub viewport_width: u32,
    /// Alto del viewport
    pub viewport_height: u32,
    /// Ratio de píxeles (device pixel ratio)
    pub pixel_ratio: f32,
    /// Color de fondo por defecto
    pub background_color: Color,
    /// Operación de composición por defecto
    pub composite_operation: CompositeOperation,
    /// Habilitar antialiasing
    pub antialias: bool,
    /// Resolución de curvas (subdivisiones)
    pub curve_resolution: f32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            viewport_width: 800,
            viewport_height: 600,
            pixel_ratio: 1.0,
            background_color: Color::WHITE,
            composite_operation: CompositeOperation::SourceOver,
            antialias: true,
            curve_resolution: 1.0,
        }
    }
}
