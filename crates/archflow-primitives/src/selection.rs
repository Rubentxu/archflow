//! ArchFlow Selection System - Sistema de selección de primitivas
//!
//! Proporciona gestión completa de selección con:
//! - Selección simple y múltiple
//! - Selección por área (drag selection)
//! - Visual feedback configurable
//! - Hit testing optimizado

use crate::Primitive;
use archflow_core::{EntityId, Rect, Uuid, Vec2};
use archflow_geometry::{HitTestConfig, IntersectionEngine};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Modo de selección
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Solo una primitiva puede estar seleccionada
    Single,
    /// Múltiples primitivas pueden estar seleccionadas
    Multiple,
    /// Modo de rango (para selección por área)
    Range,
}

/// Estado de una primitiva seleccionada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionEntry {
    /// ID de la primitiva
    pub id: EntityId,
    /// Orden de selección (para mantener consistencia)
    pub order: u64,
    /// Timestamp de selección
    pub selected_at: std::time::SystemTime,
}

impl SelectionEntry {
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            order: Uuid::new_v4().as_u128() as u64,
            selected_at: std::time::SystemTime::now(),
        }
    }
}

/// Configuración de selección
#[derive(Debug, Clone)]
pub struct SelectionConfig {
    /// Color del highlight de selección
    pub highlight_color: [f32; 4],
    /// Ancho del borde de selección
    pub highlight_width: f32,
    /// Color del handle de resize
    pub handle_color: [f32; 4],
    /// Tamaño del handle
    pub handle_size: f32,
    /// Habilitar handles de transformación
    pub show_transform_handles: bool,
    /// Habilitar bounding box
    pub show_bounding_box: bool,
    /// Usar animación al seleccionar
    pub enable_animation: bool,
    /// Duración de la animación (ms)
    pub animation_duration_ms: u32,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            highlight_color: [0.2, 0.6, 1.0, 1.0], // Azul
            highlight_width: 2.0,
            handle_color: [1.0, 1.0, 1.0, 1.0], // Blanco
            handle_size: 8.0,
            show_transform_handles: true,
            show_bounding_box: true,
            enable_animation: false,
            animation_duration_ms: 200,
        }
    }
}

/// Estado del rectángulo de selección por arrastre (drag selection)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragSelectionBox {
    /// Punto inicial del drag (donde comenzó el arrastre)
    pub start_point: Vec2,
    /// Punto actual del drag (donde está el cursor)
    pub current_point: Vec2,
    /// Indica si el drag está activo
    pub is_active: bool,
    /// Indica si el rectángulo debe mostrarse en modo "add" (Shift)
    pub add_to_selection: bool,
}

impl DragSelectionBox {
    /// Crear un nuevo rectángulo de selección
    pub fn new(start_point: Vec2) -> Self {
        Self {
            start_point,
            current_point: start_point,
            is_active: true,
            add_to_selection: false,
        }
    }

    /// Actualizar el punto actual
    pub fn update(&mut self, current_point: Vec2) {
        self.current_point = current_point;
    }

    /// Finalizar el drag
    pub fn end(&mut self) {
        self.is_active = false;
    }

    /// Reiniciar el drag desde un nuevo punto
    pub fn restart(&mut self, start_point: Vec2) {
        self.start_point = start_point;
        self.current_point = start_point;
        self.is_active = true;
    }

