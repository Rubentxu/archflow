//! ArchFlow Drag & Drop System - Sistema de arrastre de primitivas
//!
//! Proporciona:
//! - Draggable trait para primitivas
//! - Drag con feedback visual instantáneo
//! - Snap to grid configurable
//! - Multi-drag para múltiples objetos

use crate::{EntityId, Primitive, Vec2};
use archflow_core::Rect;
use serde::{Deserialize, Serialize};

/// Estado del drag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragState {
    /// No está arrastrando
    Idle,
    /// Preparando para arrastrar (mouse down pero no ha alcanzado threshold)
    Preparing,
    /// Arrastrando activamente
    Dragging,
    /// Drag cancelado
    Cancelled,
}

/// Evento de drag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DragEvent {
    /// Inicio del drag
    DragStarted {
        id: EntityId,
        start_position: Vec2,
        current_position: Vec2,
    },
    /// Durante el drag
    Dragging {
        id: EntityId,
        start_position: Vec2,
        current_position: Vec2,
        delta: Vec2,
    },
    /// Fin del drag
    DragEnded {
        id: EntityId,
        start_position: Vec2,
        end_position: Vec2,
        total_delta: Vec2,
    },
    /// Drag cancelado
    DragCancelled { id: EntityId, start_position: Vec2 },
}

/// Configuración de snap to grid
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapConfig {
    /// Habilitar snap
    pub enabled: bool,
    /// Tamaño de celda
    pub grid_size: Vec2,
    /// Tolerancia de snap (en pixels)
    pub tolerance: f32,
    /// Mostrar guía de grid
    pub show_guides: bool,
    /// Color de las guías
    pub guide_color: [f32; 4],
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            grid_size: Vec2::new(10.0, 10.0),
            tolerance: 5.0,
            show_guides: false,
            guide_color: [0.5, 0.5, 0.5, 0.5],
        }
    }
}

/// Configuración visual del feedback de drag
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragFeedbackConfig {
    /// Opacidad durante el drag
    pub opacity: f32,
    /// Escala durante el drag
    pub scale: f32,
    /// Sombra durante el drag
    pub shadow_blur: f32,
    /// Color de sombra
    pub shadow_color: [f32; 4],
    /// Borde de highlight
    pub highlight_color: [f32; 4],
    pub highlight_width: f32,
    /// Mostrar línea de conexión al cursor
    pub show_cursor_line: bool,
    /// Color de la línea al cursor
    pub cursor_line_color: [f32; 4],
}

impl Default for DragFeedbackConfig {
    fn default() -> Self {
        Self {
            opacity: 0.8,
            scale: 1.05,
            shadow_blur: 10.0,
            shadow_color: [0.0, 0.0, 0.0, 0.3],
            highlight_color: [0.2, 0.6, 1.0, 1.0],
            highlight_width: 2.0,
            show_cursor_line: true,
            cursor_line_color: [0.2, 0.6, 1.0, 0.5],
        }
    }
}

/// Trait para objetos arrastrables
pub trait Draggable {
    /// Obtener el ID de la entidad
    fn id(&self) -> EntityId;

    /// Posición actual
    fn position(&self) -> Vec2;

    /// Establecer nueva posición
    fn set_position(&mut self, position: Vec2);

    /// Dimensiones para feedback visual
    fn bounds(&self) -> Rect;

    /// Verificar si el punto está dentro del área de arrastre
    fn contains_point(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }
}

/// Gestor de drag & drop
#[derive(Debug, Clone)]
pub struct DragManager {
    /// Estado actual del drag
    state: DragState,
    /// Entidad que se está arrastrando
    active_id: Option<EntityId>,
    /// Posición donde comenzó el drag
    start_position: Vec2,
    /// Posición actual del mouse
    current_mouse_position: Vec2,
    /// Posición anterior (para delta)
    previous_mouse_position: Vec2,
    /// Configuración de snap
    snap_config: SnapConfig,
    /// Configuración de feedback visual
    feedback_config: DragFeedbackConfig,
    /// Threshold para iniciar drag (en pixels)
    drag_threshold: f32,
    /// Eventos acumulados
    events: Vec<DragEvent>,
    /// Multi-drag: entidades seleccionadas que se arrastran juntas
    multi_drag_ids: Vec<EntityId>,
    /// Versión para observers
    version: u64,
}

