//! Incremental Zoom - Level of Detail system for C4 model visualization
//!
//! Provides:
//! - Zoom levels (System, Container, Component)
//! - Level-based detail rendering
//! - Smooth transitions between levels
//! - Visibility rules per level

use crate::{EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Zoom level enumeration following C4 model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomLevel {
    /// Full system view (C4 Context) - 0 to 1000+ pixels
    System = 0,
    /// Container view (C4 Container) - 100 to 1000 pixels
    Container = 1,
    /// Component view (C4 Component) - 10 to 100 pixels
    Component = 2,
    /// Code view (C4 Code) - 1 to 10 pixels
    Code = 3,
}

impl Default for ZoomLevel {
    fn default() -> Self {
        Self::System
    }
}

impl ZoomLevel {
    /// Get pixel range for this level
    pub fn pixel_range(&self) -> (f32, f32) {
        match self {
            Self::System => (0.0, 100.0),
            Self::Container => (100.0, 500.0),
            Self::Component => (500.0, 1000.0),
            Self::Code => (1000.0, f32::MAX),
        }
    }

    /// Check if a scale falls within this level
    pub fn contains_scale(&self, scale: f32) -> bool {
        let (min, max) = self.pixel_range();
        scale >= min && scale < max
    }

    /// Get the zoom level for a given scale
    pub fn from_scale(scale: f32) -> Self {
        if scale >= 1000.0 {
            Self::Code
        } else if scale >= 500.0 {
            Self::Component
        } else if scale >= 100.0 {
            Self::Container
        } else {
            Self::System
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::System => "System Context",
            Self::Container => "Container",
            Self::Component => "Component",
            Self::Code => "Code",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::System => "Full system overview showing all users and systems",
            Self::Container => "Container-level view showing services and APIs",
            Self::Component => "Component-level view showing internal structure",
            Self::Code => "Code-level view showing implementation details",
        }
    }
}

/// Detail level for entities based on zoom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    /// Minimal representation (icon or dot)
    Minimal,
    /// Standard representation with label
    Standard,
    /// Full representation with all properties
    Full,
    /// Extended representation with internal details
    Extended,
}

impl DetailLevel {
    /// Get detail level for a zoom level
    pub fn for_zoom(zoom: ZoomLevel) -> Self {
        match zoom {
            ZoomLevel::System => Self::Minimal,
            ZoomLevel::Container => Self::Standard,
            ZoomLevel::Component => Self::Full,
            ZoomLevel::Code => Self::Extended,
        }
    }
}

/// Visibility rules for entities at different zoom levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityRules {
    /// Entity ID these rules apply to
    pub entity_id: EntityId,
    /// Minimum zoom level to show
    pub min_zoom: ZoomLevel,
    /// Maximum zoom level to show
    pub max_zoom: ZoomLevel,
    /// Whether to show the entity label
    pub show_label: bool,
    /// Minimum label size in pixels
    pub min_label_size: f32,
    /// Whether to show internal details
    pub show_details: bool,
    /// Whether to show connections
    pub show_connections: bool,
    /// Custom scale override (None = use default)
    pub custom_scale: Option<f32>,
}

impl VisibilityRules {
    /// Create default rules for an entity
    pub fn new(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            min_zoom: ZoomLevel::System,
            max_zoom: ZoomLevel::Code,
            show_label: true,
            min_label_size: 8.0,
            show_details: false,
            show_connections: true,
            custom_scale: None,
        }
    }

    /// Set minimum zoom level
    pub fn with_min_zoom(mut self, zoom: ZoomLevel) -> Self {
        self.min_zoom = zoom;
        self
    }

    /// Set maximum zoom level
    pub fn with_max_zoom(mut self, zoom: ZoomLevel) -> Self {
        self.max_zoom = zoom;
        self
    }

    /// Disable label
    pub fn without_label(mut self) -> Self {
        self.show_label = false;
        self
    }

    /// Enable internal details
    pub fn with_details(mut self) -> Self {
        self.show_details = true;
        self
    }

    /// Set custom scale
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.custom_scale = Some(scale);
        self
    }

    /// Check if entity is visible at given zoom
    pub fn is_visible(&self, zoom: ZoomLevel) -> bool {
        let zoom_index = zoom as u8;
        let min_index = self.min_zoom as u8;
        let max_index = self.max_zoom as u8;
        zoom_index >= min_index && zoom_index <= max_index
    }
}