    /// Obtener el rectángulo de selección
    pub fn rect(&self) -> Rect {
        let min_x = self.start_point.x.min(self.current_point.x);
        let min_y = self.start_point.y.min(self.current_point.y);
        let max_x = self.start_point.x.max(self.current_point.x);
        let max_y = self.start_point.y.max(self.current_point.y);
        Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Obtener el tamaño del rectángulo
    pub fn size(&self) -> Vec2 {
        let rect = self.rect();
        rect.size()
    }

    /// Verificar si el rectángulo tiene área válida
    pub fn has_area(&self) -> bool {
        let size = self.size();
        size.x > 1.0 && size.y > 1.0
    }

    /// Alternar modo "add to selection"
    pub fn toggle_add_mode(&mut self) {
        self.add_to_selection = !self.add_to_selection;
    }

    /// Configurar modo "add to selection"
    pub fn set_add_mode(&mut self, add: bool) {
        self.add_to_selection = add;
    }
}

impl Default for DragSelectionBox {
    fn default() -> Self {
        Self {
            start_point: Vec2::ZERO,
            current_point: Vec2::ZERO,
            is_active: false,
            add_to_selection: false,
        }
    }
}

/// Configuración visual del drag selection box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragSelectionConfig {
    /// Color del borde
    pub border_color: [f32; 4],
    /// Color del fondo (semi-transparente)
    pub fill_color: [f32; 4],
    /// Ancho del borde
    pub border_width: f32,
    /// Corner radius
    pub corner_radius: f32,
}

impl Default for DragSelectionConfig {
    fn default() -> Self {
        Self {
            border_color: [0.2, 0.6, 1.0, 1.0], // Azul
            fill_color: [0.2, 0.6, 1.0, 0.1],   // Azul semi-transparente
            border_width: 1.0,
            corner_radius: 0.0,
        }
    }
}

/// Resultado de hit testing
#[derive(Debug, Clone)]
pub struct HitTestResult {
    /// Primitiva encontrada (si existe)
    pub primitive: Option<EntityId>,
    /// Distancia al punto (parahit testing en líneas/polilíneas)
    pub distance: f32,
    /// Indica si es un handle específico
    pub handle: Option<HandleType>,
}

/// Tipo de handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleType {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Rotate,
    Scale,
}

impl HandleType {
    /// Obtener todos los tipos de handle para bounding box
    pub fn bounding_box_handles() -> [HandleType; 8] {
        [
            HandleType::TopLeft,
            HandleType::TopCenter,
            HandleType::TopRight,
            HandleType::CenterLeft,
            HandleType::CenterRight,
            HandleType::BottomLeft,
            HandleType::BottomCenter,
            HandleType::BottomRight,
        ]
    }

    /// Calcular posición del handle dado un bounding box
    pub fn position(self, bounds: Rect) -> Vec2 {
        let min = bounds.min;
        let max = bounds.max;
        let center = (min + max) / 2.0;

        match self {
            HandleType::TopLeft => min,
            HandleType::TopCenter => Vec2::new(center.x, min.y),
            HandleType::TopRight => Vec2::new(max.x, min.y),
            HandleType::CenterLeft => Vec2::new(min.x, center.y),
            HandleType::CenterRight => Vec2::new(max.x, center.y),
            HandleType::BottomLeft => Vec2::new(min.x, max.y),
            HandleType::BottomCenter => Vec2::new(center.x, max.y),
            HandleType::BottomRight => max,
            HandleType::Rotate => Vec2::new(center.x, min.y - 20.0),
            HandleType::Scale => Vec2::new(max.x + 10.0, center.y),
        }
    }
}

/// Gestor principal de selección
#[derive(Debug)]
pub struct SelectionManager {
    /// Set de primitivas seleccionadas (para lookups rápidos)
    selected: HashSet<EntityId>,
    /// Vec ordenado para mantener orden de selección
    selection_order: Vec<SelectionEntry>,
    /// Modo actual de selección
    mode: SelectionMode,
    /// Configuración de selección
    config: SelectionConfig,
    /// Engine de intersecciones para hit testing
    intersection: IntersectionEngine,
    /// Selection change counter (para observers)
    version: u64,
}

