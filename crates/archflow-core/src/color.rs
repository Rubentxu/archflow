//! Color - Sistema de color RGBA

use serde::{Deserialize, Serialize};
use std::fmt;

/// Color en formato RGBA normalizado (0.0 - 1.0)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    /// Componente rojo (0.0 - 1.0)
    pub r: f32,
    /// Componente verde (0.0 - 1.0)
    pub g: f32,
    /// Componente azul (0.0 - 1.0)
    pub b: f32,
    /// Alpha/opacidad (0.0 - 1.0)
    pub a: f32,
}

impl Color {
    /// Crear color desde componentes normalizados
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Crear color RGBA
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Desde hexadecimal (ej: "#FF0000" o "FF0000")
    pub fn from_hex(hex: &str) -> Result<Self, ColorParseError> {
        let hex = hex.trim_start_matches('#');
        let hex = if hex.len() == 3 {
            // Formato corto: #RGB -> #RRGGBB
            hex.chars().flat_map(|c| vec![c, c]).collect::<String>()
        } else if hex.len() == 6 {
            hex.to_string()
        } else if hex.len() == 8 {
            hex.to_string()
        } else {
            return Err(ColorParseError::InvalidLength(hex.len()));
        };

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ColorParseError::ParseError)? as f32
            / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ColorParseError::ParseError)? as f32
            / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ColorParseError::ParseError)? as f32
            / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| ColorParseError::ParseError)? as f32
                / 255.0
        } else {
            1.0
        };

        Ok(Self { r, g, b, a })
    }

    /// Convertir a hexadecimal (sin alpha)
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }

    /// Interpolación lineal
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Con opacidad
    pub fn with_alpha(&self, a: f32) -> Self {
        Self { a, ..*self }
    }

    /// Colores predefinidos
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color(r={:.2}, g={:.2}, b={:.2}, a={:.2})",
            self.r, self.g, self.b, self.a
        )
    }
}

/// Error al parsear color
#[derive(Debug)]
pub enum ColorParseError {
    InvalidLength(usize),
    ParseError,
}

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(len) => write!(f, "Invalid hex color length: {}", len),
            Self::ParseError => write!(f, "Failed to parse hex value"),
        }
    }
}

impl std::error::Error for ColorParseError {}

/// Color en formato RGBA como bytes (0-255)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_color(&self) -> Color {
        Color::rgba(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }
}

/// Color en formato HSLA (Hue, Saturation, Lightness, Alpha)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsla {
    pub h: f32, // 0-360
    pub s: f32, // 0-1
    pub l: f32, // 0-1
    pub a: f32, // 0-1
}

impl Hsla {
    pub fn to_color(&self) -> Color {
        let c = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        let x = c * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        let m = self.l - c / 2.0;

        let (r, g, b) = if self.h < 60.0 {
            (c, x, 0.0)
        } else if self.h < 120.0 {
            (x, c, 0.0)
        } else if self.h < 180.0 {
            (0.0, c, x)
        } else if self.h < 240.0 {
            (0.0, x, c)
        } else if self.h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color::rgba(r + m, g + m, b + m, self.a)
    }
}
