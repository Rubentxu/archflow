//! Viewport management for the infinite canvas
//!
//! The viewport represents the visible area of the infinite canvas,
//! including the offset, zoom level, and bounds.

use archflow_core::{Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for viewport operations
#[derive(Debug, thiserror::Error)]
pub enum ViewportError {
    #[error("Invalid zoom level: {0}")]
    InvalidZoom(f32),
    #[error("Viewport bounds error: {0}")]
    BoundsError(&'static str),
}

/// Represents the visible area of the infinite canvas
///
/// The viewport defines which portion of the canvas is visible to the user,
/// including the origin offset (top-left corner in canvas coordinates) and
/// the zoom level. Zoom limits are configurable through the builder pattern.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// Position of the viewport origin in canvas coordinates
    pub offset: Vec2,
    /// Zoom level (1.0 = 100%, 2.0 = 200%, etc.)
    pub zoom: f32,
    /// Minimum allowed zoom level
    pub min_zoom: f32,
    /// Maximum allowed zoom level
    pub max_zoom: f32,
}

impl Viewport {
    /// Creates a new viewport with the given offset and zoom
    ///
    /// # Arguments
    ///
    /// * `offset` - The position of the top-left corner in canvas coordinates
    /// * `zoom` - The zoom level (must be positive)
    ///
    /// # Panics
    ///
    /// Panics if `zoom` is not positive
    #[inline]
    pub fn new(offset: Vec2, zoom: f32) -> Self {
        assert!(zoom > 0.0, "Zoom must be positive");
        Self {
            offset,
            zoom,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Creates a viewport that shows the entire given bounds
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangular area to fit
    /// * `screen_width` - Width of the screen/canvas
    /// * `screen_height` - Height of the screen/canvas
    ///
    /// # Returns
    ///
    /// A viewport that shows the entire bounds
    #[inline]
    pub fn fit_bounds(bounds: Rect, screen_width: f32, screen_height: f32) -> Self {
        if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
            return Self::default();
        }

        let center = bounds.center();
        let aspect = screen_width / screen_height;
        let bounds_aspect = bounds.width() / bounds.height();

        let zoom = if aspect > bounds_aspect {
            screen_height / bounds.height()
        } else {
            screen_width / bounds.width()
        };

        let zoom = (zoom * 0.9).clamp(0.1, 10.0);

        Self {
            offset: center - Vec2::new((screen_width / 2.0) / zoom, (screen_height / 2.0) / zoom),
            zoom,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Converts a screen coordinate to a canvas coordinate
    ///
    /// # Arguments
    ///
    /// * `screen` - Position in screen coordinates
    ///
    /// # Returns
    ///
    /// Equivalent position in canvas coordinates
    #[inline]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        (screen - Vec2::new(self.offset.x * self.zoom, self.offset.y * self.zoom)) / self.zoom
    }

    /// Converts a canvas coordinate to a screen coordinate
    ///
    /// # Arguments
    ///
    /// * `canvas` - Position in canvas coordinates
    ///
    /// # Returns
    ///
    /// Equivalent position in screen coordinates
    #[inline]
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        canvas * self.zoom + self.offset * self.zoom
    }

    /// Returns the visible bounds in canvas coordinates
    ///
    /// # Arguments
    ///
    /// * `screen_width` - Width of the screen
    /// * `screen_height` - Height of the screen
    ///
    /// # Returns
    ///
    /// The visible rectangle in canvas coordinates
    #[inline]
    pub fn visible_bounds(&self, screen_width: f32, screen_height: f32) -> Rect {
        let min = self.screen_to_canvas(Vec2::ZERO);
        let max = self.screen_to_canvas(Vec2::new(screen_width, screen_height));
        Rect::from_min_max(min, max)
    }

    /// Returns the visible area in screen coordinates
    ///
    /// # Arguments
    ///
    /// * `screen_width` - Width of the screen
    /// * `screen_height` - Height of the screen
    ///
    /// # Returns
    ///
    /// The screen rectangle
    #[inline]
    pub fn screen_bounds(&self, screen_width: f32, screen_height: f32) -> Rect {
        Rect::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(screen_width, screen_height))
    }

    /// Applies a zoom operation centered on the given screen point
    ///
    /// # Arguments
    ///
    /// * `screen_center` - The screen point to zoom around
    /// * `zoom_factor` - The factor to multiply the current zoom by
    ///
    /// # Returns
    ///
    /// The new viewport after zooming
    #[inline]
    pub fn zoom_at(&self, screen_center: Vec2, zoom_factor: f32) -> Self {
        let new_zoom = (self.zoom * zoom_factor).clamp(self.min_zoom, self.max_zoom);

        // To keep the screen_center point at the same canvas position:
        // canvas = screen / zoom - offset
        // new_offset = screen / new_zoom - canvas
        // new_offset = screen / new_zoom - (screen / old_zoom - old_offset)
        // new_offset = old_offset + screen * (1/new_zoom - 1/old_zoom)
        let new_offset = self.offset + screen_center * (1.0 / new_zoom - 1.0 / self.zoom);

        Self {
            offset: new_offset,
            zoom: new_zoom,
            min_zoom: self.min_zoom,
            max_zoom: self.max_zoom,
        }
    }

    /// Applies a pan operation
    ///
    /// # Arguments
    ///
    /// * `delta` - The delta to add to the offset (in screen pixels, will be scaled)
    ///
    /// # Returns
    ///
    /// The new viewport after panning
    #[inline]
    pub fn pan(&self, delta: Vec2) -> Self {
        Self {
            offset: self.offset + delta / self.zoom,
            zoom: self.zoom,
            min_zoom: self.min_zoom,
            max_zoom: self.max_zoom,
        }
    }

    /// Constrains the viewport to reasonable bounds
    ///
    /// This prevents the user from getting lost in the infinite canvas
    /// by limiting the offset to a large but finite area.
    #[inline]
    pub fn constrain(&self, bounds: Option<Rect>) -> Self {
        let Some(bounds) = bounds else {
            return *self;
        };

        Self {
            offset: Vec2::new(
                self.offset.x.clamp(bounds.min.x, bounds.max.x),
                self.offset.y.clamp(bounds.min.y, bounds.max.y),
            ),
            zoom: self.zoom,
            min_zoom: self.min_zoom,
            max_zoom: self.max_zoom,
        }
    }
}

impl fmt::Display for Viewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Viewport(offset=({}, {}), zoom={:.2})",
            self.offset.x, self.offset.y, self.zoom
        )
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