/// Style override for a specific zoom level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomStyle {
    /// Zoom level this applies to
    pub zoom_level: ZoomLevel,
    /// Opacity override (None = use default)
    pub opacity: Option<f32>,
    /// Stroke width override
    pub stroke_width: Option<f32>,
    /// Fill color override (as hex string)
    pub fill_color: Option<String>,
    /// Show/hide outline
    pub show_outline: Option<bool>,
    /// Blur amount (for defocus effect)
    pub blur: Option<f32>,
}

impl ZoomStyle {
    pub fn new(zoom_level: ZoomLevel) -> Self {
        Self {
            zoom_level,
            opacity: None,
            stroke_width: None,
            fill_color: None,
            show_outline: None,
            blur: None,
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn with_fill_color(mut self, color: impl Into<String>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    pub fn blurred(mut self, amount: f32) -> Self {
        self.blur = Some(amount);
        self
    }
}

/// Progressive disclosure settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveDisclosure {
    /// Enable progressive disclosure
    pub enabled: bool,
    /// Steps to reveal full detail
    pub steps: u32,
    /// Delay between steps in milliseconds
    pub step_delay_ms: u64,
    /// Animation easing
    pub easing: String,
    /// Whether to animate transitions
    pub animate: bool,
}

impl Default for ProgressiveDisclosure {
    fn default() -> Self {
        Self {
            enabled: true,
            steps: 3,
            step_delay_ms: 100,
            easing: "ease-in-out".to_string(),
            animate: true,
        }
    }
}

/// Viewport with zoom information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZoomViewport {
    /// Viewport center in world coordinates
    pub center: Vec2,
    /// Current zoom scale (pixels per unit)
    pub scale: f32,
    /// Viewport width in pixels
    pub width: f32,
    /// Viewport height in pixels
    pub height: f32,
    /// Current zoom level
    pub zoom_level: ZoomLevel,
}

impl ZoomViewport {
    /// Create a new viewport
    pub fn new(center: Vec2, scale: f32, width: f32, height: f32) -> Self {
        let zoom_level = ZoomLevel::from_scale(scale);
        Self {
            center,
            scale,
            width,
            height,
            zoom_level,
        }
    }

    /// Get visible rectangle in world coordinates
    pub fn visible_rect(&self) -> Rect {
        let half_width = self.width / (2.0 * self.scale);
        let half_height = self.height / (2.0 * self.scale);
        Rect::from_min_max(
            Vec2::new(self.center.x - half_width, self.center.y - half_height),
            Vec2::new(self.center.x + half_width, self.center.y + half_height),
        )
    }

    /// Check if a point is visible
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.visible_rect().contains(point)
    }

    /// Zoom to fit a rectangle
    pub fn zoom_to_fit(&mut self, rect: Rect) {
        let center = rect.center();
        let scale_x = self.width / rect.width();
        let scale_y = self.height / rect.height();
        self.scale = scale_x.min(scale_y).min(1000.0).max(1.0);
        self.center = center;
        self.zoom_level = ZoomLevel::from_scale(self.scale);
    }

    /// Zoom in by factor
    pub fn zoom_in(&mut self, factor: f32) {
        self.scale = (self.scale * factor).min(2000.0);
        self.zoom_level = ZoomLevel::from_scale(self.scale);
    }

    /// Zoom out by factor
    pub fn zoom_out(&mut self, factor: f32) {
        self.scale = (self.scale / factor).max(0.5);
        self.zoom_level = ZoomLevel::from_scale(self.scale);
    }

    /// Pan by delta
    pub fn pan(&mut self, delta: Vec2) {
        self.center = self.center + delta;
    }

    /// Pan to center on point
    pub fn pan_to(&mut self, point: Vec2) {
        self.center = point;
    }
}