impl SelectionManager {
    /// Crear nuevo SelectionManager
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            selection_order: Vec::new(),
            mode: SelectionMode::Single,
            config: SelectionConfig::default(),
            intersection: IntersectionEngine::with_config(HitTestConfig::default()),
            version: 0,
        }
    }

    /// Crear con configuración personalizada
    #[inline]
    pub fn with_config(config: SelectionConfig) -> Self {
        Self {
            selected: HashSet::new(),
            selection_order: Vec::new(),
            mode: SelectionMode::Single,
            config,
            intersection: IntersectionEngine::with_config(HitTestConfig::default()),
            version: 0,
        }
    }

    /// Obtener referencia a la configuración
    #[inline]
    pub fn config(&self) -> &SelectionConfig {
        &self.config
    }

    /// Obtener referencia mutable a la configuración
    #[inline]
    pub fn config_mut(&mut self) -> &mut SelectionConfig {
        &mut self.config
    }

    /// Establecer modo de selección
    pub fn set_mode(&mut self, mode: SelectionMode) {
        if self.mode == SelectionMode::Multiple && mode == SelectionMode::Single {
            // Clonar primero para evitar borrow checker issues
            let first_entry = self.selection_order.first().cloned();
            if let Some(entry) = first_entry {
                self.selected.clear();
                self.selected.insert(entry.id);
                self.selection_order.clear();
                self.selection_order.push(entry);
            }
        }
        self.mode = mode;
        self.version = self.version.wrapping_add(1);
    }

    /// Obtener modo actual
    #[inline]
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Verificar si una primitiva está seleccionada
    #[inline]
    pub fn is_selected(&self, id: EntityId) -> bool {
        self.selected.contains(&id)
    }

    /// Obtener todas las primitivas seleccionadas
    #[inline]
    pub fn selected_ids(&self) -> impl Iterator<Item = &EntityId> {
        self.selected.iter()
    }

    /// Obtener primitivas seleccionadas en orden de selección
    #[inline]
    pub fn selected_in_order(&self) -> impl Iterator<Item = &EntityId> {
        self.selection_order.iter().map(|e| &e.id)
    }

    /// Obtener número de elementos seleccionados
    #[inline]
    pub fn count(&self) -> usize {
        self.selected.len()
    }

    /// Verificar si hay selección
    #[inline]
    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }

    /// Obtener la primera primitiva seleccionada
    #[inline]
    pub fn primary(&self) -> Option<EntityId> {
        self.selection_order.first().map(|e| e.id)
    }

    /// Obtener la última primitiva seleccionada
    #[inline]
    pub fn last(&self) -> Option<EntityId> {
        self.selection_order.last().map(|e| e.id)
    }

    /// Seleccionar una primitiva (reemplaza selección actual si modo es Single)
    pub fn select(&mut self, id: EntityId) -> bool {
        if self.mode == SelectionMode::Single && self.selected.contains(&id) {
            return false;
        }

        if self.mode == SelectionMode::Single {
            self.clear();
        }

        if self.selected.insert(id) {
            self.selection_order.push(SelectionEntry::new(id));
            self.version = self.version.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Añadir a la selección (para modo Multiple)
    pub fn add_to_selection(&mut self, id: EntityId) -> bool {
        if self.selected.insert(id) {
            self.selection_order.push(SelectionEntry::new(id));
            self.version = self.version.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Deseleccionar una primitiva
    #[inline]
    pub fn deselect(&mut self, id: EntityId) -> bool {
        if self.selected.remove(&id) {
            self.selection_order.retain(|e| e.id != id);
            self.version = self.version.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Toggle de selección
    pub fn toggle(&mut self, id: EntityId) -> bool {
        if self.is_selected(id) {
            self.deselect(id);
            false
        } else {
            self.add_to_selection(id);
            true
        }
    }

    /// Seleccionar rango de IDs
    pub fn select_range(&mut self, ids: impl Iterator<Item = EntityId>) {
        for id in ids {
            self.add_to_selection(id);
        }
    }

    /// Limpiar selección
    pub fn clear(&mut self) {
        if !self.selected.is_empty() {
            self.selected.clear();
            self.selection_order.clear();
            self.version = self.version.wrapping_add(1);
        }
    }

    /// Seleccionar todo
    pub fn select_all(&mut self, all_ids: impl Iterator<Item = EntityId>) {
        self.clear();
        for id in all_ids {
            self.add_to_selection(id);
        }
    }

    /// Obtener versión (para observers)
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Hit testing contra una primitiva
    pub fn hit_test(&self, point: Vec2, primitive: &dyn Primitive) -> HitTestResult {
        let bounds = primitive.local_bounds();

        // Verificar handles primero (si la primitiva está seleccionada)
        if self.is_selected(primitive.id()) {
            for handle in HandleType::bounding_box_handles() {
                let handle_pos = handle.position(bounds);
                let half_size = self.config.handle_size / 2.0;
                let handle_rect = Rect::from_pos_size(
                    handle_pos - Vec2::new(half_size, half_size),
                    Vec2::splat(self.config.handle_size),
                );
                if self.intersection.point_in_rect(point, handle_rect.into()) {
                    return HitTestResult {
                        primitive: Some(primitive.id()),
                        distance: 0.0,
                        handle: Some(handle),
                    };
                }
            }
        }

        // Verificar si el punto está dentro de la primitiva
        if primitive.contains_point(point) {
            let center = bounds.center();
            let distance = (point - center).length();

            HitTestResult {
                primitive: Some(primitive.id()),
                distance,
                handle: None,
            }
        } else {
            HitTestResult {
                primitive: None,
                distance: f32::MAX,
                handle: None,
            }
        }
    }

    /// Encontrar la primitiva más cercana bajo un punto (hit testing en colección)
    pub fn hit_test_collection<'a>(
        &self,
        point: Vec2,
        primitives: impl Iterator<Item = &'a dyn Primitive>,
    ) -> HitTestResult {
        let mut best_result = HitTestResult {
            primitive: None,
            distance: f32::MAX,
            handle: None,
        };

        for primitive in primitives {
            let result = self.hit_test(point, primitive);

            if result.handle.is_some() {
                return result;
            }

            if result.primitive.is_some() && result.distance < best_result.distance {
                best_result = result;
            }
        }

        best_result
    }

    /// Encontrar todas las primitivas dentro de un rectángulo (drag selection)
    pub fn select_in_rect<'a>(
        &mut self,
        selection_rect: Rect,
        primitives: impl Iterator<Item = &'a dyn Primitive>,
    ) -> Vec<EntityId> {
        let mut found = Vec::new();

        for primitive in primitives {
            let bounds = primitive.local_bounds();
            if self
                .intersection
                .rect_rect(bounds.into(), selection_rect.into())
            {
                self.add_to_selection(primitive.id());
                found.push(primitive.id());
            }
        }

        found
    }
}

impl Default for SelectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SelectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SelectionManager(selected: {}, mode: {:?})",
            self.count(),
            self.mode
        )
    }
}

