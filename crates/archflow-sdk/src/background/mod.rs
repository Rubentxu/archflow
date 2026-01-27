//! Background and grid rendering system
//!
//! This module provides the background renderer for grids, dots, and other
//! background patterns used in the canvas.

use archflow_core::{Color, Vec2};
use serde::{Deserialize, Serialize};

/// Type of grid to render
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridType {
    /// Dot grid - evenly spaced dots
    Dots,
    /// Line grid - horizontal and vertical lines
    Lines,
    /// Isometric grid - triangular pattern
    Isometric,
}

impl Default for GridType {
    fn default() -> Self {
        Self::Dots
    }
}

/// Configuration for grid rendering
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GridConfig {
    /// Type of grid to render
    pub grid_type: GridType,
    /// Spacing between grid elements in canvas units
    pub spacing: f32,
    /// Radius of dots (for dot grid)
    pub dot_radius: f32,
    /// Color of dots (for dot grid)
    pub dot_color: Color,
    /// Color of lines (for line grid)
    pub line_color: Color,
    /// Width of lines (for line grid)
    pub line_width: f32,
    /// Whether the grid is visible
    pub show_grid: bool,
    /// Grid offset for alignment
    pub offset: Vec2,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            grid_type: GridType::default(),
            spacing: 20.0,
            dot_radius: 1.5,
            dot_color: Color::rgba(0.5, 0.5, 0.5, 0.3),
            line_color: Color::rgba(0.5, 0.5, 0.5, 0.2),
            line_width: 1.0,
            show_grid: true,
            offset: Vec2::ZERO,
        }
    }
}

impl GridConfig {
    /// Creates a new grid configuration with dots
    #[inline]
    pub fn dots(spacing: f32) -> Self {
        Self {
            spacing,
            ..Default::default()
        }
    }

    /// Creates a new grid configuration with lines
    #[inline]
    pub fn lines(spacing: f32) -> Self {
        Self {
            grid_type: GridType::Lines,
            spacing,
            ..Default::default()
        }
    }

    /// Creates a new grid configuration with isometric pattern
    #[inline]
    pub fn isometric(spacing: f32) -> Self {
        Self {
            grid_type: GridType::Isometric,
            spacing,
            ..Default::default()
        }
    }

    /// Sets the grid visibility
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.show_grid = visible;
    }

    /// Sets the spacing
    #[inline]
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
    }

    /// Sets the dot radius
    #[inline]
    pub fn set_dot_radius(&mut self, radius: f32) {
        self.dot_radius = radius;
    }

    /// Sets the dot color
    #[inline]
    pub fn set_dot_color(&mut self, color: Color) {
        self.dot_color = color;
    }

    /// Sets the line color
    #[inline]
    pub fn set_line_color(&mut self, color: Color) {
        self.line_color = color;
    }

    /// Sets the line width
    #[inline]
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width;
    }
}

/// Background renderer for the canvas
///
/// This struct manages the rendering of backgrounds and grids.
///
#[derive(Debug, Default)]
pub struct BackgroundRenderer {
    /// Current grid configuration
    grid_config: GridConfig,
    /// Background color
    background_color: Color,
}

impl BackgroundRenderer {
    /// Creates a new background renderer
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current grid configuration
    #[inline]
    pub fn grid_config(&self) -> &GridConfig {
        &self.grid_config
    }

    /// Sets the grid configuration
    #[inline]
    pub fn set_grid_config(&mut self, config: GridConfig) {
        self.grid_config = config;
    }

    /// Gets the background color
    #[inline]
    pub fn background_color(&self) -> Color {
        self.background_color
    }

    /// Sets the background color
    #[inline]
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    /// Hides the grid
    #[inline]
    pub fn hide_grid(&mut self) {
        self.grid_config.show_grid = false;
    }

    /// Shows dots grid
    #[inline]
    pub fn show_dots(&mut self, spacing: Option<f32>) {
        self.grid_config.grid_type = GridType::Dots;
        self.grid_config.show_grid = true;
        if let Some(spacing) = spacing {
            self.grid_config.spacing = spacing;
        }
    }

    /// Shows lines grid
    #[inline]
    pub fn show_lines(&mut self, spacing: Option<f32>) {
        self.grid_config.grid_type = GridType::Lines;
        self.grid_config.show_grid = true;
        if let Some(spacing) = spacing {
            self.grid_config.spacing = spacing;
        }
    }

    /// Shows isometric grid
    #[inline]
    pub fn show_isometric(&mut self, spacing: Option<f32>) {
        self.grid_config.grid_type = GridType::Isometric;
        self.grid_config.show_grid = true;
        if let Some(spacing) = spacing {
            self.grid_config.spacing = spacing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_config_defaults() {
        let config = GridConfig::default();
        assert!(config.show_grid);
        assert_eq!(config.spacing, 20.0);
        assert_eq!(config.grid_type, GridType::Dots);
    }

    #[test]
    fn test_grid_config_dots() {
        let config = GridConfig::dots(30.0);
        assert_eq!(config.spacing, 30.0);
        assert_eq!(config.grid_type, GridType::Dots);
    }

    #[test]
    fn test_grid_config_lines() {
        let config = GridConfig::lines(50.0);
        assert_eq!(config.spacing, 50.0);
        assert_eq!(config.grid_type, GridType::Lines);
    }

    #[test]
    fn test_background_renderer() {
        let renderer = BackgroundRenderer::new();
        assert!(renderer.grid_config().show_grid);
        assert_eq!(renderer.grid_config().spacing, 20.0);
    }
}
