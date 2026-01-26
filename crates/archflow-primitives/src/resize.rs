//! ArchFlow Resize System - Sistema de redimensionado de primitivas
//!
//! Proporciona:
//! - Resizable trait para primitivas
//! - Handles en corners y edges (8 posiciones)
//! - Aspect ratio lock
//! - Min/max constraints
//! - Centrado automático durante resize

use crate::{EntityId, Primitive, Vec2};
use archflow_core::{Rect, Transform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipo de handle de redimensionado
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
}

impl HandleType {
    /// Obtener todos los handles de un bounding box
    pub fn bounding_box_handles() -> &'static [HandleType] {
        &[
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

    /// Obtener posición del handle para un rectángulo
    pub fn position(&self, bounds: Rect) -> Vec2 {
        match self {
            HandleType::TopLeft => bounds.min,
            HandleType::TopCenter => Vec2::new(bounds.center().x, bounds.min.y),
            HandleType::TopRight => Vec2::new(bounds.max.x, bounds.min.y),
            HandleType::CenterLeft => Vec2::new(bounds.min.x, bounds.center().y),
            HandleType::CenterRight => Vec2::new(bounds.max.x, bounds.center().y),
            HandleType::BottomLeft => Vec2::new(bounds.min.x, bounds.max.y),
            HandleType::BottomCenter => Vec2::new(bounds.center().x, bounds.max.y),
            HandleType::BottomRight => bounds.max,
            HandleType::Rotate => Vec2::new(bounds.center().x, bounds.min.y - 20.0),
        }
    }
}

/// Estado del resize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeState {
    /// No está redimensionando
    Idle,
    /// Preparando para redimensionar
    Preparing,
    /// Redimensionando activamente
    Resizing,
    /// Redimensionado cancelado
    Cancelled,
}

/// Evento de resize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResizeEvent {
    /// Inicio del resize
    ResizeStarted {
        id: EntityId,
        handle: HandleType,
        initial_bounds: Rect,
        initial_mouse: Vec2,
    },
    /// Durante el resize
    Resizing {
        id: EntityId,
        handle: HandleType,
        initial_bounds: Rect,
        initial_mouse: Vec2,
        current_mouse: Vec2,
        new_bounds: Rect,
    },
    /// Fin del resize
    ResizeEnded {
        id: EntityId,
        handle: HandleType,
        initial_bounds: Rect,
        final_bounds: Rect,
    },
    /// Resize cancelado
    ResizeCancelled {
        id: EntityId,
        handle: HandleType,
        initial_bounds: Rect,
    },
}

/// Configuración de aspecto ratio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AspectRatioMode {
    /// Sin restricción de aspecto
    Free,
    /// Mantener aspecto original
    Keep,
    /// Forzar aspecto específico (width/height)
    Fixed(f32),
    /// Forzar dimensiones específicas
    FixedSize(f32, f32),
    /// Limitar a cuadrado
    Square,
}

impl Default for AspectRatioMode {
    fn default() -> Self {
        AspectRatioMode::Free
    }
}

impl AspectRatioMode {
    /// Calcular aspecto forzado
    pub fn forced_aspect(&self, bounds: Rect) -> Option<f32> {
        match self {
            AspectRatioMode::Free => None,
            AspectRatioMode::Keep => Some(bounds.width() / bounds.height()),
            AspectRatioMode::Fixed(ratio) => Some(*ratio),
            AspectRatioMode::FixedSize(w, h) => Some(w / h),
            AspectRatioMode::Square => Some(1.0),
        }
    }

    /// Aplicar aspecto a nuevas dimensiones
    pub fn apply_aspect(&self, width: f32, height: f32, bounds: Rect) -> (f32, f32) {
        if let Some(ratio) = self.forced_aspect(bounds) {
            let new_ratio = width / height.max(f32::EPSILON);
            if new_ratio > ratio {
                (width, width / ratio)
            } else {
                (height * ratio, height)
            }
        } else {
            (width, height)
        }
    }
}