/// Zoom level transition animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZoomTransition {
    /// Instant jump
    None,
    /// Smooth interpolation
    Smooth,
    /// Elastic bounce effect
    Elastic,
    /// Zoom in with spin
    SpinIn,
    /// Zoom out with spin
    SpinOut,
}

/// Zoom manager for handling zoom state and transitions
#[derive(Debug, Clone)]
pub struct ZoomManager {
    /// Current viewport
    viewport: ZoomViewport,
    /// Target viewport for transitions
    target_viewport: Option<ZoomViewport>,
    /// Current transition
    transition: ZoomTransition,
    /// Transition progress (0.0 to 1.0)
    transition_progress: f32,
    /// Visibility rules per entity
    visibility_rules: HashMap<EntityId, VisibilityRules>,
    /// Style overrides per entity
    style_overrides: HashMap<EntityId, Vec<ZoomStyle>>,
    /// Progressive disclosure settings
    progressive_disclosure: ProgressiveDisclosure,
    /// Current detail level
    current_detail_level: DetailLevel,
    /// Previous zoom level (for transitions)
    previous_zoom_level: Option<ZoomLevel>,
}

impl Default for ZoomManager {
    fn default() -> Self {
        Self {
            viewport: ZoomViewport::new(Vec2::new(0.0, 0.0), 100.0, 800.0, 600.0),
            target_viewport: None,
            transition: ZoomTransition::Smooth,
            transition_progress: 1.0,
            visibility_rules: HashMap::new(),
            style_overrides: HashMap::new(),
            progressive_disclosure: ProgressiveDisclosure::default(),
            current_detail_level: DetailLevel::Standard,
            previous_zoom_level: None,
        }
    }
}

impl ZoomManager {
    /// Create a new zoom manager
    pub fn new(width: f32, height: f32) -> Self {
        let mut manager = Self::default();
        manager.viewport = ZoomViewport::new(Vec2::new(0.0, 0.0), 100.0, width, height);
        manager
    }

