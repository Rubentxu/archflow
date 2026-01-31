// ═══════════════════════════════════════════════════════════════════════════════
// Value Objects - Domain-Specific Types
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 3.2
//
// Value Objects are immutable types that represent domain concepts.
// They are identified by their attributes rather than an identity.
//
// Examples:
// - Position: A 2D position in world space
// - Size: A 2D size (width, height)
// - Bounds: An axis-aligned bounding box
// ═══════════════════════════════════════════════════════════════════════════════

use crate::math::{Rect, Vec2};

// ═══════════════════════════════════════════════════════════════════════════════
// POSITION - 2D Position in World Space
// ═══════════════════════════════════════════════════════════════════════════════

/// A 2D position in world space
///
/// This is a thin wrapper around Vec2 that provides semantic meaning.
/// Positions are always in world coordinates (not screen coordinates).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Position {
    /// X coordinate in world space
    pub x: f32,
    /// Y coordinate in world space
    pub y: f32,
}

impl Position {
    /// Create a new position
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a position at the origin
    #[inline(always)]
    pub const fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Convert to Vec2
    #[inline(always)]
    pub const fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Create from Vec2
    #[inline(always)]
    pub const fn from_vec2(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }

    /// Calculate distance to another position
    #[inline(always)]
    pub fn distance_to(self, other: Position) -> f32 {
        self.to_vec2().distance(other.to_vec2())
    }

    /// Calculate squared distance to another position (faster, no sqrt)
    #[inline(always)]
    pub fn distance_squared_to(self, other: Position) -> f32 {
        self.to_vec2().distance_squared(other.to_vec2())
    }
}

impl From<Vec2> for Position {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        Self::from_vec2(v)
    }
}

impl From<Position> for Vec2 {
    #[inline(always)]
    fn from(p: Position) -> Self {
        p.to_vec2()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIZE - 2D Size (Width, Height)
// ═══════════════════════════════════════════════════════════════════════════════

/// A 2D size (width, height)
///
/// Sizes are always positive values.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Size {
    /// Width in world units
    pub width: f32,
    /// Height in world units
    pub height: f32,
}

impl Size {
    /// Create a new size
    ///
    /// # Panics
    /// Panics if width or height is negative
    #[inline(always)]
    pub fn new(width: f32, height: f32) -> Self {
        assert!(width >= 0.0, "Width must be non-negative");
        assert!(height >= 0.0, "Height must be non-negative");
        Self { width, height }
    }

    /// Create a new size without validation (unsafe)
    ///
    /// # Safety
    /// Caller must ensure width and height are non-negative
    #[inline(always)]
    pub const unsafe fn new_unchecked(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a square size
    #[inline(always)]
    pub fn square(side: f32) -> Self {
        Self::new(side, side)
    }

    /// Create a zero size
    #[inline(always)]
    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Convert to Vec2
    #[inline(always)]
    pub const fn to_vec2(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Create from Vec2
    #[inline(always)]
    pub fn from_vec2(v: Vec2) -> Self {
        Self::new(v.x, v.y)
    }

    /// Calculate area
    #[inline(always)]
    pub fn area(self) -> f32 {
        self.width * self.height
    }

    /// Calculate aspect ratio (width / height)
    #[inline(always)]
    pub fn aspect_ratio(self) -> f32 {
        if self.height == 0.0 {
            return f32::INFINITY;
        }
        self.width / self.height
    }

    /// Check if the size is zero
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    /// Scale the size by a factor
    #[inline(always)]
    pub fn scale(self, factor: f32) -> Size {
        Size::new(self.width * factor, self.height * factor)
    }
}

impl From<Vec2> for Size {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        Self::from_vec2(v)
    }
}

impl From<Size> for Vec2 {
    #[inline(always)]
    fn from(s: Size) -> Self {
        s.to_vec2()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOUNDS - Axis-Aligned Bounding Box
// ═══════════════════════════════════════════════════════════════════════════════

/// An axis-aligned bounding box
///
/// This is a newtype wrapper around Rect that provides semantic meaning
/// for bounding boxes in the domain.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Bounds {
    /// The underlying rectangle
    pub rect: Rect,
}

impl Bounds {
    /// Create bounds from a rectangle
    #[inline(always)]
    pub const fn from_rect(rect: Rect) -> Self {
        Self { rect }
    }