/// Configuración de restricciones de tamaño
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SizeConstraints {
    /// Ancho mínimo
    pub min_width: f32,
    /// Alto mínimo
    pub min_height: f32,
    /// Ancho máximo (0 = sin límite)
    pub max_width: f32,
    /// Alto máximo (0 = sin límite)
    pub max_height: f32,
    /// Área mínima
    pub min_area: f32,
    /// Área máxima (0 = sin límite)
    pub max_area: f32,
    /// Permitir dimensiones inversas
    pub allow_negative: bool,
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 1.0,
            min_height: 1.0,
            max_width: 0.0,
            max_height: 0.0,
            min_area: 0.0,
            max_area: 0.0,
            allow_negative: false,
        }
    }
}

impl SizeConstraints {
    /// Crear restricciones con dimensiones mínimas
    pub fn with_min_size(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            min_height: height,
            ..Default::default()
        }
    }

    /// Crear restricciones con dimensiones máximas
    pub fn with_max_size(width: f32, height: f32) -> Self {
        Self {
            max_width: width,
            max_height: height,
            ..Default::default()
        }
    }

    /// Crear restricciones con dimensiones específicas
    pub fn with_size(min_w: f32, min_h: f32, max_w: f32, max_h: f32) -> Self {
        Self {
            min_width: min_w,
            min_height: min_h,
            max_width: max_w,
            max_height: max_h,
            ..Default::default()
        }
    }

    /// Validar dimensiones
    pub fn validate(&self, width: f32, height: f32) -> (f32, f32) {
        let mut w = width.abs();
        let mut h = height.abs();

        // Aplicar mínimos
        w = w.max(self.min_width);
        h = h.max(self.min_height);

        // Aplicar máximos
        if self.max_width > 0.0 {
            w = w.min(self.max_width);
        }
        if self.max_height > 0.0 {
            h = h.min(self.max_height);
        }

        // Aplicar límites de área
        let area = w * h;
        if self.max_area > 0.0 && area > self.max_area {
            let ratio = (self.max_area / area).sqrt();
            w *= ratio;
            h *= ratio;
        }

        if self.min_area > 0.0 && area < self.min_area {
            let ratio = (self.min_area / area).sqrt();
            w *= ratio;
            h *= ratio;
        }

        // Devolver con signo original si está permitido
        if self.allow_negative {
            (w * width.signum(), h * height.signum())
        } else {
            (w, h)
        }
    }
}

/// Configuración visual del resize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizeFeedbackConfig {
    /// Color del handle
    pub handle_color: [f32; 4],
    /// Color del handle hovered
    pub handle_hover_color: [f32; 4],
    /// Color del handle active
    pub handle_active_color: [f32; 4],
    /// Tamaño del handle
    pub handle_size: f32,
    /// Grosor del borde de guía
    pub guide_width: f32,
    /// Color de la guía de dimensiones
    pub guide_color: [f32; 4],
    /// Mostrar dimensiones durante resize
    pub show_dimensions: bool,
    /// Color del texto de dimensiones
    pub dimension_text_color: [f32; 4],
    /// Opacidad de la primitiva durante resize
    pub opacity: f32,
    /// Borde de highlight durante resize
    pub highlight_color: [f32; 4],
    pub highlight_width: f32,
}

impl Default for ResizeFeedbackConfig {
    fn default() -> Self {
        Self {
            handle_color: [1.0, 1.0, 1.0, 1.0],
            handle_hover_color: [0.2, 0.6, 1.0, 1.0],
            handle_active_color: [0.2, 0.6, 1.0, 1.0],
            handle_size: 8.0,
            guide_width: 1.0,
            guide_color: [0.2, 0.6, 1.0, 0.8],
            show_dimensions: true,
            dimension_text_color: [0.0, 0.0, 0.0, 1.0],
            opacity: 0.8,
            highlight_color: [0.2, 0.6, 1.0, 1.0],
            highlight_width: 2.0,
        }
    }
}

/// Trait para objetos redimensionables
pub trait Resizable {
    /// Obtener el ID de la entidad
    fn id(&self) -> EntityId;

    /// Obtener límites actuales
    fn bounds(&self) -> Rect;

    /// Establecer nuevos límites
    fn set_bounds(&mut self, bounds: Rect);

    /// Obtener transformación actual
    fn transform(&self) -> Transform;

    /// Aplicar transformación
    fn apply_transform(&mut self, transform: Transform);

    /// Verificar si es un shape que soporta resize
    fn can_resize(&self) -> bool {
        true
    }
}