/// Builder para SelectionManager
pub struct SelectionManagerBuilder {
    mode: SelectionMode,
    config: Option<SelectionConfig>,
}

impl SelectionManagerBuilder {
    /// Create new builder
    #[inline]
    pub fn new() -> Self {
        Self {
            mode: SelectionMode::Single,
            config: None,
        }
    }

    /// Set selection mode
    #[inline]
    pub fn mode(mut self, mode: SelectionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set configuration
    #[inline]
    pub fn config(mut self, config: SelectionConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build SelectionManager
    #[inline]
    pub fn build(self) -> SelectionManager {
        let config = self.config.unwrap_or_default();
        let mut manager = SelectionManager::with_config(config);
        manager.set_mode(self.mode);
        manager
    }
}

impl Default for SelectionManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Primitive, PrimitiveProperties, PrimitiveType, Rectangle};
    use archflow_core::EntityId;

    fn create_test_rectangle(id: u128) -> Box<dyn Primitive> {
        create_test_rectangle_with_pos(id, Vec2::new(0.0, 0.0))
    }

    fn create_test_rectangle_with_pos(id: u128, position: Vec2) -> Box<dyn Primitive> {
        Box::new(TestRectangle {
            id: EntityId::from_u128(id),
            position,
        })
    }

    struct TestRectangle {
        id: EntityId,
        position: Vec2,
    }

