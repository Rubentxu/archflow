//! ArchFlow ECS - Integración con bevy_ecs 0.18
//!
//! Este crate proporciona:
//! - Componentes para integración con SpatialIndex
//! - Recursos compartidos para gestión espacial
//!
//! # Nota
//! Los sistemas ECS completos requieren que `Transform` implemente `Component`.
//! Esto se puede lograr añadiendo `#[derive(Component)]` a `archflow_core::Transform`.

pub use bevy_ecs::prelude::*;

// Re-export de tipos de geometry para conveniencia
pub use archflow_geometry::SpatialIndex;

// Re-export de tipos de core
pub use archflow_core::{EntityId, Rect, Vec2};

/// Componente que almacena los bounds locales (AABB) de una entidad.
///
/// Este componente define el tamaño de la entidad en su espacio local.
/// El sistema de sincronización convertirá esto a bounds globales
/// usando el `Transform` de la entidad.
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct SpatialBounds {
    /// Dimensiones de la entidad (ancho, alto)
    pub size: Vec2,
    /// Offset desde el origen del transform
    pub offset: Vec2,
}

impl SpatialBounds {
    /// Crear bounds con tamaño
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            offset: Vec2::ZERO,
        }
    }

    /// Crear bounds con tamaño y offset
    pub fn with_offset(size: Vec2, offset: Vec2) -> Self {
        Self { size, offset }
    }

    /// Obtener el AABB local
    pub fn local_aabb(&self) -> Rect {
        Rect::from_center_size(self.offset, self.size)
    }
}

impl Default for SpatialBounds {
    fn default() -> Self {
        Self::new(Vec2::new(100.0, 100.0))
    }
}

/// Componente que almacena los bounds globales calculados.
///
/// Este componente es mantenido automáticamente por el sistema
/// de sincronización y contiene el AABB de la entidad
/// en coordenadas globales (después de aplicar el Transform).
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct GlobalBounds {
    /// Rectángulo delimitador global
    pub bounds: Rect,
}

impl GlobalBounds {
    /// Crear con bounds
    pub fn new(bounds: Rect) -> Self {
        Self { bounds }
    }
}

impl Default for GlobalBounds {
    fn default() -> Self {
        Self::new(Rect::default())
    }
}

/// Recurso que contiene el SpatialIndex compartido.
///
/// Este recurso es accedido por el sistema de sincronización
/// y puede ser consultado por otros sistemas para operaciones
/// espaciales (picking, culling, etc.).
#[derive(Resource)]
pub struct SpatialResource {
    /// El índice espacial
    pub index: SpatialIndex,
    /// Versión para detectar cambios externos
    version: u64,
}

impl Default for SpatialResource {
    fn default() -> Self {
        Self::new()
    }
}

impl SpatialResource {
    /// Crear nuevo recurso espacial
    pub fn new() -> Self {
        Self {
            index: SpatialIndex::new(),
            version: 0,
        }
    }

    /// Obtener versión actual
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Incrementar versión (para notificar cambios)
    pub fn increment_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    /// Obtener referencia al índice
    pub fn index(&self) -> &SpatialIndex {
        &self.index
    }

    /// Obtener referencia mutable al índice
    pub fn index_mut(&mut self) -> &mut SpatialIndex {
        self.increment_version();
        &mut self.index
    }
}