/// Gestor de redimensionado
#[derive(Debug, Clone)]
pub struct ResizeManager {
    /// Estado actual del resize
    state: ResizeState,
    /// Entidad que se está redimensionando
    active_id: Option<EntityId>,
    /// Handle activo
    active_handle: Option<HandleType>,
    /// Límites originales
    initial_bounds: Rect,
    /// Posición original del mouse
    initial_mouse: Vec2,
    /// Posición actual del mouse
    current_mouse: Vec2,
    /// Modo de aspecto ratio
    aspect_mode: AspectRatioMode,
    /// Restricciones de tamaño
    constraints: SizeConstraints,
    /// Configuración de feedback visual
    feedback_config: ResizeFeedbackConfig,
    /// Modo de centrado durante resize
    pub center_mode: ResizeCenterMode,
    /// Threshold para iniciar resize (en pixels)
    resize_threshold: f32,
    /// Eventos acumulados
    events: Vec<ResizeEvent>,
    /// Versión para observers
    version: u64,
}

impl ResizeManager {
    /// Crear nuevo ResizeManager
    pub fn new() -> Self {
        Self {
            state: ResizeState::Idle,
            active_id: None,
            active_handle: None,
            initial_bounds: Rect::default(),
            initial_mouse: Vec2::ZERO,
            current_mouse: Vec2::ZERO,
            aspect_mode: AspectRatioMode::default(),
            constraints: SizeConstraints::default(),
            feedback_config: ResizeFeedbackConfig::default(),
            center_mode: ResizeCenterMode::default(),
            resize_threshold: 2.0,
            events: Vec::new(),
            version: 0,
        }
    }

    /// Crear con configuración personalizada
    #[inline]
    pub fn with_aspect_mode(aspect_mode: AspectRatioMode) -> Self {
        Self {
            state: ResizeState::Idle,
            active_id: None,
            active_handle: None,
            initial_bounds: Rect::default(),
            initial_mouse: Vec2::ZERO,
            current_mouse: Vec2::ZERO,
            aspect_mode,
            constraints: SizeConstraints::default(),
            feedback_config: ResizeFeedbackConfig::default(),
            center_mode: ResizeCenterMode::default(),
            resize_threshold: 2.0,
            events: Vec::new(),
            version: 0,
        }
    }

    /// Obtener estado actual
    #[inline]
    pub fn state(&self) -> ResizeState {
        self.state
    }

    /// Obtener entidad activa
    #[inline]
    pub fn active_id(&self) -> Option<EntityId> {
        self.active_id
    }

    /// Obtener handle activo
    #[inline]
    pub fn active_handle(&self) -> Option<HandleType> {
        self.active_handle
    }

    /// Obtener límites originales
    #[inline]
    pub fn initial_bounds(&self) -> Rect {
        self.initial_bounds
    }

    /// Obtener posición inicial del mouse
    #[inline]
    pub fn initial_mouse(&self) -> Vec2 {
        self.initial_mouse
    }

    /// Obtener posición actual del mouse
    #[inline]
    pub fn current_mouse(&self) -> Vec2 {
        self.current_mouse
    }

    /// Obtener referencia a la configuración de aspecto
    #[inline]
    pub fn aspect_mode(&self) -> &AspectRatioMode {
        &self.aspect_mode
    }

    /// Obtener referencia mutable a la configuración de aspecto
    #[inline]
    pub fn aspect_mode_mut(&mut self) -> &mut AspectRatioMode {
        &mut self.aspect_mode
    }

    /// Obtener referencia a las restricciones
    #[inline]
    pub fn constraints(&self) -> &SizeConstraints {
        &self.constraints
    }

    /// Obtener referencia mutable a las restricciones
    #[inline]
    pub fn constraints_mut(&mut self) -> &mut SizeConstraints {
        &mut self.constraints
    }

    /// Obtener referencia a la configuración de feedback
    #[inline]
    pub fn feedback_config(&self) -> &ResizeFeedbackConfig {
        &self.feedback_config
    }

    /// Obtener referencia mutable a la configuración de feedback
    #[inline]
    pub fn feedback_config_mut(&mut self) -> &mut ResizeFeedbackConfig {
        &mut self.feedback_config
    }

    /// Verificar si está redimensionando
    #[inline]
    pub fn is_resizing(&self) -> bool {
        self.state == ResizeState::Resizing
    }