    impl Primitive for TestRectangle {
        fn primitive_type(&self) -> PrimitiveType {
            PrimitiveType::Rectangle
        }

        fn id(&self) -> EntityId {
            self.id
        }

        fn transform(&self) -> archflow_core::Transform {
            archflow_core::Transform::identity()
        }

        fn set_transform(&mut self, _: archflow_core::Transform) {}

        fn local_bounds(&self) -> Rect {
            Rect::from_pos_size(self.position, Vec2::new(100.0, 50.0))
        }

        fn global_bounds(&self) -> Rect {
            self.local_bounds()
        }

        fn contains_point(&self, point: Vec2) -> bool {
            self.local_bounds().contains(point)
        }
    }

    #[test]
    fn test_single_selection() {
        let mut manager = SelectionManager::new();
        manager.set_mode(SelectionMode::Single);

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);

        assert!(manager.select(id1));
        assert!(manager.is_selected(id1));
        assert_eq!(manager.count(), 1);

        assert!(manager.select(id2));
        assert!(!manager.is_selected(id1));
        assert!(manager.is_selected(id2));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_multiple_selection() {
        let mut manager = SelectionManager::new();
        manager.set_mode(SelectionMode::Multiple);

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id3 = EntityId::from_u128(3);

        assert!(manager.add_to_selection(id1));
        assert!(manager.add_to_selection(id2));
        assert!(manager.add_to_selection(id3));
        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_toggle_selection() {
        let mut manager = SelectionManager::new();
        let id = EntityId::from_u128(1);

        assert!(manager.toggle(id));
        assert!(manager.is_selected(id));

        assert!(!manager.toggle(id));
        assert!(!manager.is_selected(id));
    }

    #[test]
    fn test_clear_selection() {
        let mut manager = SelectionManager::new();
        manager.set_mode(SelectionMode::Multiple);

        manager.add_to_selection(EntityId::from_u128(1));
        manager.add_to_selection(EntityId::from_u128(2));

        assert!(manager.has_selection());
        manager.clear();
        assert!(!manager.has_selection());
    }

    #[test]
    fn test_hit_test() {
        let manager = SelectionManager::new();
        let primitive = create_test_rectangle(1);

        // Punto dentro
        let result = manager.hit_test(Vec2::new(50.0, 25.0), primitive.as_ref());
        assert!(result.primitive.is_some());

        // Punto fuera
        let result = manager.hit_test(Vec2::new(200.0, 200.0), primitive.as_ref());
        assert!(result.primitive.is_none());
    }

    #[test]
    fn test_select_in_rect() {
        let mut manager = SelectionManager::new();
        manager.set_mode(SelectionMode::Multiple);

        // Rectángulos en diferentes posiciones
        let primitives: Vec<Box<dyn Primitive>> = vec![
            create_test_rectangle_with_pos(1, Vec2::new(10.0, 10.0)), // Dentro del rectángulo
            create_test_rectangle_with_pos(2, Vec2::new(200.0, 200.0)), // Fuera del rectángulo
            create_test_rectangle_with_pos(3, Vec2::new(80.0, 30.0)), // Dentro del rectángulo
        ];

        // Selection rect: (0,0) a (150,100) - contiene los rectángulos 1 y 3
        let selection_rect = Rect::from_pos_size(Vec2::new(0.0, 0.0), Vec2::new(150.0, 100.0));
        let found = manager.select_in_rect(selection_rect, primitives.iter().map(|p| p.as_ref()));

        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_selection_order() {
        let mut manager = SelectionManager::new();
        manager.set_mode(SelectionMode::Multiple);

        let ids: Vec<EntityId> = (1..=3).map(EntityId::from_u128).collect();

        for id in &ids {
            manager.add_to_selection(*id);
        }

        let ordered: Vec<EntityId> = manager.selected_in_order().cloned().collect();
        assert_eq!(ordered, ids);
    }

    #[test]
    fn test_builder() {
        let manager = SelectionManagerBuilder::new()
            .mode(SelectionMode::Multiple)
            .build();

        assert_eq!(manager.mode(), SelectionMode::Multiple);
        assert!(!manager.has_selection());
    }

    // ========== Tests para DragSelectionBox ==========

    #[test]
    fn test_drag_selection_new() {
        let box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));

        assert!(box_selection.is_active);
        assert_eq!(box_selection.start_point, Vec2::new(100.0, 100.0));
        assert_eq!(box_selection.current_point, Vec2::new(100.0, 100.0));
        assert!(!box_selection.add_to_selection);
    }