/// Builder pattern for creating Viewport instances with custom configuration.
///
/// This eliminates Connascence of Position by providing explicit, chainable
/// methods for configuration instead of requiring parameters in a specific order.
///
/// # Example
///
/// ```rust
/// use archflow_sdk::viewport::{Viewport, ViewportBuilder};
///
/// let viewport = ViewportBuilder::new()
///     .with_offset(archflow_core::Vec2::new(100.0, 50.0))
///     .with_zoom(1.5)
///     .with_zoom_limits(0.05, 20.0)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ViewportBuilder {
    offset: Option<Vec2>,
    zoom: Option<f32>,
    min_zoom: f32,
    max_zoom: f32,
    screen_size: Option<(f32, f32)>,
    content_bounds: Option<Rect>,
}

impl ViewportBuilder {
    /// Creates a new viewport builder with default values.
    ///
    /// Default zoom limits are 0.1 (10%) to 10.0 (1000%).
    #[inline]
    pub fn new() -> Self {
        Self {
            offset: None,
            zoom: None,
            min_zoom: 0.1,
            max_zoom: 10.0,
            screen_size: None,
            content_bounds: None,
        }
    }

    /// Sets the viewport offset (top-left corner in canvas coordinates).
    ///
    /// # Arguments
    ///
    /// * `offset` - The position of the viewport origin
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    #[inline]
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the zoom level.
    ///
    /// # Arguments
    ///
    /// * `zoom` - The zoom level (must be positive)
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    ///
    /// # Panics
    ///
    /// Will panic if zoom is not positive when [`build`] is called
    ///
    /// [`build`]: #method.build
    #[inline]
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = Some(zoom);
        self
    }

    /// Sets custom zoom limits.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum zoom level (must be positive and less than max)
    /// * `max` - Maximum zoom level (must be greater than min)
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    ///
    /// # Panics
    ///
    /// Will panic if min is not positive or max is not greater than min
    /// when [`build`] is called
    ///
    /// [`build`]: #method.build
    #[inline]
    pub fn with_zoom_limits(mut self, min: f32, max: f32) -> Self {
        self.min_zoom = min;
        self.max_zoom = max;
        self
    }

    /// Configures the viewport to fit the given content bounds.
    ///
    /// This is useful when you want the viewport to show a specific area.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The rectangular area to fit
    /// * `screen_width` - Width of the screen/canvas
    /// * `screen_height` - Height of the screen/canvas
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    #[inline]
    pub fn fit_bounds(mut self, bounds: Rect, screen_width: f32, screen_height: f32) -> Self {
        self.content_bounds = Some(bounds);
        self.screen_size = Some((screen_width, screen_height));
        self
    }

    /// Sets the screen size for zoom calculations.
    ///
    /// # Arguments
    ///
    /// * `width` - Screen width
    /// * `height` - Screen height
    ///
    /// # Returns
    ///
    /// The builder for method chaining
    #[inline]
    pub fn with_screen_size(mut self, width: f32, height: f32) -> Self {
        self.screen_size = Some((width, height));
        self
    }

    /// Validates the builder configuration.
    ///
    /// This method performs all validations and returns a Result to enable
    /// early returns with the ? operator.
    ///
    /// # Returns
    ///
    /// Ok(()) if configuration is valid, or an error otherwise
    #[inline]
    fn validate(&self) -> Result<(), ViewportError> {
        if let Some(zoom) = self.zoom {
            if zoom <= 0.0 {
                return Err(ViewportError::InvalidZoom(zoom));
            }
        }

        if self.min_zoom <= 0.0 {
            return Err(ViewportError::InvalidZoom(self.min_zoom));
        }

        if self.max_zoom <= self.min_zoom {
            return Err(ViewportError::InvalidZoom(self.max_zoom));
        }

        if let Some((width, height)) = self.screen_size {
            if width <= 0.0 || height <= 0.0 {
                return Err(ViewportError::BoundsError("Screen size must be positive"));
            }
        }

        if let Some(bounds) = self.content_bounds {
            if bounds.width() <= 0.0 || bounds.height() <= 0.0 {
                return Err(ViewportError::BoundsError(
                    "Content bounds must have positive dimensions",
                ));
            }
        }

        Ok(())
    }

    /// Builds the Viewport instance.
    ///
    /// # Returns
    ///
    /// A configured Viewport instance
    ///
    /// # Errors
    ///
    /// Returns a [`ViewportError`] if configuration is invalid:
    /// - Zoom is not positive
    /// - Zoom limits are invalid
    /// - Screen size is not positive
    /// - Content bounds have non-positive dimensions
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use archflow_sdk::viewport::{ViewportBuilder, ViewportError};
    /// use archflow_core::{Rect, Vec2};
    ///
    /// let result = ViewportBuilder::new()
    ///     .with_offset(Vec2::new(100.0, 50.0))
    ///     .with_zoom(1.5)
    ///     .with_zoom_limits(0.05, 20.0)
    ///     .build();
    ///
    /// match result {
    ///     Ok(viewport) => println!("Created: {}", viewport),
    ///     Err(e) => eprintln!("Failed to create viewport: {}", e),
    /// }
    /// ```
    #[inline]
    pub fn build(self) -> Result<Viewport, ViewportError> {
        self.validate()?;

        // Handle fit_bounds priority
        if let (Some(bounds), Some((screen_width, screen_height))) =
            (self.content_bounds, self.screen_size)
        {
            return Ok(Viewport::fit_bounds(bounds, screen_width, screen_height));
        }

        // Handle screen_size for default zoom_to_fit behavior
        if let Some((screen_width, screen_height)) = self.screen_size {
            if self.offset.is_none() && self.zoom.is_none() {
                // Default to showing a reasonable default area
                let default_bounds = Rect::from_min_max(
                    Vec2::new(-screen_width / 2.0, -screen_height / 2.0),
                    Vec2::new(screen_width / 2.0, screen_height / 2.0),
                );
                return Ok(Viewport::fit_bounds(
                    default_bounds,
                    screen_width,
                    screen_height,
                ));
            }
        }

        // Build from explicit values
        let offset = self.offset.unwrap_or_default();
        let zoom = self.zoom.unwrap_or(1.0);

        // Clamp zoom to configured limits
        let zoom = zoom.clamp(self.min_zoom, self.max_zoom);

        Ok(Viewport {
            offset,
            zoom,
            min_zoom: self.min_zoom,
            max_zoom: self.max_zoom,
        })
    }
}