impl DragManager {
    /// Crear nuevo DragManager
    pub fn new() -> Self {
        Self {
            state: DragState::Idle,
            active_id: None,
            start_position: Vec2::ZERO,
            current_mouse_position: Vec2::ZERO,
            previous_mouse_position: Vec2::ZERO,
            snap_config: SnapConfig::default(),
            feedback_config: DragFeedbackConfig::default(),
            drag_threshold: 3.0,
            events: Vec::new(),
            multi_drag_ids: Vec::new(),
            version: 0,
        }
    }

    /// Crear con configuración personalizada
    #[inline]
    pub fn with_snap_config(snap_config: SnapConfig) -> Self {
        Self {
            state: DragState::Idle,
            active_id: None,
            start_position: Vec2::ZERO,
            current_mouse_position: Vec2::ZERO,
            previous_mouse_position: Vec2::ZERO,
            snap_config,
            feedback_config: DragFeedbackConfig::default(),
            drag_threshold: 3.0,
            events: Vec::new(),
            multi_drag_ids: Vec::new(),
            version: 0,
        }
    }

    /// Obtener referencia a la configuración de snap
    #[inline]
    pub fn snap_config(&self) -> &SnapConfig {
        &self.snap_config
    }

    /// Obtener referencia mutable a la configuración de snap
    #[inline]
    pub fn snap_config_mut(&mut self) -> &mut SnapConfig {
        &mut self.snap_config
    }

    /// Obtener referencia a la configuración de feedback
    #[inline]
    pub fn feedback_config(&self) -> &DragFeedbackConfig {
        &self.feedback_config
    }

    /// Obtener referencia mutable a la configuración de feedback
    #[inline]
    pub fn feedback_config_mut(&mut self) -> &mut DragFeedbackConfig {
        &mut self.feedback_config
    }

    /// Obtener estado actual
    #[inline]
    pub fn state(&self) -> DragState {
        self.state
    }

    /// Obtener entidad activa
    #[inline]
    pub fn active_id(&self) -> Option<EntityId> {
        self.active_id
    }

    /// Obtener posición de inicio
    #[inline]
    pub fn start_position(&self) -> Vec2 {
        self.start_position
    }

    /// Obtener posición actual del mouse
    #[inline]
    pub fn current_mouse_position(&self) -> Vec2 {
        self.current_mouse_position
    }

    /// Obtener delta desde el inicio
    #[inline]
    pub fn total_delta(&self) -> Vec2 {
        self.current_mouse_position - self.start_position
    }

    /// Obtener delta desde el último frame
    #[inline]
    pub fn frame_delta(&self) -> Vec2 {
        self.current_mouse_position - self.previous_mouse_position
    }

    /// Verificar si está arrastrando
    #[inline]
    pub fn is_dragging(&self) -> bool {
        self.state == DragState::Dragging
    }

    /// Verificar si está preparándose para arrastrar
    #[inline]
    pub fn is_preparing(&self) -> bool {
        self.state == DragState::Preparing
    }

    /// Iniciar preparación para drag
    pub fn start_preparing(&mut self, id: EntityId, mouse_position: Vec2) -> bool {
        if self.state != DragState::Idle {
            return false;
        }

        self.active_id = Some(id);
        self.start_position = mouse_position;
        self.current_mouse_position = mouse_position;
        self.previous_mouse_position = mouse_position;
        self.state = DragState::Preparing;
        self.version = self.version.wrapping_add(1);

        self.events.push(DragEvent::DragStarted {
            id,
            start_position: mouse_position,
            current_position: mouse_position,
        });

        true
    }

    /// Actualizar posición del mouse durante preparación
    pub fn update_preparing(&mut self, mouse_position: Vec2) -> bool {
        let delta = (mouse_position - self.start_position).length();
        let was_at = self.current_mouse_position; // Save position before transition

        if delta >= self.drag_threshold {
            // Transicionar a dragging
            self.state = DragState::Dragging;
            self.current_mouse_position = mouse_position; // Update current position
            self.previous_mouse_position = was_at; // Use last preparing position for accurate frame delta

            // Emitir evento de drag started si no se emitió
            if let Some(_id) = self.active_id {
                // Actualizar el evento inicial con la posición correcta
                if let Some(first_event) = self.events.first_mut() {
                    if let DragEvent::DragStarted {
                        current_position, ..
                    } = first_event
                    {
                        *current_position = mouse_position;
                    }
                }
            }

            true
        } else {
            self.current_mouse_position = mouse_position;
            false
        }
    }