    /// Set viewport size
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.viewport.width = width;
        self.viewport.height = height;
    }

    /// Get current viewport
    pub fn viewport(&self) -> &ZoomViewport {
        &self.viewport
    }

    /// Get mutable viewport
    pub fn viewport_mut(&mut self) -> &mut ZoomViewport {
        &mut self.viewport
    }

    /// Set visibility rules for an entity
    pub fn set_visibility_rules(&mut self, rules: VisibilityRules) {
        self.visibility_rules.insert(rules.entity_id, rules);
    }

    /// Get visibility rules for an entity
    pub fn get_visibility_rules(&self, entity_id: EntityId) -> Option<&VisibilityRules> {
        self.visibility_rules.get(&entity_id)
    }

    /// Add style override for an entity
    pub fn add_style_override(&mut self, entity_id: EntityId, style: ZoomStyle) {
        self.style_overrides
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(style);
    }

    /// Get effective style for an entity at current zoom
    pub fn get_effective_style(&self, entity_id: EntityId) -> Option<&ZoomStyle> {
        self.style_overrides
            .get(&entity_id)?
            .iter()
            .find(|s| s.zoom_level == self.viewport.zoom_level)
    }

    /// Check if entity is visible at current zoom
    pub fn is_entity_visible(&self, entity_id: EntityId) -> bool {
        self.visibility_rules
            .get(&entity_id)
            .map(|rules| rules.is_visible(self.viewport.zoom_level))
            .unwrap_or(true)
    }

    /// Get effective detail level
    pub fn detail_level(&self) -> DetailLevel {
        if self.progressive_disclosure.enabled && self.transition_progress < 1.0 {
            // During transition, use interpolated detail
            let target = DetailLevel::for_zoom(self.viewport.zoom_level);
            let current = self.current_detail_level;
            if target as u8 > current as u8 {
                let step = (self.transition_progress * (target as u8 - current as u8) as f32) as u8;
                DetailLevel::from_u8(current as u8 + step)
            } else {
                target
            }
        } else {
            DetailLevel::for_zoom(self.viewport.zoom_level)
        }
    }

    /// Zoom to level
    pub fn zoom_to_level(&mut self, level: ZoomLevel) {
        self.previous_zoom_level = Some(self.viewport.zoom_level);
        let target_scale = match level {
            ZoomLevel::System => 50.0,
            ZoomLevel::Container => 200.0,
            ZoomLevel::Component => 700.0,
            ZoomLevel::Code => 1500.0,
        };
        self.target_viewport = Some(ZoomViewport::new(
            self.viewport.center,
            target_scale,
            self.viewport.width,
            self.viewport.height,
        ));
        self.transition_progress = 0.0;
    }

    /// Zoom in
    pub fn zoom_in(&mut self) {
        let levels = [
            ZoomLevel::System,
            ZoomLevel::Container,
            ZoomLevel::Component,
            ZoomLevel::Code,
        ];
        let current_idx = levels
            .iter()
            .position(|l| *l == self.viewport.zoom_level)
            .unwrap_or(0);
        if current_idx < levels.len() - 1 {
            self.zoom_to_level(levels[current_idx + 1]);
        }
    }

    /// Zoom out
    pub fn zoom_out(&mut self) {
        let levels = [
            ZoomLevel::System,
            ZoomLevel::Container,
            ZoomLevel::Component,
            ZoomLevel::Code,
        ];
        let current_idx = levels
            .iter()
            .position(|l| *l == self.viewport.zoom_level)
            .unwrap_or(levels.len() - 1);
        if current_idx > 0 {
            self.zoom_to_level(levels[current_idx - 1]);
        }
    }

    /// Update transition
    pub fn update(&mut self, delta_time: std::time::Duration) -> bool {
        if let Some(ref target) = self.target_viewport {
            self.transition_progress += delta_time.as_secs_f32() * 2.0; // 0.5s transition

            if self.transition_progress >= 1.0 {
                self.transition_progress = 1.0;
                self.viewport = *target;
                self.target_viewport = None;
                self.previous_zoom_level = None;
                self.current_detail_level = DetailLevel::for_zoom(self.viewport.zoom_level);
                return true; // Transition complete
            }

            // Interpolate viewport
            let t = self.ease(self.transition_progress);
            let source = if let Some(prev) = self.previous_zoom_level {
                Some(match prev {
                    ZoomLevel::System => 50.0,
                    ZoomLevel::Container => 200.0,
                    ZoomLevel::Component => 700.0,
                    ZoomLevel::Code => 1500.0,
                })
            } else {
                None
            };

            self.viewport.scale = if let Some(src) = source {
                src + (target.scale - src) * t
            } else {
                self.viewport.scale + (target.scale - self.viewport.scale) * t
            };
            self.viewport.zoom_level = ZoomLevel::from_scale(self.viewport.scale);
        }
        false
    }

    fn ease(&self, t: f32) -> f32 {
        match self.transition {
            ZoomTransition::None => t,
            ZoomTransition::Smooth => t * t * (3.0 - 2.0 * t),
            ZoomTransition::Elastic => {
                let p = 0.3;
                (2.0_f32).powf(-10.0 * t) * ((t - p / 4.0) * (2.0 * std::f32::consts::PI) / p).sin()
                    + 1.0
            }
            ZoomTransition::SpinIn => t * t,
            ZoomTransition::SpinOut => 1.0 - (1.0 - t) * (1.0 - t),
        }
    }

    /// Get current zoom level
    pub fn zoom_level(&self) -> ZoomLevel {
        self.viewport.zoom_level
    }

    /// Get current scale
    pub fn scale(&self) -> f32 {
        self.viewport.scale
    }

    /// Check if transitioning
    pub fn is_transitioning(&self) -> bool {
        self.target_viewport.is_some()
    }
}

impl DetailLevel {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Minimal,
            1 => Self::Standard,
            2 => Self::Full,
            _ => Self::Extended,
        }
    }
}

