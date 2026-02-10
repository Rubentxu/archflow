// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Math Module - Core geometric types
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 3.2
//
// Re-exports from glam with ArchFlow-specific extensions:
// - Vec2: 2D vector for positions, sizes, deltas
// - Rect: Axis-aligned bounding rectangle
// - Color: Packed RGBA color (0xRRGGBBAA)
// - Transform: 2D affine transform (position + rotation + scale)
// ═══════════════════════════════════════════════════════════════════════════════════════════════

// Import String and format macro when std feature is enabled
#[cfg(feature = "std")]
use std::{format, string::String};

/// 2D vector for positions, sizes, and deltas
///
/// Re-exported from glam::Vec2 with no_std support
pub type Vec2 = glam::Vec2;

/// 4D vector for homogeneous coordinates and colors
pub type Vec4 = glam::Vec4;

/// 2D affine transform matrix
pub type Mat4 = glam::Mat4;

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// RECT - Axis-Aligned Bounding Rectangle
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Axis-aligned bounding rectangle
///
/// Layout: [min_x, min_y, max_x, max_y] in world coordinates
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    /// Minimum corner (bottom-left in Y-up coordinate system)
    pub min: Vec2,
    /// Maximum corner (top-right in Y-up coordinate system)
    pub max: Vec2,
}

