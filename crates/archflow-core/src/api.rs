//! High-Level Developer APIs - Simplified interface for common operations

use crate::animation::{AnimationManager, EasingFunction, FloatAnimation, PositionAnimation};
use crate::{Color, EntityId, Transform, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CanvasBuilder {
    width: f32,
    height: f32,
    background_color: Option<Color>,
    antialias: bool,
    pixel_ratio: f32,
}

impl CanvasBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }
    pub fn antialias(mut self, enabled: bool) -> Self {
        self.antialias = enabled;
        self
    }
    pub fn pixel_ratio(mut self, ratio: f32) -> Self {
        self.pixel_ratio = ratio;
        self
    }
    pub fn build(self) -> CanvasConfig {
        CanvasConfig {
            width: self.width,
            height: self.height,
            background_color: self.background_color.unwrap_or(Color::WHITE),
            antialias: self.antialias,
            pixel_ratio: self.pixel_ratio,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasConfig {
    pub width: f32,
    pub height: f32,
    pub background_color: Color,
    pub antialias: bool,
    pub pixel_ratio: f32,
}

/// Simple position for shapes (lightweight, no dependency on primitives crate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapePosition {
    pub x: f32,
    pub y: f32,
}

impl ShapePosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

/// Simple size for shapes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShapeSize {
    pub width: f32,
    pub height: f32,
}

impl ShapeSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Simple bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShapeBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ShapeBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Factory for creating shapes with fluent API
#[derive(Debug, Default)]
pub struct ShapeFactory;

impl ShapeFactory {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn create_position(&self, x: f32, y: f32) -> ShapePosition {
        ShapePosition::new(x, y)
    }
    pub fn create_size(&self, width: f32, height: f32) -> ShapeSize {
        ShapeSize::new(width, height)
    }
    pub fn create_bounds(&self, x: f32, y: f32, width: f32, height: f32) -> ShapeBounds {
        ShapeBounds::new(x, y, width, height)
    }
    pub fn create_centered_bounds(
        &self,
        center_x: f32,
        center_y: f32,
        width: f32,
        height: f32,
    ) -> ShapeBounds {
        ShapeBounds::new(
            center_x - width / 2.0,
            center_y - height / 2.0,
            width,
            height,
        )
    }
}

/// Scene for managing document state (simplified, using core types only)
#[derive(Debug, Default)]
pub struct Scene {
    entities: HashMap<EntityId, ShapeData>,
    transform: Transform,
    animation_manager: AnimationManager,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_shape(&mut self, shape: ShapeData) -> EntityId {
        let id = shape.id;
        self.entities.insert(id, shape);
        id
    }
    pub fn add_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> EntityId {
        self.add_shape(ShapeData::rectangle(x, y, width, height))
    }
    pub fn add_ellipse(&mut self, x: f32, y: f32, radius_x: f32, radius_y: f32) -> EntityId {
        self.add_shape(ShapeData::ellipse(x, y, radius_x, radius_y))
    }
    pub fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> EntityId {
        self.add_shape(ShapeData::line(x1, y1, x2, y2))
    }
    pub fn remove_shape(&mut self, id: EntityId) -> bool {
        self.entities.remove(&id).is_some()
    }
    pub fn get_shape(&self, id: EntityId) -> Option<&ShapeData> {
        self.entities.get(&id)
    }
    pub fn get_shape_mut(&mut self, id: EntityId) -> Option<&mut ShapeData> {
        self.entities.get_mut(&id)
    }
    pub fn all_ids(&self) -> Vec<EntityId> {
        self.entities.keys().cloned().collect()
    }
    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains_key(&id)
    }
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
    pub fn transform(&self) -> &Transform {
        &self.transform
    }
    pub fn animation_manager(&mut self) -> &mut AnimationManager {
        &mut self.animation_manager
    }
}

/// Simple shape data for scene management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeData {
    pub id: EntityId,
    pub name: Option<String>,
    pub shape_type: ShapeType,
    pub position: ShapePosition,
    pub size: Option<ShapeSize>,
    pub color: Option<Color>,
    pub opacity: f32,
    pub visible: bool,
    pub layer: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Polyline,
    Custom(String),
}