    /// Actualizar durante drag activo
    pub fn update_drag(&mut self, mouse_position: Vec2) {
        self.previous_mouse_position = self.current_mouse_position;
        self.current_mouse_position = mouse_position;

        if let Some(id) = self.active_id {
            self.events.push(DragEvent::Dragging {
                id,
                start_position: self.start_position,
                current_position: mouse_position,
                delta: self.frame_delta(),
            });
        }
    }

    /// Finalizar drag
    pub fn end_drag(&mut self) -> Option<DragEvent> {
        let result = if let Some(id) = self.active_id {
            let event = DragEvent::DragEnded {
                id,
                start_position: self.start_position,
                end_position: self.current_mouse_position,
                total_delta: self.total_delta(),
            };
            self.events.push(event.clone());
            Some(event)
        } else {
            None
        };

        self.state = DragState::Idle;
        self.active_id = None;
        self.multi_drag_ids.clear();
        self.version = self.version.wrapping_add(1);

        result
    }

    /// Cancelar drag
    pub fn cancel_drag(&mut self) -> Option<DragEvent> {
        let result = if let Some(id) = self.active_id {
            let event = DragEvent::DragCancelled {
                id,
                start_position: self.start_position,
            };
            self.events.push(event.clone());
            Some(event)
        } else {
            None
        };

        self.state = DragState::Idle;
        self.active_id = None;
        self.multi_drag_ids.clear();
        self.version = self.version.wrapping_add(1);

        result
    }

    /// Calcular posición con snap a grid
    pub fn apply_snap(&self, position: Vec2) -> Vec2 {
        if !self.snap_config.enabled {
            return position;
        }

        let _half_grid = self.snap_config.grid_size / 2.0;
        let snapped_x =
            (position.x / self.snap_config.grid_size.x).round() * self.snap_config.grid_size.x;
        let snapped_y =
            (position.y / self.snap_config.grid_size.y).round() * self.snap_config.grid_size.y;

        Vec2::new(snapped_x, snapped_y)
    }

    /// Calcular múltiples posiciones con snap (para multi-drag)
    pub fn apply_snap_multi(&self, positions: &[Vec2]) -> Vec<Vec2> {
        if !self.snap_config.enabled {
            return positions.to_vec();
        }

        positions.iter().map(|p| self.apply_snap(*p)).collect()
    }

    /// Verificar snap guides y retornar posiciones de guías
    pub fn get_snap_guides(&self, position: Vec2, bounds: Rect) -> SnapGuides {
        if !self.snap_config.enabled || !self.snap_config.show_guides {
            return SnapGuides {
                horizontal: None,
                vertical: None,
            };
        }

        let snapped = self.apply_snap(position);

        let horizontal = (snapped.y - position.y).abs() <= self.snap_config.tolerance;
        let vertical = (snapped.x - position.x).abs() <= self.snap_config.tolerance;

        SnapGuides {
            horizontal: if horizontal {
                Some(SnapGuideLine::horizontal(
                    snapped.y,
                    (bounds.min.x, bounds.max.x),
                ))
            } else {
                None
            },
            vertical: if vertical {
                Some(SnapGuideLine::vertical(
                    snapped.x,
                    (bounds.min.y, bounds.max.y),
                ))
            } else {
                None
            },
        }
    }

    /// Configurar multi-drag
    pub fn set_multi_drag(&mut self, ids: Vec<EntityId>) {
        self.multi_drag_ids = ids;
    }

    /// Obtener IDs para multi-drag
    #[inline]
    pub fn multi_drag_ids(&self) -> &[EntityId] {
        &self.multi_drag_ids
    }

    /// Verificar si hay multi-drag activo
    #[inline]
    pub fn is_multi_drag(&self) -> bool {
        // Check if multi-drag is configured (multiple entities selected for drag)
        // regardless of current drag state
        !self.multi_drag_ids.is_empty()
    }

    /// Obtener eventos acumulados
    #[inline]
    pub fn events(&self) -> &[DragEvent] {
        &self.events
    }

    /// Limpiar eventos acumulados
    #[inline]
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Obtener versión (para observers)
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Hit test para iniciar drag
    pub fn hit_test<T: Draggable>(&self, point: Vec2, draggable: &T) -> bool {
        draggable.contains_point(point)
    }
}

impl Default for DragManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Línea de guía de snap
#[derive(Debug, Clone, PartialEq)]
pub struct SnapGuideLine {
    /// Para guías horizontales: y coordinate
    /// Para guías verticales: x coordinate
    pub coordinate: f32,
    /// Rango de la otra dimensión
    pub range: (f32, f32),
}

