//! Renderable Trait and Types
//!
//! This module provides the core trait for renderable objects
//! and supporting types for 2D batch rendering.

use glam::Vec2;

/// Trait for renderable objects in the batch rendering system.
///
/// This trait defines the interface that all renderable objects must implement
/// to be compatible with the batch renderer. Implementations should be
/// lightweight and cache-friendly.
///
/// # Safety
///
/// Implementors must ensure that bounds() returns valid data when called
/// during the render preparation phase.
pub trait Renderable: Send + Sync {
    /// Returns the bounding box of this renderable object.
    /// Returns None if the object has no spatial bounds.
    fn bounds(&self) -> Option<Bounds>;

    /// Checks if a point is contained within this renderable.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check in world coordinates
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the renderable's bounds
    fn contains_point(&self, point: Vec2) -> bool;

    /// Returns the render priority for draw order.
    ///
    /// Lower values are drawn first (behind), higher values are drawn last (in front).
    /// This allows for explicit control over draw order independently of batching.
    fn render_priority(&self) -> i32;

    /// Returns the material ID for batch grouping.
    ///
    /// Renderables with the same material_id will be batched together
    /// to minimize GPU state changes.
    fn material_id(&self) -> u64;

    /// Returns the color for this renderable.
    fn color(&self) -> RgbaColor;
}

/// 2D axis-aligned bounding box.
///
/// Represents the min and max corners of an axis-aligned rectangle.
/// All coordinates are in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    /// Minimum corner (bottom-left in typical 2D coordinate systems)
    pub min: Vec2,
    /// Maximum corner (top-right in typical 2D coordinate systems)
    pub max: Vec2,
}

impl Bounds {
    /// Creates a new Bounds from min and max corners.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum corner
    /// * `max` - The maximum corner
    ///
    /// # Panics
    ///
    /// Does not panic, but results may be invalid if min > max.
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Creates a bounds from center and size.
    ///
    /// # Arguments
    ///
    /// * `center` - The center point of the bounds
    /// * `size` - The width and height of the bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use glam::Vec2;
    /// use archflow_renderers::Bounds;
    ///
    /// let bounds = Bounds::from_center_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));
    /// assert_eq!(bounds.min, Vec2::new(75.0, 85.0));
    /// assert_eq!(bounds.max, Vec2::new(125.0, 115.0));
    /// ```
    #[inline]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size / 2.0;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    /// Returns the center point of the bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use glam::Vec2;
    /// use archflow_renderers::Bounds;
    ///
    /// let bounds = Bounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
    /// assert_eq!(bounds.center(), Vec2::new(50.0, 50.0));
    /// ```
    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Returns the width of the bounds.
    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Returns the height of the bounds.
    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Returns the size (width, height) of the bounds.
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    /// Checks if this bounds contains a point.
    ///
    /// Points on the boundary are considered inside.
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Checks if this bounds intersects with another.
    ///
    /// Returns true if the two bounds have any overlap.
    #[inline]
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Creates an invalid/empty bounds.
    ///
    /// Useful as a sentinel value or default.
    #[inline]
    pub fn invalid() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        }
    }

    /// Checks if this bounds is valid (has positive area).
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max.x > self.min.x && self.max.y > self.min.y
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::invalid()
    }
}

/// RGBA color representation.
///
/// Colors are stored as u8 components (0-255) for efficient storage
/// and converted to f32 (0.0-1.0) when uploading to the GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbaColor {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha/opacity component (0-255)
    pub a: u8,
}

