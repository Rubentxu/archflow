//! Styles - Sistema de estilos para primitivas
//!
//! Define los estilos de relleno y trazo para primitivas

use archflow_core::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Estilo de relleno
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub color: Color,
    pub opacity: f32,
    pub pattern: Option<FillPattern>,
}

impl FillStyle {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            opacity: 1.0,
            pattern: None,
        }
    }
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for FillStyle {
    fn default() -> Self {
        Self::new(Color::BLACK)
    }
}

/// Patrones de relleno
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillPattern {
    Solid,
    HorizontalLines,
    VerticalLines,
    DiagonalLines,
    Crosshatch,
    Dots,
    Checkerboard,
}

/// Estilo de trazo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
    pub line_type: LineType,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub opacity: f32,
}

impl StrokeStyle {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            line_type: LineType::Solid,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            opacity: 1.0,
        }
    }
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::new(Color::BLACK, 1.0)
    }
}

/// Tipos de línea
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    Solid,
    Dashed(Vec<f32>), // [dash, gap, dash, gap, ...]
    Dotted(Vec<f32>),
}

/// Terminadores de línea
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Uniones entre segmentos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Estilo de texto
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16, // 100-900
    pub color: Color,
    pub align_x: TextAlign,
    pub align_y: TextAlignY,
}

impl TextStyle {
    pub fn new(font_family: &str, font_size: f32, color: Color) -> Self {
        Self {
            font_family: font_family.to_string(),
            font_size,
            font_weight: 400,
            color,
            align_x: TextAlign::Left,
            align_y: TextAlignY::Top,
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new("sans-serif", 12.0, Color::BLACK)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignY {
    Top,
    Middle,
    Bottom,
}

/// Efectos visuales
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectStyle {
    pub shadow: Option<Shadow>,
    pub blur_radius: Option<f32>,
    pub glow_color: Option<Color>,
    pub glow_intensity: Option<f32>,
}

impl EffectStyle {
    pub fn new() -> Self {
        Self {
            shadow: None,
            blur_radius: None,
            glow_color: None,
            glow_intensity: None,
        }
    }
    pub fn with_shadow(mut self, offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        self.shadow = Some(Shadow {
            offset_x,
            offset_y,
            blur,
            color,
        });
        self
    }
    pub fn with_blur(mut self, radius: f32) -> Self {
        self.blur_radius = Some(radius);
        self
    }
    pub fn with_glow(mut self, color: Color, intensity: f32) -> Self {
        self.glow_color = Some(color);
        self.glow_intensity = Some(intensity);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub color: Color,
}

/// Estilo completo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
    pub text: Option<TextStyle>,
    pub effects: Option<EffectStyle>,
    pub custom: HashMap<String, String>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            fill: None,
            stroke: None,
            text: None,
            effects: None,
            custom: HashMap::new(),
        }
    }
    pub fn with_fill(mut self, fill: FillStyle) -> Self {
        self.fill = Some(fill);
        self
    }
    pub fn with_stroke(mut self, stroke: StrokeStyle) -> Self {
        self.stroke = Some(stroke);
        self
    }
    pub fn with_text(mut self, text: TextStyle) -> Self {
        self.text = Some(text);
        self
    }
    pub fn with_effects(mut self, effects: EffectStyle) -> Self {
        self.effects = Some(effects);
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

/// Combinación Fill + Stroke
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeStyle {
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
}

impl ShapeStyle {
    pub fn filled(fill: FillStyle) -> Self {
        Self {
            fill: Some(fill),
            stroke: None,
        }
    }
    pub fn stroked(stroke: StrokeStyle) -> Self {
        Self {
            fill: None,
            stroke: Some(stroke),
        }
    }
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self::filled(FillStyle::default())
    }
}