impl fmt::Display for ViewportBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ViewportBuilder(")?;
        if let Some(offset) = self.offset {
            write!(f, "offset=({}, {})", offset.x, offset.y)?;
        }
        if let Some(zoom) = self.zoom {
            if self.offset.is_some() {
                write!(f, ", ")?;
            }
            write!(f, "zoom={}", zoom)?;
        }
        write!(f, ", zoom_limits=[{}, {}])", self.min_zoom, self.max_zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_to_canvas() {
        let viewport = Viewport::new(Vec2::new(100.0, 50.0), 2.0);

        // Screen (0, 0) should map to canvas (-50, -25) because:
        // canvas = (screen - offset * zoom) / zoom
        // canvas = (0 - 100 * 2, 50 * 2) / 2
        // canvas = (-200, -100) / 2 = (-100, -50)
        let canvas = viewport.screen_to_canvas(Vec2::ZERO);
        assert!(
            (canvas.x - (-100.0)).abs() < 0.01,
            "x mismatch: {}",
            canvas.x
        );
        assert!(
            (canvas.y - (-50.0)).abs() < 0.01,
            "y mismatch: {}",
            canvas.y
        );
    }

    #[test]
    fn test_canvas_to_screen() {
        let viewport = Viewport::new(Vec2::new(100.0, 50.0), 2.0);

        // Canvas (0, 0) should map to screen (200, 100) because:
        // screen = canvas * zoom + offset * zoom
        // screen = 0 * 2 + 100 * 2 = 200
        let screen = viewport.canvas_to_screen(Vec2::ZERO);
        assert!((screen.x - 200.0).abs() < 0.01, "x mismatch: {}", screen.x);
        assert!((screen.y - 100.0).abs() < 0.01, "y mismatch: {}", screen.y);
    }

    #[test]
    fn test_zoom_at() {
        let viewport = Viewport::new(Vec2::ZERO, 1.0);
        let zoomed = viewport.zoom_at(Vec2::new(400.0, 300.0), 2.0);

        // After zooming by 2x at screen (400, 300):
        // The point at screen (400, 300) should remain at the same canvas position
        // new_offset = screen * (1/new_zoom - 1/old_zoom)
        // new_offset = (400, 300) * (0.5 - 1) = (400, 300) * (-0.5) = (-200, -150)
        assert!((zoomed.zoom - 2.0).abs() < 0.01);
        assert!(
            (zoomed.offset.x - (-200.0)).abs() < 0.01,
            "offset x: {}",
            zoomed.offset.x
        );
        assert!(
            (zoomed.offset.y - (-150.0)).abs() < 0.01,
            "offset y: {}",
            zoomed.offset.y
        );

        // Verify the point under the mouse stays in the same canvas position
        let canvas_before = viewport.screen_to_canvas(Vec2::new(400.0, 300.0));
        let canvas_after = zoomed.screen_to_canvas(Vec2::new(400.0, 300.0));
        assert!(
            (canvas_before.x - canvas_after.x).abs() < 0.01,
            "canvas x changed from {} to {}",
            canvas_before.x,
            canvas_after.x
        );
        assert!(
            (canvas_before.y - canvas_after.y).abs() < 0.01,
            "canvas y changed from {} to {}",
            canvas_before.y,
            canvas_after.y
        );
    }

    #[test]
    fn test_fit_bounds() {
        let bounds = Rect::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let viewport = Viewport::fit_bounds(bounds, 800.0, 600.0);

        // Bounds aspect = 1.0, screen aspect = 800/600 = 1.33
        // Since screen aspect > bounds aspect, use height
        // zoom = 600 / 100 * 0.9 = 5.4
        assert!((viewport.zoom - 5.4).abs() < 0.1);
    }

    // ViewportBuilder tests
    #[test]
    fn test_viewport_builder_default() {
        let builder = ViewportBuilder::new();
        let viewport = builder.build().unwrap();

        assert_eq!(viewport.offset, Vec2::ZERO);
        assert_eq!(viewport.zoom, 1.0);
        assert_eq!(viewport.min_zoom, 0.1);
        assert_eq!(viewport.max_zoom, 10.0);
    }

    #[test]
    fn test_viewport_builder_with_offset_and_zoom() {
        let viewport = ViewportBuilder::new()
            .with_offset(Vec2::new(100.0, 50.0))
            .with_zoom(1.5)
            .build()
            .unwrap();

        assert_eq!(viewport.offset, Vec2::new(100.0, 50.0));
        assert_eq!(viewport.zoom, 1.5);
    }

    #[test]
    fn test_viewport_builder_with_custom_zoom_limits() {
        let viewport = ViewportBuilder::new()
            .with_zoom(5.0)
            .with_zoom_limits(0.05, 20.0)
            .build()
            .unwrap();

        assert_eq!(viewport.min_zoom, 0.05);
        assert_eq!(viewport.max_zoom, 20.0);
    }

    #[test]
    fn test_viewport_builder_fit_bounds() {
        let bounds = Rect::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0));
        let viewport = ViewportBuilder::new()
            .fit_bounds(bounds, 800.0, 600.0)
            .build()
            .unwrap();

        // Should fit bounds into 800x600 screen with 10% padding
        assert!(viewport.zoom > 1.0);
        assert!(viewport.zoom < 10.0);
    }

    #[test]
    fn test_viewport_builder_invalid_zoom() {
        let result = ViewportBuilder::new().with_zoom(-1.0).build();

        assert!(matches!(result, Err(ViewportError::InvalidZoom(_))));
    }

    #[test]
    fn test_viewport_builder_invalid_zoom_limits() {
        let result = ViewportBuilder::new().with_zoom_limits(-0.1, 10.0).build();

        assert!(matches!(result, Err(ViewportError::InvalidZoom(_))));
    }

    #[test]
    fn test_viewport_builder_invalid_bounds() {
        let empty_bounds = Rect::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
        let result = ViewportBuilder::new()
            .fit_bounds(empty_bounds, 800.0, 600.0)
            .build();

        assert!(matches!(result, Err(ViewportError::BoundsError(_))));
    }

    #[test]
    fn test_viewport_builder_zoom_clamping() {
        // Zoom should be clamped to max_zoom
        let viewport = ViewportBuilder::new()
            .with_zoom(100.0)
            .with_zoom_limits(0.1, 10.0)
            .build()
            .unwrap();

        assert_eq!(viewport.zoom, 10.0);
    }

    #[test]
    fn test_viewport_builder_display() {
        let builder = ViewportBuilder::new()
            .with_offset(Vec2::new(100.0, 50.0))
            .with_zoom(1.5);

        let display = format!("{}", builder);
        assert!(display.contains("100"));
        assert!(display.contains("50"));
        assert!(display.contains("1.5"));
    }
}

