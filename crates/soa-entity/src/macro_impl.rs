//! SOA Expandable Trait
//!
//! Trait for expanding complex types into SOA-compatible arrays.

use archflow_core::{Color, Vec2};

/// Trait for expanding complex types into SOA-compatible arrays.
///
/// Types that are composed of multiple primitive values should implement
/// this trait to enable automatic expansion by the SOA macro.
///
/// # Examples
///
/// ```
/// use soa_entity::SoaExpandable;
/// use archflow_core::Vec2;
///
/// impl SoaExpandable for Vec2 {
///     type Output = (&'static [f32], &'static [f32]);
///
///     fn expand(&self) -> Self::Output {
///         (&[self.x], &[self.y])
///     }
/// }
/// ```
pub trait SoaExpandable {
    /// The expanded SOA array type (tuple of slices).
    type Output: 'static;

    /// Expand this value into SOA arrays.
    fn expand(&self) -> Self::Output;
}

/// Blanket implementation for all types that don't need expansion.
///
/// Simple types like `f32`, `i32`, `bool` are stored as-is.
impl<T> SoaExpandable for T {
    type Output = &'static [T];

    fn expand(&self) -> Self::Output {
        // This is a simplified version - in real SOA we'd expand to separate arrays
        // For now, return a single-element slice
        // Note: This is a limitation of the current simplified implementation
        unimplemented!("Simple types cannot be expanded without macro")
    }
}

// Vec2 expansion
impl SoaExpandable for Vec2 {
    type Output = (f32, f32);

    fn expand(&self) -> Self::Output {
        (self.x, self.y)
    }
}

// Color expansion
impl SoaExpandable for Color {
    type Output = (u8, u8, u8, u8);

    fn expand(&self) -> Self::Output {
        (self.r, self.g, self.b, self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vec2 expansion
    #[test]
    fn test_expand_vec2() {
        let vec = Vec2::new(10.0, 20.0);
        let (x, y) = vec.expand();

        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
    }

    // Color expansion
    #[test]
    fn test_expand_color() {
        let col = Color::new(255, 128, 64, 255);
        let (r, g, b, a) = col.expand();

        assert_eq!(r, 255);
        assert_eq!(g, 128);
        assert_eq!(b, 64);
        assert_eq!(a, 255);
    }
}