impl ShapeData {
    pub fn rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: EntityId::new(),
            name: None,
            shape_type: ShapeType::Rectangle,
            position: ShapePosition::new(x, y),
            size: Some(ShapeSize::new(width, height)),
            color: None,
            opacity: 1.0,
            visible: true,
            layer: 0,
        }
    }
    pub fn ellipse(x: f32, y: f32, radius_x: f32, radius_y: f32) -> Self {
        Self {
            id: EntityId::new(),
            name: None,
            shape_type: ShapeType::Ellipse,
            position: ShapePosition::new(x, y),
            size: Some(ShapeSize::new(radius_x * 2.0, radius_y * 2.0)),
            color: None,
            opacity: 1.0,
            visible: true,
            layer: 0,
        }
    }
    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            id: EntityId::new(),
            name: None,
            shape_type: ShapeType::Line,
            position: ShapePosition::new(x1, y1),
            size: Some(ShapeSize::new(x2 - x1, y2 - y1)),
            color: None,
            opacity: 1.0,
            visible: true,
            layer: 0,
        }
    }
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
    pub fn bounds(&self) -> ShapeBounds {
        ShapeBounds::new(
            self.position.x,
            self.position.y,
            self.size.map(|s| s.width).unwrap_or(0.0),
            self.size.map(|s| s.height).unwrap_or(0.0),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApiConfig {
    pub debug: bool,
    pub performance: bool,
    pub default_width: f32,
    pub default_height: f32,
    pub grid_enabled: bool,
    pub grid_size: f32,
    pub grid_color: Color,
    pub snap_enabled: bool,
    pub snap_threshold: f32,
}

impl ApiConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }
    pub fn with_performance(mut self, enabled: bool) -> Self {
        self.performance = enabled;
        self
    }
    pub fn with_grid(mut self, size: f32, color: Color) -> Self {
        self.grid_enabled = true;
        self.grid_size = size;
        self.grid_color = color;
        self
    }
    pub fn with_snap(mut self, threshold: f32) -> Self {
        self.snap_enabled = true;
        self.snap_threshold = threshold;
        self
    }
}

#[derive(Debug, Default)]
pub struct AnimationHelper;

