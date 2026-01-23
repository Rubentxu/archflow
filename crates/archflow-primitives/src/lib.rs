//! ArchFlow Primitives - Primitivas geométricas del engine

mod connectivity;
mod drag_drop;
mod resize;
mod routing;
mod selection;
mod shapes;
mod styles;

#[cfg(test)]
mod selection_integration_tests;

pub use shapes::{
    Ellipse, Line, Polyline, Primitive, PrimitiveProperties, PrimitiveType, Rectangle,
};

pub use styles::{
    EffectStyle, FillPattern, FillStyle, LineCap, LineJoin, LineType, Shadow, ShapeStyle,
    StrokeStyle, Style, TextAlign, TextAlignY, TextStyle,
};

pub use connectivity::{
    Connection, ConnectionManager, ConnectionState, ConnectionType, Port, PortCollection,
    PortDirection, PortType,
};

pub use routing::{
    ConnectionRouter, ControlPointMode, CornerStyle, MarkerConfig, MarkerType, Obstacle,
    RouterConfig, RoutingPriority, RoutingResult, RoutingType,
};

pub use selection::{
    DragSelectionBox, DragSelectionConfig, HandleType, HitTestResult, SelectionConfig,
    SelectionEntry, SelectionManager, SelectionManagerBuilder, SelectionMode,
};

pub use drag_drop::{
    DragEvent, DragFeedbackConfig, DragManager, DragManagerBuilder, DragState, Draggable,
    SnapConfig, SnapGuideLine, SnapGuides,
};

pub use resize::{
    AspectRatioMode, Resizable, ResizeCenterMode, ResizeEvent, ResizeFeedbackConfig, ResizeManager,
    ResizeManagerBuilder, ResizeState, SizeConstraints,
};

use archflow_core::{EntityId, Rect, Vec2};

/// Contenedor de primitivas
pub struct PrimitiveContainer {
    primitives: Vec<Box<dyn Primitive>>,
}

impl PrimitiveContainer {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    pub fn add(&mut self, primitive: Box<dyn Primitive>) {
        self.primitives.push(primitive);
    }

    pub fn remove(&mut self, id: EntityId) -> Option<Box<dyn Primitive>> {
        self.primitives
            .iter()
            .position(|p| p.id() == id)
            .map(|i| self.primitives.remove(i))
    }

    pub fn get(&self, id: EntityId) -> Option<&dyn Primitive> {
        self.primitives
            .iter()
            .find(|p| p.id() == id)
            .map(|p| p.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Primitive> {
        self.primitives.iter().map(|p| p.as_ref())
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub fn global_bounds(&self) -> Rect {
        let mut result: Option<Rect> = None;
        for p in &self.primitives {
            let pb = p.global_bounds();
            result = Some(if let Some(b) = result {
                let min = Vec2::new(b.min.x.min(pb.min.x), b.min.y.min(pb.min.y));
                let max = Vec2::new(b.max.x.max(pb.max.x), b.max.y.max(pb.max.y));
                Rect::from_min_max(min, max)
            } else {
                pb
            });
        }
        result.unwrap_or_default()
    }
}
