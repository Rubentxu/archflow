// Copyright 2024 ArchFlow Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Geometry module - Vec2 and Bounds wrappers.

use glam::Vec2 as GlamVec2;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// 2D Vector wrapper sobre glam::Vec2 con API simplificada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2(GlamVec2);

impl Vec2 {
    /// Creates a new Vec2.
    pub fn new(x: f32, y: f32) -> Self {
        Self(GlamVec2::new(x, y))
    }

    /// Returns the x component.
    pub fn x(&self) -> f32 {
        self.0.x
    }

    /// Returns the y component.
    pub fn y(&self) -> f32 {
        self.0.y
    }

    /// Sets the x component.
    pub fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }

    /// Sets the y component.
    pub fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }

    /// Returns the length of the vector.
    pub fn length(&self) -> f32 {
        self.0.length()
    }

    /// Returns the squared length of the vector.
    pub fn length_squared(&self) -> f32 {
        self.0.length_squared()
    }

    /// Returns a normalized vector (unit length).
    pub fn normalize(&self) -> Self {
        if self.0 == GlamVec2::ZERO {
            Self(GlamVec2::ZERO)
        } else {
            Self(self.0.normalize())
        }
    }

    /// Calculates the dot product with another vector.
    pub fn dot(&self, other: Vec2) -> f32 {
        self.0.dot(other.0)
    }

    /// Calculates the 2D cross product (scalar).
    pub fn cross(&self, other: Vec2) -> f32 {
        self.0.x * other.0.y - self.0.y * other.0.x
    }

    /// Calculates the distance to another point.
    pub fn distance_to(&self, other: Vec2) -> f32 {
        self.0.distance(other.0)
    }

    /// Linear interpolation between two vectors.
    pub fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
        Self(a.0.lerp(b.0, t))
    }

    /// Returns the angle of the vector in radians from the positive x-axis.
    pub fn angle(&self) -> f32 {
        self.0.y.atan2(self.0.x)
    }

    /// Rotates the vector by an angle (in radians).
    pub fn rotate(&self, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(
            self.0.x * cos - self.0.y * sin,
            self.0.x * sin + self.0.y * cos,
        )
    }

    /// Returns the perpendicular vector (90 degrees counter-clockwise).
    pub fn perp(&self) -> Vec2 {
        Self::new(-self.0.y, self.0.x)
    }

    /// Element-wise minimum.
    pub fn min(a: Vec2, b: Vec2) -> Vec2 {
        Self(GlamVec2::new(a.0.x.min(b.0.x), a.0.y.min(b.0.y)))
    }

    /// Element-wise maximum.
    pub fn max(a: Vec2, b: Vec2) -> Vec2 {
        Self(GlamVec2::new(a.0.x.max(b.0.x), a.0.y.max(b.0.y)))
    }

    /// Element-wise clamp.
    pub fn clamp(&self, min: Vec2, max: Vec2) -> Vec2 {
        Self(GlamVec2::new(
            self.0.x.clamp(min.0.x, max.0.x),
            self.0.y.clamp(min.0.y, max.0.y),
        ))
    }

    /// Constant zero vector.
    pub const ZERO: Vec2 = Vec2(GlamVec2::ZERO);

    /// Constant one vector.
    pub const ONE: Vec2 = Vec2(GlamVec2::ONE);

    /// Unit vector along x-axis.
    pub const X: Vec2 = Vec2(GlamVec2::X);

    /// Unit vector along y-axis.
    pub const Y: Vec2 = Vec2(GlamVec2::Y);
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2(self.0 + other.0)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2(self.0 - other.0)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2(self.0 * scalar)
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, scalar: f32) -> Vec2 {
        Vec2(self.0 / scalar)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.0 += other.0;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Vec2) {
        self.0 -= other.0;
    }
}

impl std::ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, scalar: f32) {
        self.0 *= scalar;
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Bounds {
    /// Creates a new bounds from position and size.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the x coordinate.
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Returns the y coordinate.
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Returns the width.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Returns the height.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Returns the minimum point (top-left corner).
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Returns the maximum point (bottom-right corner).
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Returns the center point.
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns the area.
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Returns the perimeter.
    pub fn perimeter(&self) -> f32 {
        2.0 * (self.width + self.height)
    }

    /// Returns the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }

    /// Checks if a point is inside the bounds.
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x() >= self.x
            && point.x() <= self.x + self.width
            && point.y() >= self.y
            && point.y() <= self.y + self.height
    }

    /// Checks if this bounds intersects with another.
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Returns the union of two bounds.
    pub fn union(&self, other: &Bounds) -> Bounds {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);

        Bounds::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Returns the intersection of two bounds, if any.
    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        let min_x = self.x.max(other.x);
        let min_y = self.y.max(other.y);
        let max_x = (self.x + self.width).min(other.x + other.width);
        let max_y = (self.y + self.height).min(other.y + other.height);

        if max_x > min_x && max_y > min_y {
            Some(Bounds::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }

    /// Expands the bounds by a margin on all sides.
    pub fn expanded(&self, margin: f32) -> Bounds {
        Bounds::new(
            self.x - margin,
            self.y - margin,
            self.width + 2.0 * margin,
            self.height + 2.0 * margin,
        )
    }

    /// Scales the bounds from the center.
    pub fn scaled(&self, factor: f32) -> Bounds {
        let center = self.center();
        Bounds::new(
            center.x() - self.width * factor / 2.0,
            center.y() - self.height * factor / 2.0,
            self.width * factor,
            self.height * factor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_add() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        let result = a + b;
        assert_eq!(result.x(), 4.0);
        assert_eq!(result.y(), 6.0);
    }

    #[test]
    fn test_vec2_sub() {
        let a = Vec2::new(5.0, 7.0);
        let b = Vec2::new(2.0, 3.0);
        let result = a - b;
        assert_eq!(result.x(), 3.0);
        assert_eq!(result.y(), 4.0);
    }

    #[test]
    fn test_dot_product() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.dot(b), 11.0);
    }

    #[test]
    fn test_cross_product() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(a.cross(b), 1.0);
    }

    #[test]
    fn test_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let normalized = v.normalize();
        assert!((normalized.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        assert_eq!(Vec2::lerp(a, b, 0.5), Vec2::new(5.0, 10.0));
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        assert!(bounds.contains_point(Vec2::new(50.0, 50.0)));
        assert!(!bounds.contains_point(Vec2::new(150.0, 50.0)));
    }

    #[test]
    fn test_bounds_intersects() {
        let a = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let b = Bounds::new(50.0, 50.0, 100.0, 100.0);
        assert!(a.intersects(&b));

        let c = Bounds::new(200.0, 200.0, 50.0, 50.0);
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_bounds_union() {
        let a = Bounds::new(0.0, 0.0, 50.0, 50.0);
        let b = Bounds::new(50.0, 50.0, 50.0, 50.0);
        let union = a.union(&b);
        assert_eq!(union.x(), 0.0);
        assert_eq!(union.y(), 0.0);
        assert_eq!(union.width(), 100.0);
        assert_eq!(union.height(), 100.0);
    }

    #[test]
    fn test_bounds_intersection() {
        let a = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let b = Bounds::new(50.0, 50.0, 100.0, 100.0);
        let intersection = a.intersection(&b).unwrap();
        assert_eq!(intersection.x(), 50.0);
        assert_eq!(intersection.y(), 50.0);
        assert_eq!(intersection.width(), 50.0);
        assert_eq!(intersection.height(), 50.0);
    }
}
