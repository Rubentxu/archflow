//! ECS Renderable Component
//!
//! This module provides the Renderable component for ECS entities,
//! used by the rendering system to determine which entities should be rendered.

use bevy_ecs::prelude::*;

/// Renderable component for ECS entities.
///
/// Entities with this component will be rendered by the rendering system.
/// This allows for selective rendering and culling.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct RenderableEcs {
    /// Layer for z-ordering (lower = drawn first)
    pub layer: u32,

    /// Visibility flag
    pub visible: bool,

    /// Opacity value (0.0 = fully transparent, 1.0 = fully opaque)
    pub opacity: f32,
}

impl Default for RenderableEcs {
    fn default() -> Self {
        Self {
            layer: 0,
            visible: true,
            opacity: 1.0,
        }
    }
}

impl RenderableEcs {
    /// Creates a new Renderable component with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Renderable component with the specified layer.
    ///
    /// # Arguments
    ///
    /// * `layer` - Z-ordering layer (lower = drawn first)
    #[inline]
    pub fn with_layer(layer: u32) -> Self {
        Self {
            layer,
            ..Default::default()
        }
    }

    /// Creates a Renderable component with the specified visibility.
    ///
    /// # Arguments
    ///
    /// * `visible` - Whether the entity should be rendered
    #[inline]
    pub fn with_visibility(visible: bool) -> Self {
        Self {
            visible,
            ..Default::default()
        }
    }

    /// Creates a Renderable component with the specified opacity.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Opacity value (0.0 to 1.0)
    ///
    /// # Panics
    ///
    /// Panics if opacity is outside the range [0.0, 1.0]
    #[inline]
    pub fn with_opacity(opacity: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&opacity),
            "Opacity must be between 0.0 and 1.0, got {}",
            opacity
        );
        Self {
            opacity,
            ..Default::default()
        }
    }

    /// Sets the layer.
    #[inline]
    pub fn set_layer(&mut self, layer: u32) {
        self.layer = layer;
    }

    /// Sets the visibility.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Sets the opacity.
    ///
    /// # Panics
    ///
    /// Panics if opacity is outside the range [0.0, 1.0]
    #[inline]
    pub fn set_opacity(&mut self, opacity: f32) {
        assert!(
            (0.0..=1.0).contains(&opacity),
            "Opacity must be between 0.0 and 1.0, got {}",
            opacity
        );
        self.opacity = opacity;
    }

    /// Returns true if the entity should be rendered.
    #[inline]
    pub fn should_render(&self) -> bool {
        self.visible && self.opacity > 0.0
    }
}

/// Bundle for spawning entities with Renderable component.
#[derive(Bundle, Clone, Debug)]
pub struct RenderableBundle {
    /// Renderable component
    pub renderable: RenderableEcs,
}

impl Default for RenderableBundle {
    fn default() -> Self {
        Self {
            renderable: RenderableEcs::new(),
        }
    }
}

impl RenderableBundle {
    /// Creates a RenderableBundle with the specified layer.
    ///
    /// # Arguments
    ///
    /// * `layer` - Z-ordering layer
    #[inline]
    pub fn with_layer(layer: u32) -> Self {
        Self {
            renderable: RenderableEcs::with_layer(layer),
        }
    }

    /// Creates a RenderableBundle with the specified visibility.
    ///
    /// # Arguments
    ///
    /// * `visible` - Whether the entity should be rendered
    #[inline]
    pub fn with_visibility(visible: bool) -> Self {
        Self {
            renderable: RenderableEcs::with_visibility(visible),
        }
    }

    /// Creates a RenderableBundle with the specified opacity.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Opacity value (0.0 to 1.0)
    #[inline]
    pub fn with_opacity(opacity: f32) -> Self {
        Self {
            renderable: RenderableEcs::with_opacity(opacity),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderable_default() {
        let renderable = RenderableEcs::default();
        assert_eq!(renderable.layer, 0);
        assert!(renderable.visible);
        assert_eq!(renderable.opacity, 1.0);
    }

    #[test]
    fn test_renderable_new() {
        let renderable = RenderableEcs::new();
        assert_eq!(renderable, RenderableEcs::default());
    }

    #[test]
    fn test_renderable_with_layer() {
        let renderable = RenderableEcs::with_layer(5);
        assert_eq!(renderable.layer, 5);
        assert!(renderable.visible);
        assert_eq!(renderable.opacity, 1.0);
    }

    #[test]
    fn test_renderable_with_visibility() {
        let renderable = RenderableEcs::with_visibility(false);
        assert_eq!(renderable.layer, 0);
        assert!(!renderable.visible);
        assert_eq!(renderable.opacity, 1.0);
    }

    #[test]
    fn test_renderable_with_opacity() {
        let renderable = RenderableEcs::with_opacity(0.5);
        assert_eq!(renderable.layer, 0);
        assert!(renderable.visible);
        assert_eq!(renderable.opacity, 0.5);
    }

    #[test]
    #[should_panic(expected = "Opacity must be between 0.0 and 1.0")]
    fn test_renderable_invalid_opacity_low() {
        RenderableEcs::with_opacity(-0.1);
    }

    #[test]
    #[should_panic(expected = "Opacity must be between 0.0 and 1.0")]
    fn test_renderable_invalid_opacity_high() {
        RenderableEcs::with_opacity(1.1);
    }

    #[test]
    fn test_renderable_set_layer() {
        let mut renderable = RenderableEcs::default();
        renderable.set_layer(10);
        assert_eq!(renderable.layer, 10);
    }

    #[test]
    fn test_renderable_set_visible() {
        let mut renderable = RenderableEcs::default();
        renderable.set_visible(false);
        assert!(!renderable.visible);
    }

    #[test]
    fn test_renderable_set_opacity() {
        let mut renderable = RenderableEcs::default();
        renderable.set_opacity(0.75);
        assert_eq!(renderable.opacity, 0.75);
    }

    #[test]
    fn test_renderable_should_render() {
        let renderable = RenderableEcs::default();
        assert!(renderable.should_render());
    }

    #[test]
    fn test_renderable_should_not_render_invisible() {
        let renderable = RenderableEcs::with_visibility(false);
        assert!(!renderable.should_render());
    }

    #[test]
    fn test_renderable_should_not_render_zero_opacity() {
        let renderable = RenderableEcs::with_opacity(0.0);
        assert!(!renderable.should_render());
    }

    #[test]
    fn test_renderable_bundle_default() {
        let bundle = RenderableBundle::default();
        assert_eq!(bundle.renderable, RenderableEcs::default());
    }

    #[test]
    fn test_renderable_bundle_with_layer() {
        let bundle = RenderableBundle::with_layer(3);
        assert_eq!(bundle.renderable.layer, 3);
    }

    #[test]
    fn test_renderable_bundle_with_visibility() {
        let bundle = RenderableBundle::with_visibility(true);
        assert!(bundle.renderable.visible);
    }

    #[test]
    fn test_renderable_bundle_with_opacity() {
        let bundle = RenderableBundle::with_opacity(0.8);
        assert_eq!(bundle.renderable.opacity, 0.8);
    }

    #[test]
    fn test_renderable_clone() {
        let renderable = RenderableEcs {
            layer: 5,
            visible: false,
            opacity: 0.5,
        };
        let cloned = renderable.clone();

        assert_eq!(cloned.layer, 5);
        assert!(!cloned.visible);
        assert_eq!(cloned.opacity, 0.5);
    }

    #[test]
    fn test_renderable_equality() {
        let r1 = RenderableEcs::default();
        let r2 = RenderableEcs::default();
        let r3 = RenderableEcs::with_layer(1);

        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }
}