/// Export for use in other modules
pub use self::ZoomLevel as Level;
pub use self::ZoomManager as Zoom;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_level_from_scale() {
        assert_eq!(ZoomLevel::from_scale(50.0), ZoomLevel::System);
        assert_eq!(ZoomLevel::from_scale(200.0), ZoomLevel::Container);
        assert_eq!(ZoomLevel::from_scale(700.0), ZoomLevel::Component);
        assert_eq!(ZoomLevel::from_scale(1500.0), ZoomLevel::Code);
    }

    #[test]
    fn test_zoom_level_contains() {
        let system = ZoomLevel::System;
        assert!(system.contains_scale(50.0));
        assert!(!system.contains_scale(150.0));

        let container = ZoomLevel::Container;
        assert!(container.contains_scale(200.0));
        assert!(container.contains_scale(400.0));
    }

    #[test]
    fn test_visibility_rules() {
        let rules = VisibilityRules::new(EntityId::from_u128(1))
            .with_min_zoom(ZoomLevel::Container)
            .with_max_zoom(ZoomLevel::Component)
            .without_label();

        assert!(!rules.is_visible(ZoomLevel::System));
        assert!(rules.is_visible(ZoomLevel::Container));
        assert!(rules.is_visible(ZoomLevel::Component));
        assert!(!rules.is_visible(ZoomLevel::Code));
        assert!(!rules.show_label);
    }

    #[test]
    fn test_zoom_viewport() {
        let viewport = ZoomViewport::new(Vec2::new(100.0, 100.0), 100.0, 800.0, 600.0);

        assert_eq!(viewport.zoom_level, ZoomLevel::Container);
        let rect = viewport.visible_rect();
        assert!(rect.width() > 0.0);
        assert!(rect.height() > 0.0);
    }

    #[test]
    fn test_zoom_manager() {
        let mut manager = ZoomManager::new(800.0, 600.0);
        assert_eq!(manager.zoom_level(), ZoomLevel::Container);

        manager.zoom_to_level(ZoomLevel::System);
        assert!(manager.is_transitioning());

        manager.update(std::time::Duration::from_millis(600));
        assert!(!manager.is_transitioning());
        assert_eq!(manager.zoom_level(), ZoomLevel::System);
    }

    #[test]
    fn test_zoom_in_out() {
        let mut manager = ZoomManager::new(800.0, 600.0);
        assert_eq!(manager.zoom_level(), ZoomLevel::Container);

        manager.zoom_out();
        assert!(manager.is_transitioning()); // Transition started
                                             // Complete transition
        manager.update(std::time::Duration::from_millis(600));
        assert_eq!(manager.zoom_level(), ZoomLevel::System);

        manager.zoom_in();
        assert!(manager.is_transitioning());
        manager.update(std::time::Duration::from_millis(600));
        assert_eq!(manager.zoom_level(), ZoomLevel::Container);

        manager.zoom_in();
        manager.update(std::time::Duration::from_millis(600));
        assert_eq!(manager.zoom_level(), ZoomLevel::Component);

        manager.zoom_in();
        manager.update(std::time::Duration::from_millis(600));
        assert_eq!(manager.zoom_level(), ZoomLevel::Code);
    }

    #[test]
    fn test_detail_level_for_zoom() {
        assert_eq!(
            DetailLevel::for_zoom(ZoomLevel::System),
            DetailLevel::Minimal
        );
        assert_eq!(
            DetailLevel::for_zoom(ZoomLevel::Container),
            DetailLevel::Standard
        );
        assert_eq!(
            DetailLevel::for_zoom(ZoomLevel::Component),
            DetailLevel::Full
        );
        assert_eq!(
            DetailLevel::for_zoom(ZoomLevel::Code),
            DetailLevel::Extended
        );
    }

    #[test]
    fn test_zoom_style() {
        let style = ZoomStyle::new(ZoomLevel::Component)
            .with_opacity(0.5)
            .with_stroke_width(2.0)
            .with_fill_color("#FF0000")
            .blurred(1.0);

        assert_eq!(style.opacity, Some(0.5));
        assert_eq!(style.stroke_width, Some(2.0));
        assert_eq!(style.fill_color, Some("#FF0000".to_string()));
        assert_eq!(style.blur, Some(1.0));
    }
}