    /// Verificar si está preparándose para redimensionar
    #[inline]
    pub fn is_preparing(&self) -> bool {
        self.state == ResizeState::Preparing
    }

    /// Iniciar preparación para resize
    pub fn start_preparing(
        &mut self,
        id: EntityId,
        handle: HandleType,
        bounds: Rect,
        mouse_position: Vec2,
    ) -> bool {
        if self.state != ResizeState::Idle {
            return false;
        }

        self.active_id = Some(id);
        self.active_handle = Some(handle);
        self.initial_bounds = bounds;
        self.initial_mouse = mouse_position;
        self.current_mouse = mouse_position;
        self.state = ResizeState::Preparing;
        self.version = self.version.wrapping_add(1);

        self.events.push(ResizeEvent::ResizeStarted {
            id,
            handle,
            initial_bounds: bounds,
            initial_mouse: mouse_position,
        });

        true
    }

    /// Actualizar durante preparación
    pub fn update_preparing(&mut self, mouse_position: Vec2) -> bool {
        let delta = (mouse_position - self.initial_mouse).length();

        if delta >= self.resize_threshold {
            self.state = ResizeState::Resizing;
            self.current_mouse = mouse_position;
            true
        } else {
            self.current_mouse = mouse_position;
            false
        }
    }

    /// Calcular nuevos límites dado el handle y la posición del mouse
    pub fn calculate_new_bounds(&self, mouse_position: Vec2) -> Rect {
        let bounds = self.initial_bounds;
        let mouse_delta = mouse_position - self.initial_mouse;

        // Determinar dimensiones base según el handle
        let (new_min, new_max) = match self.active_handle {
            Some(HandleType::TopLeft) => {
                let mut new_min = bounds.min + mouse_delta;
                let new_max = bounds.max;
                if !self.constraints.allow_negative {
                    new_min.x = new_min.x.min(new_max.x - self.constraints.min_width);
                    new_min.y = new_min.y.min(new_max.y - self.constraints.min_height);
                }
                (new_min, new_max)
            }
            Some(HandleType::TopCenter) => {
                let mut new_min_y = bounds.min.y + mouse_delta.y;
                if !self.constraints.allow_negative {
                    new_min_y = new_min_y.min(bounds.max.y - self.constraints.min_height);
                }
                (Vec2::new(bounds.min.x, new_min_y), bounds.max)
            }
            Some(HandleType::TopRight) => {
                let mut new_max = bounds.max + mouse_delta;
                let new_min = bounds.min;
                if !self.constraints.allow_negative {
                    new_max.x = new_max.x.max(new_min.x + self.constraints.min_width);
                    new_max.y = new_max.y.max(new_min.y + self.constraints.min_height);
                }
                (new_min, new_max)
            }
            Some(HandleType::CenterLeft) => {
                let mut new_min_x = bounds.min.x + mouse_delta.x;
                if !self.constraints.allow_negative {
                    new_min_x = new_min_x.min(bounds.max.x - self.constraints.min_width);
                }
                (Vec2::new(new_min_x, bounds.min.y), bounds.max)
            }
            Some(HandleType::CenterRight) => {
                let mut new_max_x = bounds.max.x + mouse_delta.x;
                if !self.constraints.allow_negative {
                    new_max_x = new_max_x.max(bounds.min.x + self.constraints.min_width);
                }
                (bounds.min, Vec2::new(new_max_x, bounds.max.y))
            }
            Some(HandleType::BottomLeft) => {
                let mut new_min = bounds.min + mouse_delta;
                let new_max = bounds.max;
                if !self.constraints.allow_negative {
                    new_min.x = new_min.x.min(new_max.x - self.constraints.min_width);
                    new_min.y = new_min.y.min(new_max.y - self.constraints.min_height);
                }
                (new_min, new_max)
            }
            Some(HandleType::BottomCenter) => {
                let mut new_max_y = bounds.max.y + mouse_delta.y;
                if !self.constraints.allow_negative {
                    new_max_y = new_max_y.max(bounds.min.y + self.constraints.min_height);
                }
                (bounds.min, Vec2::new(bounds.max.x, new_max_y))
            }
            Some(HandleType::BottomRight) => {
                let mut new_max = bounds.max + mouse_delta;
                let new_min = bounds.min;
                if !self.constraints.allow_negative {
                    new_max.x = new_max.x.max(new_min.x + self.constraints.min_width);
                    new_max.y = new_max.y.max(new_min.y + self.constraints.min_height);
                }
                (new_min, new_max)
            }
            _ => (bounds.min, bounds.max),
        };

        let raw_width = new_max.x - new_min.x;
        let raw_height = new_max.y - new_min.y;

        // Aplicar aspecto ratio
        let (width, height) = self.aspect_mode.apply_aspect(raw_width, raw_height, bounds);

        // Crear nuevos límites con el modo de centrado
        let center = bounds.center();
        let new_min = match self.center_mode {
            ResizeCenterMode::Keep => Vec2::new(center.x - width / 2.0, center.y - height / 2.0),
            ResizeCenterMode::KeepTopLeft => new_min,
            ResizeCenterMode::KeepBottomLeft => Vec2::new(center.x - width / 2.0, new_min.y),
            ResizeCenterMode::KeepTopRight => Vec2::new(new_max.x - width, center.y - height / 2.0),
            ResizeCenterMode::KeepBottomRight => new_max - Vec2::new(width, height),
            ResizeCenterMode::CenterOnMouse => Vec2::new(
                mouse_position.x - width / 2.0,
                mouse_position.y - height / 2.0,
            ),
            ResizeCenterMode::Free => new_min,
        };

        let new_bounds = Rect::from_min_max(new_min, new_min + Vec2::new(width, height));

        // Aplicar restricciones finales
        let (w, h) = self
            .constraints
            .validate(new_bounds.width(), new_bounds.height());
        let adjusted_min = match self.center_mode {
            ResizeCenterMode::Keep | ResizeCenterMode::Free => new_bounds.min,
            ResizeCenterMode::KeepTopLeft => new_bounds.min,
            _ => new_bounds.center() - Vec2::new(w / 2.0, h / 2.0),
        };

        Rect::from_pos_size(adjusted_min, Vec2::new(w, h))
    }