    #[test]
    fn test_drag_selection_update() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));
        box_selection.update(Vec2::new(200.0, 150.0));

        assert_eq!(box_selection.current_point, Vec2::new(200.0, 150.0));
    }

    #[test]
    fn test_drag_selection_end() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));
        box_selection.end();

        assert!(!box_selection.is_active);
    }

    #[test]
    fn test_drag_selection_rect() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));
        box_selection.update(Vec2::new(200.0, 150.0));

        let rect = box_selection.rect();
        assert_eq!(rect.min, Vec2::new(100.0, 100.0));
        assert_eq!(rect.max, Vec2::new(200.0, 150.0));
    }

    #[test]
    fn test_drag_selection_rect_reverse_drag() {
        // Drag de abajo a arriba (reverse direction)
        let mut box_selection = DragSelectionBox::new(Vec2::new(200.0, 150.0));
        box_selection.update(Vec2::new(100.0, 100.0));

        let rect = box_selection.rect();
        assert_eq!(rect.min, Vec2::new(100.0, 100.0));
        assert_eq!(rect.max, Vec2::new(200.0, 150.0));
    }

    #[test]
    fn test_drag_selection_has_area() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));

        // Sin movimiento - sin área
        assert!(!box_selection.has_area());

        // Con movimiento pequeño - sin área
        box_selection.update(Vec2::new(100.5, 100.5));
        assert!(!box_selection.has_area());

        // Con movimiento suficiente - tiene área
        box_selection.update(Vec2::new(150.0, 150.0));
        assert!(box_selection.has_area());
    }

    #[test]
    fn test_drag_selection_add_mode() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));

        assert!(!box_selection.add_to_selection);

        box_selection.toggle_add_mode();
        assert!(box_selection.add_to_selection);

        box_selection.set_add_mode(false);
        assert!(!box_selection.add_to_selection);
    }

    #[test]
    fn test_drag_selection_restart() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));
        box_selection.update(Vec2::new(200.0, 200.0));
        box_selection.end();

        // Restart
        box_selection.restart(Vec2::new(300.0, 300.0));

        assert!(box_selection.is_active);
        assert_eq!(box_selection.start_point, Vec2::new(300.0, 300.0));
        assert_eq!(box_selection.current_point, Vec2::new(300.0, 300.0));
    }

    #[test]
    fn test_drag_selection_size() {
        let mut box_selection = DragSelectionBox::new(Vec2::new(100.0, 100.0));
        box_selection.update(Vec2::new(200.0, 150.0));

        let size = box_selection.size();
        assert_eq!(size, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_drag_selection_default() {
        let default_box = DragSelectionBox::default();

        assert!(!default_box.is_active);
        assert_eq!(default_box.start_point, Vec2::ZERO);
        assert_eq!(default_box.current_point, Vec2::ZERO);
        assert!(!default_box.add_to_selection);
    }
}

/// Helper trait for creating EntityId from u128
pub trait IntoEntityId {
    fn entity_id(self) -> EntityId;
}

impl IntoEntityId for u128 {
    fn entity_id(self) -> EntityId {
        let bytes = self.to_be_bytes();
        let uuid = Uuid::from_bytes(bytes);
        EntityId::from(uuid)
    }
}