/// Transform de ECS - Wrapper que implementa Component
///
/// Este es un wrapper alrededor de `archflow_core::Transform`
/// que añade la derivación `Component` de bevy_ecs.
/// Usar este tipo en lugar de `archflow_core::Transform` para
/// entidades que necesitan sincronización espacial.
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct Transform {
    /// Posición local (translation)
    pub translation: Vec2,
    /// Rotación en radianes
    pub rotation: f32,
    /// Escala (1.0 = sin escala)
    pub scale: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform {
    /// Crear transform identidad
    pub fn identity() -> Self {
        Self::default()
    }

    /// Crear con posición
    pub fn from_translation(x: f32, y: f32) -> Self {
        Self {
            translation: Vec2::new(x, y),
            ..Default::default()
        }
    }

    /// Convertir desde archflow_core::Transform
    pub fn from_core(transform: &archflow_core::Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    /// Convertir a archflow_core::Transform
    pub fn to_core(&self) -> archflow_core::Transform {
        archflow_core::Transform {
            translation: self.translation,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

/// Calcular el AABB global de una entidad.
///
/// Aplica la transformación (posición, rotación, escala) a los
/// bounds locales para obtener el rectángulo delimitador global.
///
/// Esta función es útil fuera del contexto de ECS para calcular
/// bounds transformados.
pub fn calculate_global_aabb(transform: &Transform, bounds: &SpatialBounds) -> Rect {
    let local_aabb = bounds.local_aabb();

    // Aplicar escala a las dimensiones (multiplicación componente a componente)
    let scaled_half_size = Vec2::new(
        local_aabb.size().x * transform.scale.x * 0.5,
        local_aabb.size().y * transform.scale.y * 0.5,
    );
    let scaled_offset = Vec2::new(
        bounds.offset.x * transform.scale.x,
        bounds.offset.y * transform.scale.y,
    );

    // Calcular las 4 esquinas del rectángulo escalado
    let corners = [
        Vec2::new(-scaled_half_size.x, -scaled_half_size.y),
        Vec2::new(scaled_half_size.x, -scaled_half_size.y),
        Vec2::new(scaled_half_size.x, scaled_half_size.y),
        Vec2::new(-scaled_half_size.x, scaled_half_size.y),
    ];

    // Aplicar rotación
    let cos_r = transform.rotation.cos();
    let sin_r = transform.rotation.sin();
    let rotated_corners: [Vec2; 4] =
        corners.map(|c| Vec2::new(c.x * cos_r - c.y * sin_r, c.x * sin_r + c.y * cos_r));

    // Aplicar traslación y offset
    let translated_corners: [Vec2; 4] =
        rotated_corners.map(|c| c + transform.translation + scaled_offset);

    // Calcular AABB de las esquinas transformadas
    let min_x = translated_corners
        .iter()
        .map(|c| c.x)
        .fold(f32::MAX, f32::min);
    let min_y = translated_corners
        .iter()
        .map(|c| c.y)
        .fold(f32::MAX, f32::min);
    let max_x = translated_corners
        .iter()
        .map(|c| c.x)
        .fold(f32::MIN, f32::max);
    let max_y = translated_corners
        .iter()
        .map(|c| c.y)
        .fold(f32::MIN, f32::max);

    Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

/// Sistema de sincronización espacial.
///
/// **Nota:** Este sistema requiere que las entidades tengan:
/// - Componente `Transform` (este wrapper, no `archflow_core::Transform`)
/// - Componente `SpatialBounds`
///
/// El sistema detecta cambios y actualiza el SpatialIndex.
pub fn spatial_sync_system(
    mut resource: ResMut<SpatialResource>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &SpatialBounds,
            Option<&mut GlobalBounds>,
        ),
        Or<(Changed<Transform>, Changed<SpatialBounds>)>,
    >,
) {
    let index = resource.index_mut();

    for (entity, transform, spatial_bounds, global_bounds) in query.iter_mut() {
        let global_aabb = calculate_global_aabb(transform, spatial_bounds);

        // Actualizar SpatialIndex
        // EntityIndex contiene un u32 internamente, lo convertimos directamente
        let idx = entity.index();
        let idx_u32: u32 = unsafe { std::mem::transmute(idx) };
        let item =
            archflow_geometry::SpatialItem::new(EntityId::from_u128(idx_u32 as u128), global_aabb);
        index.insert(item);

        // Actualizar GlobalBounds si existe el componente
        if let Some(mut gb) = global_bounds {
            gb.bounds = global_aabb;
        }
    }
}

/// Configuración del sistema de sincronización espacial
#[derive(Debug, Clone, Resource)]
pub struct SpatialSyncConfig {
    /// Habilitar sincronización
    pub enabled: bool,
    /// Query batch size
    pub batch_size: usize,
}

impl Default for SpatialSyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_bounds_local_aabb() {
        let bounds = SpatialBounds::new(Vec2::new(100.0, 50.0));
        let aabb = bounds.local_aabb();

        assert_eq!(aabb.size(), Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_calculate_global_aabb_identity() {
        let transform = Transform::default();
        let bounds = SpatialBounds::new(Vec2::new(100.0, 50.0));

        let global_aabb = calculate_global_aabb(&transform, &bounds);

        // Sin transformación, el AABB debería estar centrado en el origen
        assert_eq!(global_aabb.min, Vec2::new(-50.0, -25.0));
        assert_eq!(global_aabb.max, Vec2::new(50.0, 25.0));
    }

    #[test]
    fn test_calculate_global_aabb_translation() {
        let transform = Transform::from_translation(100.0, 200.0);
        let bounds = SpatialBounds::new(Vec2::new(100.0, 50.0));

        let global_aabb = calculate_global_aabb(&transform, &bounds);

        // El AABB debería estar desplazado
        assert_eq!(global_aabb.min, Vec2::new(50.0, 175.0));
        assert_eq!(global_aabb.max, Vec2::new(150.0, 225.0));
    }

    #[test]
    fn test_spatial_resource_default() {
        let resource = SpatialResource::new();
        assert!(resource.index().is_empty());
        assert_eq!(resource.version(), 0);
    }

    #[test]
    fn test_transform_from_core() {
        let core_transform = archflow_core::Transform {
            translation: Vec2::new(10.0, 20.0),
            rotation: std::f32::consts::PI / 2.0,
            scale: Vec2::new(2.0, 2.0),
        };

        let transform = Transform::from_core(&core_transform);

        assert_eq!(transform.translation, Vec2::new(10.0, 20.0));
        assert!((transform.rotation - std::f32::consts::PI / 2.0).abs() < 1e-6);
        assert_eq!(transform.scale, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_transform_roundtrip() {
        let original = archflow_core::Transform {
            translation: Vec2::new(42.0, 24.0),
            rotation: 0.5,
            scale: Vec2::new(1.5, 0.5),
        };

        let ecs_transform = Transform::from_core(&original);
        let back = ecs_transform.to_core();

        assert_eq!(original.translation, back.translation);
        assert!((original.rotation - back.rotation).abs() < 1e-6);
        assert_eq!(original.scale, back.scale);
    }
}