    /// Actualizar durante resize activo
    pub fn update_resize(&mut self, mouse_position: Vec2) -> ResizeEvent {
        self.current_mouse = mouse_position;

        if let (Some(id), Some(handle)) = (self.active_id, self.active_handle) {
            let new_bounds = self.calculate_new_bounds(mouse_position);

            let event = ResizeEvent::Resizing {
                id,
                handle,
                initial_bounds: self.initial_bounds,
                initial_mouse: self.initial_mouse,
                current_mouse: mouse_position,
                new_bounds,
            };

            self.events.push(event.clone());
            event
        } else {
            ResizeEvent::Resizing {
                id: self.active_id.unwrap_or_else(EntityId::new),
                handle: self.active_handle.unwrap_or(HandleType::BottomRight),
                initial_bounds: self.initial_bounds,
                initial_mouse: self.initial_mouse,
                current_mouse: mouse_position,
                new_bounds: self.initial_bounds,
            }
        }
    }

    /// Finalizar resize
    pub fn end_resize(&mut self) -> Option<ResizeEvent> {
        let result = if let (Some(id), Some(handle)) = (self.active_id, self.active_handle) {
            let new_bounds = self.calculate_new_bounds(self.current_mouse);

            let event = ResizeEvent::ResizeEnded {
                id,
                handle,
                initial_bounds: self.initial_bounds,
                final_bounds: new_bounds,
            };
            self.events.push(event.clone());
            Some(event)
        } else {
            None
        };

        self.state = ResizeState::Idle;
        self.active_id = None;
        self.active_handle = None;
        self.version = self.version.wrapping_add(1);

        result
    }

    /// Cancelar resize
    pub fn cancel_resize(&mut self) -> Option<ResizeEvent> {
        let result = if let (Some(id), Some(handle)) = (self.active_id, self.active_handle) {
            let event = ResizeEvent::ResizeCancelled {
                id,
                handle,
                initial_bounds: self.initial_bounds,
            };
            self.events.push(event.clone());
            Some(event)
        } else {
            None
        };

        self.state = ResizeState::Idle;
        self.active_id = None;
        self.active_handle = None;
        self.version = self.version.wrapping_add(1);

        result
    }

