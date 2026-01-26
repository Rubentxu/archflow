//! ArchFlow Selection System - Sistema de selección de elementos
//!
//! Proporciona:
//! - SelectionConfig: Configuración de visualización de selección
//! - DragSelectionConfig: Configuración de drag selection box
//! - DragSelectionBox: Estado del drag selection

use archflow_core::{Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Configuración de visualización de selección
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionConfig {
    /// Mostrar bounding box de selección
    pub show_bounding_box: bool,
    /// Mostrar handles de transformación
    pub show_transform_handles: bool,
    /// Tamaño de los handles
    pub handle_size: f32,
    /// Color de los handles [r, g, b, a]
    pub handle_color: [f32; 4],
    /// Color del highlight [r, g, b, a]
    pub highlight_color: [f32; 4],
    /// Ancho de línea del highlight
    pub highlight_width: f32,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            show_bounding_box: true,
            show_transform_handles: true,
            handle_size: 8.0,
            handle_color: [0.0, 0.5, 1.0, 1.0],
            highlight_color: [0.0, 0.5, 1.0, 0.3],
            highlight_width: 1.0,
        }
    }
}

/// Configuración del drag selection box
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragSelectionConfig {
    /// Color del borde [r, g, b, a]
    pub border_color: [f32; 4],
    /// Color de relleno [r, g, b, a]
    pub fill_color: [f32; 4],
    /// Ancho del borde
    pub border_width: f32,
}

impl Default for DragSelectionConfig {
    fn default() -> Self {
        Self {
            border_color: [0.0, 0.5, 1.0, 1.0],
            fill_color: [0.0, 0.5, 1.0, 0.1],
            border_width: 1.0,
        }
    }
}

/// Estado del drag selection box
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragSelectionBox {
    /// Indica si el drag selection está activo
    pub is_active: bool,
    /// Punto inicial del drag
    pub start_point: Option<(f32, f32)>,
    /// Punto actual del drag
    pub current_point: Option<(f32, f32)>,
    /// Si está en modo añadir a selección existente
    pub add_to_selection: bool,
}

impl DragSelectionBox {
    /// Crear un nuevo DragSelectionBox
    pub fn new() -> Self {
        Self {
            is_active: false,
            start_point: None,
            current_point: None,
            add_to_selection: false,
        }
    }

    /// Iniciar drag selection
    pub fn start(&mut self, x: f32, y: f32, add_to_selection: bool) {
        self.is_active = true;
        self.start_point = Some((x, y));
        self.current_point = Some((x, y));
        self.add_to_selection = add_to_selection;
    }

    /// Actualizar posición actual del drag
    pub fn update(&mut self, x: f32, y: f32) {
        if self.is_active {
            self.current_point = Some((x, y));
        }
    }

    /// Finalizar drag selection
    pub fn end(&mut self) {
        self.is_active = false;
        self.start_point = None;
        self.current_point = None;
    }

    /// Obtener el rectángulo del drag selection
    pub fn rect(&self) -> Rect {
        match (self.start_point, self.current_point) {
            (Some((x1, y1)), Some((x2, y2))) => {
                let min = Vec2::new(x1.min(x2), y1.min(y2));
                let max = Vec2::new(x1.max(x2), y1.max(y2));
                Rect::from_min_max(min, max)
            }
            _ => Rect::default(),
        }
    }

    /// Verificar si tiene área (no es un punto)
    pub fn has_area(&self) -> bool {
        match (self.start_point, self.current_point) {
            (Some((x1, y1)), Some((x2, y2))) => (x1 - x2).abs() > 0.0 || (y1 - y2).abs() > 0.0,
            _ => false,
        }
    }

    /// Obtener punto de inicio
    pub fn start_point(&self) -> Option<(f32, f32)> {
        self.start_point
    }

    /// Obtener punto actual
    pub fn current_point(&self) -> Option<(f32, f32)> {
        self.current_point
    }
}

impl Default for DragSelectionBox {
    fn default() -> Self {
        Self::new()
    }
}