impl SnapGuideLine {
    /// Crear guía horizontal
    pub fn horizontal(y: f32, x_range: (f32, f32)) -> Self {
        Self {
            coordinate: y,
            range: x_range,
        }
    }

    /// Crear guía vertical
    pub fn vertical(x: f32, y_range: (f32, f32)) -> Self {
        Self {
            coordinate: x,
            range: y_range,
        }
    }
}

/// Guías de snap calculadas
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SnapGuides {
    pub horizontal: Option<SnapGuideLine>,
    pub vertical: Option<SnapGuideLine>,
}

/// Builder para DragManager
pub struct DragManagerBuilder {
    snap_config: Option<SnapConfig>,
    feedback_config: Option<DragFeedbackConfig>,
    drag_threshold: Option<f32>,
}

impl DragManagerBuilder {
    /// Create new builder
    #[inline]
    pub fn new() -> Self {
        Self {
            snap_config: None,
            feedback_config: None,
            drag_threshold: None,
        }
    }

    /// Set snap configuration
    #[inline]
    pub fn snap_config(mut self, snap_config: SnapConfig) -> Self {
        self.snap_config = Some(snap_config);
        self
    }

    /// Set feedback configuration
    #[inline]
    pub fn feedback_config(mut self, feedback_config: DragFeedbackConfig) -> Self {
        self.feedback_config = Some(feedback_config);
        self
    }

    /// Set drag threshold
    #[inline]
    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = Some(threshold);
        self
    }

    /// Build DragManager
    #[inline]
    pub fn build(self) -> DragManager {
        let mut manager = DragManager::new();

        if let Some(config) = self.snap_config {
            manager.snap_config = config;
        }

        if let Some(config) = self.feedback_config {
            manager.feedback_config = config;
        }

        if let Some(threshold) = self.drag_threshold {
            manager.drag_threshold = threshold;
        }

        manager
    }
}

