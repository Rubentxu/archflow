//! Viewport management for pan and zoom operations
//!
//! This module consolidates viewport functionality from `archflow-spatial`
//! and `archflow-sdk` into the canvas bounded context.

use crate::Vec2;

/// Viewport for managing pan and zoom state.
///
/// Represents the visible area of the canvas in screen coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Center position in canvas coordinates
    pub center: Vec2,
    /// Zoom level (1.0 = 100%, 2.0 = 200%, etc.)
    pub zoom: f32,
    /// Screen width in pixels
    pub screen_width: f32,
    /// Screen height in pixels
    pub screen_height: f32,
}

impl Viewport {
    /// Creates a new viewport with the given dimensions.
    #[inline]
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            screen_width,
            screen_height,
        }
    }

    /// Returns the screen width.
    #[inline]
    pub fn width(&self) -> f32 {
        self.screen_width
    }

    /// Returns the screen height.
    #[inline]
    pub fn height(&self) -> f32 {
        self.screen_height
    }

    /// Converts a screen coordinate to canvas coordinate.
    #[inline]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        let half_size = Vec2::new(self.screen_width / 2.0, self.screen_height / 2.0);
        (screen - half_size) / self.zoom + self.center
    }

    /// Converts a canvas coordinate to screen coordinate.
    #[inline]
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        let half_size = Vec2::new(self.screen_width / 2.0, self.screen_height / 2.0);
        (canvas - self.center) * self.zoom + half_size
    }
}