impl Rect {
    /// Create a rectangle from min and max corners
    #[inline(always)]
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    /// Create a rectangle from center and size
    #[inline(always)]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size / 2.0;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Create a rectangle from origin (min corner) and size
    #[inline(always)]
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        Self {
            min: origin,
            max: origin + size,
        }
    }

    /// Get the width of the rectangle
    #[inline(always)]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Get the height of the rectangle
    #[inline(always)]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Get the size (width, height) of the rectangle
    #[inline(always)]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    /// Get the center point of the rectangle
    #[inline(always)]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Check if a point is contained within the rectangle
    #[inline(always)]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if this rectangle fully contains another
    #[inline(always)]
    pub fn contains_rect(&self, other: &Rect) -> bool {
        other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
    }

    /// Check if this rectangle intersects another
    #[inline(always)]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Get the intersection of two rectangles
    #[inline(always)]
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        Some(Rect {
            min: Vec2::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            max: Vec2::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        })
    }

    /// Expand the rectangle by a margin on all sides
    #[inline(always)]
    pub fn inflate(&self, margin: f32) -> Rect {
        Rect {
            min: self.min - Vec2::splat(margin),
            max: self.max + Vec2::splat(margin),
        }
    }

    /// Get the area of the rectangle
    #[inline(always)]
    pub fn area(&self) -> f32 {
        self.width() * self.height()
    }

    /// Check if the rectangle is empty (width or height <= 0)
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.width() <= 0.0 || self.height() <= 0.0
    }

    /// Get the union of two rectangles (smallest rect containing both)
    #[inline(always)]
    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            min: Vec2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Vec2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// Find the closest point on the rectangle to a given point
    ///
    /// If the point is inside the rectangle, returns the point itself.
    /// Otherwise returns the closest point on the rectangle's boundary.
    #[inline(always)]
    pub fn closest_point(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// COLOR - Packed RGBA (0xRRGGBBAA)
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Packed RGBA color in 0xAABBGGRR format (ABGR)
///
/// This format ensures that when written as Little Endian bytes,
/// it produces [RR, GG, BB, AA] in memory, which WebGL reads as
/// vec4(r, g, b, a) when normalized=true.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Color(pub u32);

impl Color {
    /// Black color (R=0, G=0, B=0, A=255) -> 0xFF000000
    pub const BLACK: Color = Color(0xFF000000);

    /// White color (R=255, G=255, B=255, A=255) -> 0xFFFFFFFF
    pub const WHITE: Color = Color(0xFFFFFFFF);

    /// Transparent color (0x00000000)
    pub const TRANSPARENT: Color = Color(0x00000000);

    /// Red color (R=255, G=0, B=0, A=255) -> 0xFF0000FF
    pub const RED: Color = Color(0xFF0000FF);

    /// Green color (R=0, G=255, B=0, A=255) -> 0xFF00FF00
    pub const GREEN: Color = Color(0xFF00FF00);

    /// Blue color (R=0, G=0, B=255, A=255) -> 0xFFFF0000
    pub const BLUE: Color = Color(0xFFFF0000);

    /// Create a color from RGBA components (0-255)
    #[inline(always)]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32))
    }

    /// Create a color from RGB components with full opacity
    #[inline(always)]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Create a color from normalized RGBA components (0.0-1.0)
    #[inline(always)]
    pub fn rgba_normalized(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::rgba(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
            (a.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    /// Extract the red component (0-255)
    #[inline(always)]
    pub const fn r(self) -> u8 {
        self.0 as u8
    }

    /// Extract the green component (0-255)
    #[inline(always)]
    pub const fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// Extract the blue component (0-255)
    #[inline(always)]
    pub const fn b(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Extract the alpha component (0-255)
    #[inline(always)]
    pub const fn a(self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// Get the raw u32 value
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Convert to RGBA array [f32; 4] with normalized values (0.0-1.0)
    #[inline(always)]
    pub fn to_array_normalized(self) -> [f32; 4] {
        [
            self.r() as f32 / 255.0,
            self.g() as f32 / 255.0,
            self.b() as f32 / 255.0,
            self.a() as f32 / 255.0,
        ]
    }

    /// Multiply RGB components by a factor (for tinting)
    #[inline(always)]
    pub fn multiply_rgb(self, factor: f32) -> Self {
        Self::rgba_normalized(
            self.r() as f32 / 255.0 * factor,
            self.g() as f32 / 255.0 * factor,
            self.b() as f32 / 255.0 * factor,
            self.a() as f32 / 255.0,
        )
    }

    /// Set the alpha component
    #[inline(always)]
    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self::rgba(self.r(), self.g(), self.b(), alpha)
    }
}

#[cfg(feature = "std")]
impl Color {
    /// Create color from hex string (e.g., "#FF0000" or "#FF0000FF")
    #[inline(always)]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };

        Some(Self::rgba(r, g, b, a))
    }

    /// Convert to hex string (e.g., "#FF0000FF")
    #[inline(always)]
    pub fn to_hex(self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.r(),
            self.g(),
            self.b(),
            self.a()
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// TRANSFORM - 2D Affine Transform
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// 2D affine transform for entity positioning
///
/// For most entities, we only need position and size (stored as [x, y, w, h])
/// This is used for more complex transforms (rotation, skew, etc.)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Transform {
    /// Translation component
    pub translation: Vec2,
    /// Rotation in radians (counter-clockwise)
    pub rotation: f32,
    /// Scale factor (uniform for now, could be Vec2 for non-uniform)
    pub scale: f32,
}

impl Transform {
    /// Create a new transform
    #[inline(always)]
    pub fn new(translation: Vec2, rotation: f32, scale: f32) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Create an identity transform (no transformation)
    #[inline(always)]
    pub fn identity() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: 1.0,
        }
    }

    /// Create a translation-only transform
    #[inline(always)]
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            translation: Vec2::new(x, y),
            rotation: 0.0,
            scale: 1.0,
        }
    }

    /// Apply this transform to a point
    #[inline(always)]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        // Scale → Rotate → Translate
        let scaled = point * self.scale;
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotated = Vec2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        );
        rotated + self.translation
    }

    /// Compose two transforms (this * other)
    #[inline(always)]
    pub fn compose(&self, other: &Transform) -> Transform {
        Transform {
            translation: self.transform_point(other.translation),
            rotation: self.rotation + other.rotation,
            scale: self.scale * other.scale,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// CONNECTION STYLE - Connection routing styles for diagrams
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/// Connection style types for arrow/line rendering.
///
/// Defines how connections (arrows, lines) are routed between entities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum ConnectionStyle {
    /// Straight line between points (direct connection)
    Straight = 0,
    /// Orthogonal with 90° turns only (L/Z shaped)
    Orthogonal = 1,
    /// Smooth cubic Bezier curve
    Bezier = 2,
    /// Elbow routing with corner optimization
    Elbow = 3,
}

#[allow(clippy::derivable_impls)]
impl Default for ConnectionStyle {
    fn default() -> Self {
        ConnectionStyle::Straight
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_from_center_size() {
        let rect = Rect::from_center_size(Vec2::new(10.0, 10.0), Vec2::new(10.0, 6.0));
        assert_eq!(rect.min, Vec2::new(5.0, 7.0));
        assert_eq!(rect.max, Vec2::new(15.0, 13.0));
        assert_eq!(rect.width(), 10.0);
        assert_eq!(rect.height(), 6.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains(Vec2::new(5.0, 5.0)));
        assert!(rect.contains(Vec2::new(0.0, 0.0)));
        assert!(rect.contains(Vec2::new(10.0, 10.0)));
        assert!(!rect.contains(Vec2::new(11.0, 5.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 15.0, 15.0);
        let c = Rect::new(20.0, 20.0, 30.0, 30.0);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_color_packing() {
        let color = Color::rgba(255, 128, 64, 192);
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 128);
        assert_eq!(color.b(), 64);
        assert_eq!(color.a(), 192);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF8000").unwrap();
        assert_eq!(color, Color::rgba(255, 128, 0, 255));
        let with_alpha = Color::from_hex("#FF8000C0").unwrap();
        assert_eq!(with_alpha, Color::rgba(255, 128, 0, 192));
    }

    #[test]
    fn test_transform_identity() {
        let identity = Transform::identity();
        let point = Vec2::new(5.0, 3.0);
        assert_eq!(identity.transform_point(point), point);
    }

    #[test]
    fn test_transform_translation() {
        let transform = Transform::translation(10.0, 5.0);
        let point = Vec2::new(2.0, 3.0);
        assert_eq!(transform.transform_point(point), Vec2::new(12.0, 8.0));
    }

    #[test]
    fn test_connection_style_default() {
        assert_eq!(ConnectionStyle::default(), ConnectionStyle::Straight);
    }

    #[test]
    fn test_connection_style_values() {
        assert_eq!(ConnectionStyle::Straight as u8, 0);
        assert_eq!(ConnectionStyle::Orthogonal as u8, 1);
        assert_eq!(ConnectionStyle::Bezier as u8, 2);
        assert_eq!(ConnectionStyle::Elbow as u8, 3);
    }
}