/// Manages viewport state and provides viewport-related operations
#[derive(Debug)]
pub struct ViewportManager {
    viewport: Viewport,
    screen_width: f32,
    screen_height: f32,
    constrained_bounds: Option<Rect>,
}

impl ViewportManager {
    /// Creates a new viewport manager
    #[inline]
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            viewport: Viewport::default(),
            screen_width,
            screen_height,
            constrained_bounds: None,
        }
    }

    /// Sets the screen dimensions
    #[inline]
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Gets the current viewport
    #[inline]
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Sets the viewport directly
    #[inline]
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport.constrain(self.constrained_bounds);
    }

    /// Sets the viewport using a builder for flexible configuration
    ///
    /// # Arguments
    ///
    /// * `builder` - A configured ViewportBuilder
    ///
    /// # Errors
    ///
    /// Returns an error if the builder configuration is invalid
    #[inline]
    pub fn set_viewport_builder(&mut self, builder: ViewportBuilder) -> Result<(), ViewportError> {
        let viewport = builder
            .with_screen_size(self.screen_width, self.screen_height)
            .build()?;

        self.viewport = viewport.constrain(self.constrained_bounds);
        Ok(())
    }

    /// Pans the viewport by the given delta (in screen pixels)
    #[inline]
    pub fn pan(&mut self, delta: Vec2) {
        self.viewport = self.viewport.pan(delta).constrain(self.constrained_bounds);
    }

    /// Zooms at the given screen point
    #[inline]
    pub fn zoom_at(&mut self, screen_point: Vec2, factor: f32) {
        self.viewport = self.viewport.zoom_at(screen_point, factor);
    }

    /// Zooms in by the given factor
    #[inline]
    pub fn zoom_in(&mut self, factor: f32, center: Option<Vec2>) {
        let center =
            center.unwrap_or_else(|| Vec2::new(self.screen_width / 2.0, self.screen_height / 2.0));
        self.zoom_at(center, factor);
    }

    /// Zooms out by the given factor
    #[inline]
    pub fn zoom_out(&mut self, factor: f32, center: Option<Vec2>) {
        let center =
            center.unwrap_or_else(|| Vec2::new(self.screen_width / 2.0, self.screen_height / 2.0));
        self.zoom_at(center, 1.0 / factor);
    }

    /// Zooms to fit the given bounds
    #[inline]
    pub fn zoom_to_fit(&mut self, bounds: Rect) {
        self.viewport = Viewport::fit_bounds(bounds, self.screen_width, self.screen_height);
    }

    /// Zooms to fit all content in the canvas
    #[inline]
    pub fn zoom_to_content(&mut self, content_bounds: Rect) {
        if content_bounds.width() > 0.0 && content_bounds.height() > 0.0 {
            self.zoom_to_fit(content_bounds);
        }
    }

    /// Sets the constrained bounds for the viewport
    #[inline]
    pub fn set_constrained_bounds(&mut self, bounds: Option<Rect>) {
        self.constrained_bounds = bounds;
        self.viewport = self.viewport.constrain(self.constrained_bounds);
    }

    /// Gets the visible bounds in canvas coordinates
    #[inline]
    pub fn visible_bounds(&self) -> Rect {
        self.viewport
            .visible_bounds(self.screen_width, self.screen_height)
    }

    /// Checks if a point in canvas coordinates is visible
    #[inline]
    pub fn is_point_visible(&self, point: Vec2) -> bool {
        self.visible_bounds().contains(point)
    }

    /// Converts a screen coordinate to canvas coordinate
    #[inline]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        self.viewport.screen_to_canvas(screen)
    }

    /// Converts a canvas coordinate to screen coordinate
    #[inline]
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        self.viewport.canvas_to_screen(canvas)
    }
}