impl Default for DragManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Draggable;
    use archflow_core::EntityId;

    struct TestDraggable {
        id: EntityId,
        position: Vec2,
        size: Vec2,
    }

    impl TestDraggable {
        fn new(id: u128, x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                id: EntityId::from_u128(id),
                position: Vec2::new(x, y),
                size: Vec2::new(width, height),
            }
        }
    }

    impl Draggable for TestDraggable {
        fn id(&self) -> EntityId {
            self.id
        }

        fn position(&self) -> Vec2 {
            self.position
        }

        fn set_position(&mut self, position: Vec2) {
            self.position = position;
        }

        fn bounds(&self) -> Rect {
            Rect::from_pos_size(self.position, self.size)
        }
    }

    #[test]
    fn test_drag_manager_new() {
        let manager = DragManager::new();

        assert_eq!(manager.state(), DragState::Idle);
        assert!(manager.active_id().is_none());
        assert!(!manager.is_dragging());
    }

    #[test]
    fn test_start_preparing() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);
        let position = Vec2::new(100.0, 100.0);

        assert!(manager.start_preparing(id, position));
        assert_eq!(manager.state(), DragState::Preparing);
        assert_eq!(manager.active_id(), Some(id));
        assert_eq!(manager.start_position(), position);
    }

    #[test]
    fn test_prevent_double_prepare() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        assert!(manager.start_preparing(id, Vec2::new(100.0, 100.0)));
        assert!(!manager.start_preparing(id, Vec2::new(200.0, 200.0)));
    }

    #[test]
    fn test_update_preparing_below_threshold() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));

        // Mover solo 2 pixels (menos que el threshold de 3)
        assert!(!manager.update_preparing(Vec2::new(101.0, 102.0)));
        assert_eq!(manager.state(), DragState::Preparing);
    }

    #[test]
    fn test_update_preparing_above_threshold() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));

        // Mover 5 pixels (más que el threshold de 3)
        assert!(manager.update_preparing(Vec2::new(103.0, 104.0)));
        assert_eq!(manager.state(), DragState::Dragging);
    }

    #[test]
    fn test_update_drag() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));
        manager.update_preparing(Vec2::new(200.0, 200.0));

        let previous = manager.current_mouse_position();
        manager.update_drag(Vec2::new(210.0, 220.0));

        assert_eq!(manager.frame_delta(), Vec2::new(10.0, 20.0));
        assert_eq!(manager.total_delta(), Vec2::new(110.0, 120.0));
    }

    #[test]
    fn test_end_drag() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));
        manager.update_preparing(Vec2::new(200.0, 200.0));
        manager.update_drag(Vec2::new(250.0, 250.0));

        let event = manager.end_drag();

        assert!(event.is_some());
        if let DragEvent::DragEnded {
            start_position,
            end_position,
            total_delta,
            ..
        } = event.unwrap()
        {
            assert_eq!(start_position, Vec2::new(100.0, 100.0));
            assert_eq!(end_position, Vec2::new(250.0, 250.0));
            assert_eq!(total_delta, Vec2::new(150.0, 150.0));
        }

        assert_eq!(manager.state(), DragState::Idle);
        assert!(manager.active_id().is_none());
    }

    #[test]
    fn test_cancel_drag() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));
        manager.update_preparing(Vec2::new(200.0, 200.0));

        let event = manager.cancel_drag();

        assert!(event.is_some());
        if let DragEvent::DragCancelled {
            id: cancelled_id, ..
        } = event.unwrap()
        {
            assert_eq!(cancelled_id, id);
        }

        assert_eq!(manager.state(), DragState::Idle);
    }

    #[test]
    fn test_snap_disabled() {
        let manager = DragManager::new();
        let position = Vec2::new(105.5, 205.5);

        assert_eq!(manager.apply_snap(position), position);
    }

    #[test]
    fn test_snap_enabled() {
        let mut manager = DragManager::new();
        manager.snap_config_mut().enabled = true;
        manager.snap_config_mut().grid_size = Vec2::new(10.0, 10.0);

        let position = Vec2::new(105.5, 205.5);
        let snapped = manager.apply_snap(position);

        assert_eq!(snapped, Vec2::new(110.0, 210.0));
    }

    #[test]
    fn test_snap_guides() {
        let mut manager = DragManager::new();
        manager.snap_config_mut().enabled = true;
        manager.snap_config_mut().grid_size = Vec2::new(10.0, 10.0);
        manager.snap_config_mut().tolerance = 5.0;
        manager.snap_config_mut().show_guides = true;

        let position = Vec2::new(105.0, 205.0); // 5px away from snap point
        let bounds = Rect::from_pos_size(Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0));

        let guides = manager.get_snap_guides(position, bounds);

        assert!(guides.horizontal.is_some());
        assert!(guides.vertical.is_some());
    }

    #[test]
    fn test_multi_drag() {
        let mut manager = DragManager::new();
        let ids = vec![EntityId::from_u128(1), EntityId::from_u128(2)];
        manager.set_multi_drag(ids.clone());
        assert!(manager.is_multi_drag());
        assert_eq!(manager.multi_drag_ids(), &ids);
    }

    #[test]
    fn test_builder() {
        let manager = DragManagerBuilder::new()
            .snap_config(SnapConfig {
                enabled: true,
                grid_size: Vec2::new(20.0, 20.0),
                ..Default::default()
            })
            .drag_threshold(5.0)
            .build();

        assert!(manager.snap_config().enabled);
        assert_eq!(manager.snap_config().grid_size, Vec2::new(20.0, 20.0));
    }

    #[test]
    fn test_draggable_trait() {
        let draggable = TestDraggable::new(1, 100.0, 100.0, 50.0, 30.0);

        assert_eq!(draggable.id(), EntityId::from_u128(1));
        assert_eq!(draggable.position(), Vec2::new(100.0, 100.0));
        assert_eq!(
            draggable.bounds(),
            Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0))
        );

        // Test contains_point
        assert!(draggable.contains_point(Vec2::new(120.0, 110.0)));
        assert!(!draggable.contains_point(Vec2::new(200.0, 200.0)));
    }

    #[test]
    fn test_events_accumulation() {
        let mut manager = DragManager::new();
        let id = EntityId::from_u128(1);

        manager.start_preparing(id, Vec2::new(100.0, 100.0));
        manager.update_preparing(Vec2::new(200.0, 200.0));
        manager.update_drag(Vec2::new(210.0, 210.0));
        manager.update_drag(Vec2::new(220.0, 220.0));
        manager.end_drag();

        assert!(manager.events().len() >= 3); // Started + 2 Dragging + Ended
    }

    #[test]
    fn test_hit_test() {
        let manager = DragManager::new();
        let draggable = TestDraggable::new(1, 100.0, 100.0, 50.0, 30.0);
        assert!(manager.hit_test(Vec2::new(125.0, 115.0), &draggable));
        assert!(!manager.hit_test(Vec2::new(200.0, 200.0), &draggable));
    }
}
