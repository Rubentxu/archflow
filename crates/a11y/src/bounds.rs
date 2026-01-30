//! Bounding box for accessibility

use crate::Vec2;
use serde::{Deserialize, Serialize};

/// Bounding box for accessibility
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl A11yBounds {
    /// Creates new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center point of the bounds
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns the minimum corner
    pub fn min(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Returns the maximum corner
    pub fn max(&self) -> Vec2 {
        Vec2::new(self.x + self.width, self.y + self.height)
    }

    /// Checks if a point is contained within the bounds
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

impl Default for A11yBounds {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}