    /// Hit test para encontrar el handle bajo el mouse
    pub fn hit_test_handle(
        &self,
        point: Vec2,
        bounds: Rect,
        handle_size: f32,
    ) -> Option<HandleType> {
        let half_size = handle_size / 2.0;

        // Verificar cada handle
        for handle in HandleType::bounding_box_handles() {
            let pos = handle.position(bounds);
            let handle_rect =
                Rect::from_pos_size(pos - Vec2::splat(half_size), Vec2::splat(handle_size));

            if handle_rect.contains(point) {
                return Some(*handle);
            }
        }

        None
    }

    /// Obtener eventos acumulados
    #[inline]
    pub fn events(&self) -> &[ResizeEvent] {
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
}

impl Default for ResizeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Modo de centrado durante resize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeCenterMode {
    /// Mantener centro original
    Keep,
    /// Mantener esquina superior izquierda
    KeepTopLeft,
    /// Mantener esquina inferior izquierda
    KeepBottomLeft,
    /// Mantener esquina superior derecha
    KeepTopRight,
    /// Mantener esquina inferior derecha
    KeepBottomRight,
    /// Centrar en la posición del mouse
    CenterOnMouse,
    /// Sin restricciones de centrado (resize libre)
    Free,
}

impl Default for ResizeCenterMode {
    fn default() -> Self {
        ResizeCenterMode::Keep
    }
}

/// Builder para ResizeManager
pub struct ResizeManagerBuilder {
    aspect_mode: Option<AspectRatioMode>,
    constraints: Option<SizeConstraints>,
    feedback_config: Option<ResizeFeedbackConfig>,
    center_mode: Option<ResizeCenterMode>,
}

impl ResizeManagerBuilder {
    /// Create new builder
    #[inline]
    pub fn new() -> Self {
        Self {
            aspect_mode: None,
            constraints: None,
            feedback_config: None,
            center_mode: None,
        }
    }

    /// Set aspect ratio mode
    #[inline]
    pub fn aspect_mode(mut self, aspect_mode: AspectRatioMode) -> Self {
        self.aspect_mode = Some(aspect_mode);
        self
    }

    /// Set size constraints
    #[inline]
    pub fn constraints(mut self, constraints: SizeConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }

    /// Set feedback configuration
    #[inline]
    pub fn feedback_config(mut self, feedback_config: ResizeFeedbackConfig) -> Self {
        self.feedback_config = Some(feedback_config);
        self
    }

    /// Set center mode
    #[inline]
    pub fn center_mode(mut self, center_mode: ResizeCenterMode) -> Self {
        self.center_mode = Some(center_mode);
        self
    }

    /// Build ResizeManager
    #[inline]
    pub fn build(self) -> ResizeManager {
        let mut manager = ResizeManager::new();

        if let Some(aspect) = self.aspect_mode {
            manager.aspect_mode = aspect;
        }

        if let Some(constraints) = self.constraints {
            manager.constraints = constraints;
        }

        if let Some(config) = self.feedback_config {
            manager.feedback_config = config;
        }

        if let Some(center) = self.center_mode {
            manager.center_mode = center;
        }

        manager
    }
}

impl Default for ResizeManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResizable {
        id: EntityId,
        bounds: Rect,
    }

