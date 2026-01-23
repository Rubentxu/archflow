//! Selection Renderer - Visual feedback para selección
//!
//! Proporciona renderizado de:
//! - Selection highlights
//! - Transform handles (corners, edges)
//! - Drag selection box
//! - Bounding boxes

use crate::{Renderer, StrokeStyle};
use archflow_core::{Color, Rect, Vec2};
use archflow_primitives::{DragSelectionBox, DragSelectionConfig, HandleType, SelectionConfig};

/// Renderizador de elementos de selección
pub struct SelectionRenderer<R: Renderer> {
    /// Renderer base
    renderer: R,
    /// Configuración de selección
    selection_config: SelectionConfig,
    /// Configuración del drag selection box
    drag_config: DragSelectionConfig,
}

impl<R: Renderer> SelectionRenderer<R> {
    /// Crear nuevo SelectionRenderer
    pub fn new(renderer: R) -> Self {
        Self {
            renderer,
            selection_config: SelectionConfig::default(),
            drag_config: DragSelectionConfig::default(),
        }
    }

    /// Crear con configuración personalizada
    pub fn with_config(
        renderer: R,
        selection_config: SelectionConfig,
        drag_config: DragSelectionConfig,
    ) -> Self {
        Self {
            renderer,
            selection_config,
            drag_config,
        }
    }

    /// Actualizar configuración de selección
    pub fn set_selection_config(&mut self, config: SelectionConfig) {
        self.selection_config = config;
    }

    /// Actualizar configuración de drag
    pub fn set_drag_config(&mut self, config: DragSelectionConfig) {
        self.drag_config = config;
    }

    /// Renderizar highlight de selección para un rectángulo
    pub fn draw_selection_highlight(&mut self, bounds: Rect) {
        if !self.selection_config.show_bounding_box {
            return;
        }

        let color = Color::rgba(
            self.selection_config.highlight_color[0],
            self.selection_config.highlight_color[1],
            self.selection_config.highlight_color[2],
            self.selection_config.highlight_color[3],
        );

        self.renderer.save();
        self.renderer.translate(bounds.min.x, bounds.min.y);

        // Borde del bounding box
        let width = bounds.width();
        let height = bounds.height();

        let stroke_style = StrokeStyle {
            color,
            width: self.selection_config.highlight_width,
            line_cap: crate::LineCap::Square,
            line_join: crate::LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        };

        // Draw bounding box rectangle
        self.renderer.draw_rect(0.0, 0.0, width, height);

        self.renderer.restore();
    }

    /// Renderizar handles de transformación para un rectángulo
    pub fn draw_transform_handles(&mut self, bounds: Rect) {
        if !self.selection_config.show_transform_handles {
            return;
        }

        let handle_size = self.selection_config.handle_size;
        let half_size = handle_size / 2.0;

        let handle_color = Color::rgba(
            self.selection_config.handle_color[0],
            self.selection_config.handle_color[1],
            self.selection_config.handle_color[2],
            self.selection_config.handle_color[3],
        );

        self.renderer.save();

        // Draw each handle at its calculated position
        for handle_type in HandleType::bounding_box_handles() {
            let pos = handle_type.position(bounds);

            let x = pos.x - half_size;
            let y = pos.y - half_size;

            // Fill handle
            let fill_color = Color::rgba(
                self.selection_config.handle_color[0],
                self.selection_config.handle_color[1],
                self.selection_config.handle_color[2],
                self.selection_config.handle_color[3] * 0.9, // Slightly darker when filled
            );

            // Draw filled handle rectangle
            self.renderer.draw_rect(x, y, handle_size, handle_size);

            // Stroke handle
            let stroke_style = StrokeStyle {
                color: handle_color,
                width: 1.0,
                line_cap: crate::LineCap::Butt,
                line_join: crate::LineJoin::Miter,
                dash_pattern: None,
                miter_limit: 4.0,
            };
        }

        self.renderer.restore();
    }

    /// Renderizar un handle individual
    pub fn draw_handle(&mut self, position: Vec2, handle_type: HandleType) {
        let handle_size = self.selection_config.handle_size;
        let half_size = handle_size / 2.0;

        let x = position.x - half_size;
        let y = position.y - half_size;

        self.renderer.save();
        self.renderer.draw_rect(x, y, handle_size, handle_size);
        self.renderer.restore();
    }

    /// Renderizar drag selection box
    pub fn draw_drag_selection_box(&mut self, box_selection: DragSelectionBox) {
        if !box_selection.is_active || !box_selection.has_area() {
            return;
        }

        let rect = box_selection.rect();

        self.renderer.save();

        // Determinar color según modo add
        let (border_color, fill_color) = if box_selection.add_to_selection {
            let add_color = Color::rgba(
                0.2, 0.8, 0.3, 1.0, // Verde para add
            );
            let add_fill = Color::rgba(
                0.2, 0.8, 0.3, 0.1, // Verde semi-transparente
            );
            (add_color, add_fill)
        } else {
            let border = Color::rgba(
                self.drag_config.border_color[0],
                self.drag_config.border_color[1],
                self.drag_config.border_color[2],
                self.drag_config.border_color[3],
            );
            let fill = Color::rgba(
                self.drag_config.fill_color[0],
                self.drag_config.fill_color[1],
                self.drag_config.fill_color[2],
                self.drag_config.fill_color[3],
            );
            (border, fill)
        };

        let width = rect.width();
        let height = rect.height();

        // Fill rectangle
        let fill_style = StrokeStyle {
            color: fill_color,
            width: self.drag_config.border_width,
            line_cap: crate::LineCap::Butt,
            line_join: crate::LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        };
        self.renderer
            .draw_rect(rect.min.x, rect.min.y, width, height);

        // Stroke rectangle
        let stroke_style = StrokeStyle {
            color: border_color,
            width: self.drag_config.border_width,
            line_cap: crate::LineCap::Square,
            line_join: crate::LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        };
        self.renderer
            .draw_rect(rect.min.x, rect.min.y, width, height);

        self.renderer.restore();
    }

    /// Renderizar elementos de selección completos (highlight + handles)
    pub fn draw_selection(&mut self, bounds: Rect) {
        self.draw_selection_highlight(bounds);
        self.draw_transform_handles(bounds);
    }

    /// Renderizar hit test result highlight
    pub fn draw_hit_test_highlight(&mut self, bounds: Rect, is_selected: bool) {
        if is_selected {
            self.draw_selection_highlight(bounds);
        } else {
            // Draw subtle hover highlight
            let hover_color = Color::rgba(0.5, 0.5, 0.5, 0.3);
            self.renderer.save();
            self.renderer
                .draw_rect(bounds.min.x, bounds.min.y, bounds.width(), bounds.height());
            self.renderer.restore();
        }
    }
}

impl<R: Renderer> AsMut<R> for SelectionRenderer<R> {
    fn as_mut(&mut self) -> &mut R {
        &mut self.renderer
    }
}

impl<R: Renderer> AsRef<R> for SelectionRenderer<R> {
    fn as_ref(&self) -> &R {
        &self.renderer
    }
}