    /// Create bounds from center and size
    #[inline(always)]
    pub fn from_center_size(center: Position, size: Size) -> Self {
        Self {
            rect: Rect::from_center_size(center.to_vec2(), size.to_vec2()),
        }
    }

    /// Create bounds from min and max corners
    #[inline(always)]
    pub fn from_min_max(min: Position, max: Position) -> Self {
        Self {
            rect: Rect::new(min.x, min.y, max.x, max.y),
        }
    }

    /// Get the minimum corner
    #[inline(always)]
    pub const fn min(self) -> Position {
        Position::new(self.rect.min.x, self.rect.min.y)
    }

    /// Get the maximum corner
    #[inline(always)]
    pub const fn max(self) -> Position {
        Position::new(self.rect.max.x, self.rect.max.y)
    }

    /// Get the center position
    #[inline(always)]
    pub fn center(self) -> Position {
        Position::from_vec2(self.rect.center())
    }

    /// Get the size
    #[inline(always)]
    pub fn size(self) -> Size {
        Size::from_vec2(self.rect.size())
    }

    /// Check if a position is contained within these bounds
    #[inline(always)]
    pub fn contains(self, position: Position) -> bool {
        self.rect.contains(position.to_vec2())
    }

    /// Check if these bounds intersect another
    #[inline(always)]
    pub fn intersects(self, other: Bounds) -> bool {
        self.rect.intersects(&other.rect)
    }

    /// Expand bounds by a margin
    #[inline(always)]
    pub fn inflate(self, margin: f32) -> Bounds {
        Bounds {
            rect: self.rect.inflate(margin),
        }
    }

    /// Get the union of two bounds
    #[inline(always)]
    pub fn union(self, other: Bounds) -> Bounds {
        Bounds {
            rect: self.rect.union(&other.rect),
        }
    }

    /// Get the area of the bounds
    #[inline(always)]
    pub fn area(self) -> f32 {
        self.rect.area()
    }

    /// Check if the bounds are empty
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.rect.is_empty()
    }
}

impl From<Rect> for Bounds {
    #[inline(always)]
    fn from(rect: Rect) -> Self {
        Self::from_rect(rect)
    }
}

impl From<Bounds> for Rect {
    #[inline(always)]
    fn from(bounds: Bounds) -> Self {
        bounds.rect
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let a = Position::new(0.0, 0.0);
        let b = Position::new(3.0, 4.0);
        assert_eq!(a.distance_to(b), 5.0);
    }

    #[test]
    fn test_size_area() {
        let size = Size::new(10.0, 5.0);
        assert_eq!(size.area(), 50.0);
    }

    #[test]
    fn test_size_aspect_ratio() {
        let size = Size::new(16.0, 9.0);
        assert!((size.aspect_ratio() - 16.0 / 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bounds_from_center_size() {
        let bounds = Bounds::from_center_size(Position::new(10.0, 10.0), Size::new(10.0, 6.0));
        assert_eq!(bounds.min(), Position::new(5.0, 7.0));
        assert_eq!(bounds.max(), Position::new(15.0, 13.0));
        assert_eq!(bounds.center(), Position::new(10.0, 10.0));
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::from_min_max(Position::new(0.0, 0.0), Position::new(10.0, 10.0));
        assert!(bounds.contains(Position::new(5.0, 5.0)));
        assert!(!bounds.contains(Position::new(15.0, 5.0)));
    }

    #[test]
    #[should_panic(expected = "Width must be non-negative")]
    fn test_size_negative_panics() {
        Size::new(-10.0, 5.0);
    }
}