    impl TestResizable {
        fn new(id: u128, x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                id: EntityId::from_u128(id),
                bounds: Rect::from_pos_size(Vec2::new(x, y), Vec2::new(width, height)),
            }
        }
    }

    impl Resizable for TestResizable {
        fn id(&self) -> EntityId {
            self.id
        }

        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn set_bounds(&mut self, bounds: Rect) {
            self.bounds = bounds;
        }

        fn transform(&self) -> Transform {
            Transform::identity()
        }

        fn apply_transform(&mut self, _transform: Transform) {}
    }

    #[test]
    fn test_resize_manager_new() {
        let manager = ResizeManager::new();

        assert_eq!(manager.state(), ResizeState::Idle);
        assert!(manager.active_id().is_none());
        assert!(!manager.is_resizing());
    }

    #[test]
    fn test_start_preparing() {
        let mut manager = ResizeManager::new();
        let id = EntityId::from_u128(1);
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));
        let mouse = Vec2::new(150.0, 115.0);

        assert!(manager.start_preparing(id, HandleType::BottomRight, bounds, mouse));
        assert_eq!(manager.state(), ResizeState::Preparing);
        assert_eq!(manager.active_id(), Some(id));
        assert_eq!(manager.active_handle(), Some(HandleType::BottomRight));
        assert_eq!(manager.initial_bounds(), bounds);
    }

    #[test]
    fn test_update_preparing_below_threshold() {
        let mut manager = ResizeManager::new();
        let id = EntityId::from_u128(1);
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(id, HandleType::BottomRight, bounds, Vec2::new(150.0, 130.0));

        // Mover solo 1 pixel (menos que el threshold de 2)
        assert!(!manager.update_preparing(Vec2::new(150.5, 130.5)));
        assert_eq!(manager.state(), ResizeState::Preparing);
    }

    #[test]
    fn test_update_preparing_above_threshold() {
        let mut manager = ResizeManager::new();
        let id = EntityId::from_u128(1);
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(id, HandleType::BottomRight, bounds, Vec2::new(150.0, 130.0));

        // Mover 3 pixels (más que el threshold de 2)
        assert!(manager.update_preparing(Vec2::new(153.0, 133.0)));
        assert_eq!(manager.state(), ResizeState::Resizing);
    }

    #[test]
    fn test_calculate_new_bounds_bottom_right() {
        let mut manager = ResizeManager::new();
        // Use KeepTopLeft to preserve the corner position during resize
        manager.center_mode = ResizeCenterMode::KeepTopLeft;
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(
            EntityId::from_u128(1),
            HandleType::BottomRight,
            bounds,
            Vec2::new(150.0, 130.0),
        );
        manager.update_preparing(Vec2::new(200.0, 150.0));

        let new_bounds = manager.calculate_new_bounds(Vec2::new(200.0, 150.0));

        // With KeepTopLeft, the min corner stays at (100, 100)
        assert_eq!(new_bounds.min, Vec2::new(100.0, 100.0));
        // The width and height should increase based on mouse movement
        assert!(new_bounds.width() > 50.0);
        assert!(new_bounds.height() > 30.0);
    }

    #[test]
    fn test_aspect_ratio_keep() {
        let mut manager = ResizeManager::new();
        manager.aspect_mode = AspectRatioMode::Keep;

        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(100.0, 50.0)); // 2:1

        manager.start_preparing(
            EntityId::from_u128(1),
            HandleType::BottomRight,
            bounds,
            Vec2::new(200.0, 150.0),
        );
        manager.update_preparing(Vec2::new(300.0, 200.0));

        let new_bounds = manager.calculate_new_bounds(Vec2::new(300.0, 200.0));

        // El aspecto debería mantenerse en 2:1
        let ratio = new_bounds.width() / new_bounds.height();
        assert!((ratio - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_aspect_ratio_fixed() {
        let mut manager = ResizeManager::new();
        manager.aspect_mode = AspectRatioMode::Fixed(1.0); // Cuadrado

        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(100.0, 50.0));

        manager.start_preparing(
            EntityId::from_u128(1),
            HandleType::BottomRight,
            bounds,
            Vec2::new(200.0, 150.0),
        );
        manager.update_preparing(Vec2::new(250.0, 180.0));

        let new_bounds = manager.calculate_new_bounds(Vec2::new(250.0, 180.0));

        // El aspecto debería ser 1:1
        assert!((new_bounds.width() - new_bounds.height()).abs() < 0.001);
    }

    #[test]
    fn test_size_constraints() {
        let mut manager = ResizeManager::new();
        manager.constraints = SizeConstraints::with_size(20.0, 20.0, 200.0, 200.0);

        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(
            EntityId::from_u128(1),
            HandleType::BottomRight,
            bounds,
            Vec2::new(150.0, 130.0),
        );
        manager.update_preparing(Vec2::new(50.0, 50.0)); // Intentando hacer muy pequeño

        let new_bounds = manager.calculate_new_bounds(Vec2::new(50.0, 50.0));

        // Debe respetar el mínimo
        assert!(new_bounds.width() >= 20.0);
        assert!(new_bounds.height() >= 20.0);
    }

    #[test]
    fn test_hit_test_handle() {
        let manager = ResizeManager::new();
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(100.0, 50.0));
        let handle_size = 10.0;

        // Corner bottom-right
        let result = manager.hit_test_handle(Vec2::new(199.0, 149.0), bounds, handle_size);
        assert_eq!(result, Some(HandleType::BottomRight));

        // Corner top-left
        let result = manager.hit_test_handle(Vec2::new(101.0, 101.0), bounds, handle_size);
        assert_eq!(result, Some(HandleType::TopLeft));

        // Center - no debería detectar handle
        let result = manager.hit_test_handle(Vec2::new(150.0, 125.0), bounds, handle_size);
        assert!(result.is_none());
    }

    #[test]
    fn test_end_resize() {
        let mut manager = ResizeManager::new();
        let id = EntityId::from_u128(1);
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(id, HandleType::BottomRight, bounds, Vec2::new(150.0, 130.0));
        manager.update_preparing(Vec2::new(200.0, 150.0));
        manager.update_resize(Vec2::new(210.0, 160.0));

        let event = manager.end_resize();

        assert!(event.is_some());
        if let ResizeEvent::ResizeEnded {
            initial_bounds,
            final_bounds,
            ..
        } = event.unwrap()
        {
            assert_eq!(initial_bounds, bounds);
            assert_ne!(final_bounds, bounds);
        }

        assert_eq!(manager.state(), ResizeState::Idle);
    }

    #[test]
    fn test_cancel_resize() {
        let mut manager = ResizeManager::new();
        let id = EntityId::from_u128(1);
        let bounds = Rect::from_pos_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));

        manager.start_preparing(id, HandleType::BottomRight, bounds, Vec2::new(150.0, 130.0));
        manager.update_preparing(Vec2::new(200.0, 150.0));

        let event = manager.cancel_resize();

        assert!(event.is_some());
        if let ResizeEvent::ResizeCancelled { initial_bounds, .. } = event.unwrap() {
            assert_eq!(initial_bounds, bounds);
        }

        assert_eq!(manager.state(), ResizeState::Idle);
    }

    #[test]
    fn test_builder() {
        let manager = ResizeManagerBuilder::new()
            .aspect_mode(AspectRatioMode::Square)
            .constraints(SizeConstraints::with_min_size(50.0, 50.0))
            .center_mode(ResizeCenterMode::Free)
            .build();

        assert_eq!(manager.aspect_mode(), &AspectRatioMode::Square);
        assert_eq!(manager.constraints().min_width, 50.0);
        assert_eq!(manager.constraints().min_height, 50.0);
    }

    #[test]
    fn test_resizable_trait() {
        let mut resizable = TestResizable::new(1, 100.0, 100.0, 50.0, 30.0);

        assert_eq!(resizable.id(), EntityId::from_u128(1));
        assert_eq!(resizable.bounds().width(), 50.0);
        assert_eq!(resizable.bounds().height(), 30.0);

        resizable.set_bounds(Rect::from_pos_size(
            Vec2::new(200.0, 200.0),
            Vec2::new(100.0, 60.0),
        ));
        assert_eq!(resizable.bounds().width(), 100.0);
        assert_eq!(resizable.bounds().height(), 60.0);
    }

    #[test]
    fn test_aspect_ratio_mode() {
        assert!(
            AspectRatioMode::Free
                .forced_aspect(Rect::default())
                .is_none()
        );
        assert!(
            AspectRatioMode::Keep
                .forced_aspect(Rect::from_pos_size(Vec2::ZERO, Vec2::new(100.0, 50.0)))
                .unwrap()
                - 2.0
                < 0.001
        );
        assert_eq!(
            AspectRatioMode::Fixed(1.5)
                .forced_aspect(Rect::default())
                .unwrap(),
            1.5
        );
        assert_eq!(
            AspectRatioMode::FixedSize(100.0, 50.0)
                .forced_aspect(Rect::default())
                .unwrap(),
            2.0
        );
        assert_eq!(
            AspectRatioMode::Square
                .forced_aspect(Rect::default())
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_size_constraints_validate() {
        let constraints = SizeConstraints::with_size(20.0, 20.0, 100.0, 100.0);

        // Dentro de límites
        let (w, h) = constraints.validate(50.0, 50.0);
        assert_eq!((w, h), (50.0, 50.0));

        // Por debajo del mínimo
        let (w, h) = constraints.validate(10.0, 10.0);
        assert_eq!((w, h), (20.0, 20.0));

        // Por encima del máximo
        let (w, h) = constraints.validate(150.0, 150.0);
        assert_eq!((w, h), (100.0, 100.0));
    }
}