impl RgbaColor {
    /// Creates a new color from u8 components.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255)
    #[inline]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Converts this color to an f32 array for GPU upload.
    ///
    /// Each component is normalized to [0.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_renderers::RgbaColor;
    ///
    /// let red = RgbaColor::new(255, 0, 0, 255);
    /// let [r, g, b, a] = red.to_f32_array();
    /// assert_eq!(r, 1.0);
    /// assert_eq!(g, 0.0);
    /// assert_eq!(b, 0.0);
    /// assert_eq!(a, 1.0);
    /// ```
    #[inline]
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Returns a fully transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Returns a fully opaque white color.
    #[inline]
    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }

    /// Returns a fully opaque black color.
    #[inline]
    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
    }

    /// Returns a fully opaque red color.
    #[inline]
    pub fn red() -> Self {
        Self::new(255, 0, 0, 255)
    }

    /// Returns a fully opaque green color.
    #[inline]
    pub fn green() -> Self {
        Self::new(0, 255, 0, 255)
    }

    /// Returns a fully opaque blue color.
    #[inline]
    pub fn blue() -> Self {
        Self::new(0, 0, 255, 255)
    }

    /// Creates a color from linear RGB f32 components.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0.0-1.0)
    /// * `g` - Green component (0.0-1.0)
    /// * `b` - Blue component (0.0-1.0)
    /// * `a` - Alpha component (0.0-1.0)
    ///
    /// # Panics
    ///
    /// Panics if any component is outside [0.0, 1.0].
    #[inline]
    pub fn from_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        assert!((0.0..=1.0).contains(&r), "r must be in [0, 1]");
        assert!((0.0..=1.0).contains(&g), "g must be in [0, 1]");
        assert!((0.0..=1.0).contains(&b), "b must be in [0, 1]");
        assert!((0.0..=1.0).contains(&a), "a must be in [0, 1]");

        Self::new(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    /// Returns true if this color is fully transparent.
    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::EPSILON;

    /// Test helper struct implementing Renderable
    #[derive(Clone, Debug, PartialEq)]
    struct TestRenderable {
        bounds: Bounds,
        priority: i32,
        material_id: u64,
        color: RgbaColor,
    }

    impl TestRenderable {
        fn new(bounds: Bounds) -> Self {
            Self {
                bounds,
                priority: 0,
                material_id: 1,
                color: RgbaColor::white(),
            }
        }

        fn with_priority(priority: i32) -> Self {
            Self {
                bounds: Bounds::invalid(),
                priority,
                material_id: 1,
                color: RgbaColor::white(),
            }
        }
    }

    impl Renderable for TestRenderable {
        fn bounds(&self) -> Option<Bounds> {
            Some(self.bounds)
        }

        fn contains_point(&self, point: Vec2) -> bool {
            self.bounds.contains(point)
        }

        fn render_priority(&self) -> i32 {
            self.priority
        }

        fn material_id(&self) -> u64 {
            self.material_id
        }

        fn color(&self) -> RgbaColor {
            self.color
        }
    }

    // === Bounds Tests ===

    #[test]
    fn test_bounds_creation() {
        let bounds = Bounds::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert_eq!(bounds.min, Vec2::new(0.0, 0.0));
        assert_eq!(bounds.max, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_bounds_center() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let center = bounds.center();
        assert!((center.x - 50.0).abs() < EPSILON);
        assert!((center.y - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_bounds_width_height() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 50.0));
        assert!((bounds.width() - 100.0).abs() < EPSILON);
        assert!((bounds.height() - 50.0).abs() < EPSILON);
    }

    #[test]
    fn test_bounds_from_center_size() {
        let bounds = Bounds::from_center_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));
        assert!((bounds.min.x - 75.0).abs() < EPSILON);
        assert!((bounds.min.y - 85.0).abs() < EPSILON);
        assert!((bounds.max.x - 125.0).abs() < EPSILON);
        assert!((bounds.max.y - 115.0).abs() < EPSILON);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        assert!(bounds.contains(Vec2::new(50.0, 50.0)));
        assert!(bounds.contains(Vec2::new(0.0, 0.0)));
        assert!(bounds.contains(Vec2::new(100.0, 100.0)));
        assert!(!bounds.contains(Vec2::new(150.0, 50.0)));
    }

    #[test]
    fn test_bounds_intersects() {
        let a = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let b = Bounds::new(Vec2::new(50.0, 50.0), Vec2::new(150.0, 150.0));
        let c = Bounds::new(Vec2::new(200.0, 200.0), Vec2::new(300.0, 300.0));

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_bounds_invalid() {
        let bounds = Bounds::invalid();
        assert!(!bounds.is_valid());
    }

    #[test]
    fn test_bounds_default() {
        let bounds = Bounds::default();
        assert!(!bounds.is_valid());
    }

    // === RgbaColor Tests ===

    #[test]
    fn test_rgba_color_creation() {
        let color = RgbaColor::new(255, 128, 64, 255);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_rgba_color_to_f32_array() {
        let color = RgbaColor::new(255, 128, 0, 255);
        let [r, g, b, a] = color.to_f32_array();

        assert!((r - 1.0).abs() < 0.01, "r should be ~1.0, got {}", r);
        assert!((g - 0.5).abs() < 0.01, "g should be ~0.5, got {}", g);
        assert!((b - 0.0).abs() < 0.01, "b should be ~0.0, got {}", b);
        assert!((a - 1.0).abs() < 0.01, "a should be ~1.0, got {}", a);
    }

    #[test]
    fn test_rgba_color_transparent() {
        let color = RgbaColor::transparent();
        assert_eq!(color, RgbaColor::new(0, 0, 0, 0));
    }

    #[test]
    fn test_rgba_color_white() {
        let color = RgbaColor::white();
        assert_eq!(color, RgbaColor::new(255, 255, 255, 255));
    }

    #[test]
    fn test_rgba_color_black() {
        let color = RgbaColor::black();
        assert_eq!(color, RgbaColor::new(0, 0, 0, 255));
    }

    #[test]
    fn test_rgba_color_from_linear() {
        let color = RgbaColor::from_linear(0.5, 0.25, 0.75, 1.0);
        assert_eq!(color.r, 127);
        assert_eq!(color.g, 63);
        assert_eq!(color.b, 191);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_rgba_color_is_transparent() {
        let opaque = RgbaColor::white();
        let transparent = RgbaColor::transparent();

        assert!(!opaque.is_transparent());
        assert!(transparent.is_transparent());
    }

    // === Renderable Trait Tests ===

    #[test]
    fn test_renderable_bounds() {
        let renderable = TestRenderable::new(Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        let bounds = renderable.bounds().unwrap();
        assert_eq!(bounds.min, Vec2::ZERO);
        assert_eq!(bounds.max, Vec2::new(100.0, 100.0));
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

    #[test]
    fn test_renderable_material_id() {
        let renderable = TestRenderable::new(Bounds::invalid());
        assert_eq!(renderable.material_id(), 1);
    }

    #[test]
    fn test_renderable_color() {
        let renderable = TestRenderable::new(Bounds::invalid());
        assert_eq!(renderable.color(), RgbaColor::white());
    }
}