impl AnimationHelper {
    pub fn animate_position(
        from: (f32, f32),
        to: (f32, f32),
        duration_ms: u64,
    ) -> PositionAnimation {
        PositionAnimation::new(
            EntityId::new(),
            vec![
                crate::animation::PositionKeyframe::new(0.0, from, EasingFunction::EaseInOut),
                crate::animation::PositionKeyframe::new(1.0, to, EasingFunction::EaseInOut),
            ],
        )
        .with_config(crate::AnimationConfig {
            duration: std::time::Duration::from_millis(duration_ms),
            ..Default::default()
        })
    }
    pub fn animate_opacity(from: f32, to: f32, duration_ms: u64) -> FloatAnimation {
        FloatAnimation::new(
            EntityId::new(),
            crate::AnimatedProperty::Opacity,
            vec![
                crate::animation::FloatKeyframe::new(0.0, from, EasingFunction::EaseInOut),
                crate::animation::FloatKeyframe::new(1.0, to, EasingFunction::EaseInOut),
            ],
        )
        .with_config(crate::AnimationConfig {
            duration: std::time::Duration::from_millis(duration_ms),
            ..Default::default()
        })
    }
    pub fn animate_scale(from: f32, to: f32, duration_ms: u64) -> FloatAnimation {
        FloatAnimation::new(
            EntityId::new(),
            crate::AnimatedProperty::Scale,
            vec![
                crate::animation::FloatKeyframe::new(0.0, from, EasingFunction::Elastic),
                crate::animation::FloatKeyframe::new(1.0, to, EasingFunction::Elastic),
            ],
        )
        .with_config(crate::AnimationConfig {
            duration: std::time::Duration::from_millis(duration_ms),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub divider: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: Color::rgb(0.23, 0.51, 0.89),
            secondary: Color::rgb(0.61, 0.15, 0.69),
            accent: Color::rgb(0.95, 0.61, 0.07),
            background: Color::rgb(0.98, 0.98, 0.98),
            surface: Color::rgb(1.0, 1.0, 1.0),
            error: Color::rgb(0.91, 0.12, 0.39),
            warning: Color::rgb(1.0, 0.76, 0.03),
            success: Color::rgb(0.10, 0.74, 0.61),
            text: Color::rgb(0.13, 0.13, 0.13),
            text_secondary: Color::rgb(0.46, 0.46, 0.46),
            border: Color::rgb(0.82, 0.82, 0.82),
            divider: Color::rgb(0.92, 0.92, 0.92),
        }
    }
}

#[derive(Debug, Default)]
pub struct SnapHelper {
    enabled: bool,
    threshold: f32,
    grid_size: f32,
}

impl SnapHelper {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }
    pub fn with_grid_size(mut self, size: f32) -> Self {
        self.grid_size = size;
        self
    }
    pub fn grid_size(&self) -> f32 {
        self.grid_size
    }
    pub fn snap_to_grid(&self, point: Vec2) -> Vec2 {
        if self.grid_size <= 0.0 {
            return point;
        }
        Vec2::new(
            (point.x / self.grid_size).round() * self.grid_size,
            (point.y / self.grid_size).round() * self.grid_size,
        )
    }
    pub fn snap_to_axis(&self, point: Vec2, reference: Vec2) -> Vec2 {
        let dx = (point.x - reference.x).abs();
        let dy = (point.y - reference.y).abs();
        if dx < self.threshold && dx < dy {
            Vec2::new(reference.x, point.y)
        } else if dy < self.threshold {
            Vec2::new(point.x, reference.y)
        } else {
            point
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_builder() {
        let config = CanvasBuilder::new()
            .size(800.0, 600.0)
            .background_color(Color::rgb(0.5, 0.5, 0.5))
            .pixel_ratio(2.0)
            .build();
        assert_eq!(config.width, 800.0);
        assert_eq!(config.height, 600.0);
    }

    #[test]
    fn test_shape_factory() {
        let factory = ShapeFactory::new();
        let pos = factory.create_position(10.0, 20.0);
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_scene() {
        let mut scene = Scene::new();
        assert!(scene.is_empty());
        let id = scene.add_rectangle(0.0, 0.0, 100.0, 50.0);
        assert_eq!(scene.len(), 1);
        assert!(scene.contains(id));
        assert!(scene.get_shape(id).is_some());
        scene.remove_shape(id);
        assert!(scene.is_empty());
    }

    #[test]
    fn test_shape_data() {
        let shape = ShapeData::rectangle(10.0, 20.0, 100.0, 50.0)
            .with_color(Color::RED)
            .with_name("my_rect");
        assert_eq!(shape.shape_type, ShapeType::Rectangle);
        assert!(shape.color.is_some());
        assert_eq!(shape.name, Some("my_rect".to_string()));
    }

    #[test]
    fn test_api_config() {
        let config = ApiConfig::new()
            .with_debug(true)
            .with_performance(true)
            .with_grid(10.0, Color::rgb(0.5, 0.5, 0.5))
            .with_snap(5.0);
        assert!(config.debug);
        assert!(config.performance);
        assert!(config.grid_enabled);
        assert!(config.snap_enabled);
    }

    #[test]
    fn test_animation_helper() {
        let anim = AnimationHelper::animate_position((0.0, 0.0), (100.0, 100.0), 500);
        assert_eq!(anim.keyframes.len(), 2);
    }

    #[test]
    fn test_color_palette() {
        let palette = ColorPalette::default();
        assert!(palette.primary != palette.secondary);
    }

    #[test]
    fn test_snap_helper() {
        let snap = SnapHelper::new()
            .enable()
            .with_grid_size(10.0)
            .with_threshold(5.0);
        let snapped = snap.snap_to_grid(Vec2::new(13.0, 27.0));
        assert!((snapped.x - 10.0).abs() < 0.01 || (snapped.x - 20.0).abs() < 0.01);
    }
}
