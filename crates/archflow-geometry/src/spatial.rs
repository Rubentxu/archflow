//! ArchFlow Spatial Index - R-Tree implementation for spatial queries
//!
//! Provides:
//! - R-Tree spatial index for efficient queries
//! - Viewport, point, and area queries
//! - Integration with ECS components
//! - Dirty tracking for updates

use archflow_core::{EntityId, Rect, Vec2};
use rstar::{RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Elemento espacial almacenado en el R-Tree
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialItem {
    /// ID de la entidad
    pub id: EntityId,
    /// Rectángulo delimitador
    pub bounds: Rect,
    /// Datos adicionales opcionales
    pub data: Option<SpatialData>,
}

impl SpatialItem {
    /// Crear nuevo item espacial
    pub fn new(id: EntityId, bounds: Rect) -> Self {
        Self {
            id,
            bounds,
            data: None,
        }
    }

    /// Crear con datos adicionales
    pub fn with_data(id: EntityId, bounds: Rect, data: SpatialData) -> Self {
        Self {
            id,
            bounds,
            data: Some(data),
        }
    }
}

/// Implementación de RTreeObject para SpatialItem
impl RTreeObject for SpatialItem {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.bounds.min.x, self.bounds.min.y],
            [self.bounds.max.x, self.bounds.max.y],
        )
    }
}

/// Datos adicionales para un item espacial
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialData {
    /// Tipo de entidad
    pub entity_type: String,
    /// Prioridad de renderizado
    pub render_priority: i32,
    /// Flags adicionales
    pub flags: u32,
}

impl SpatialData {
    /// Crear nuevos datos espaciales
    pub fn new(entity_type: &str, render_priority: i32) -> Self {
        Self {
            entity_type: entity_type.to_string(),
            render_priority,
            flags: 0,
        }
    }

    /// Agregar flag
    pub fn with_flag(mut self, flag: u32) -> Self {
        self.flags |= flag;
        self
    }
}

/// Configuración del índice espacial
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialIndexConfig {
    /// Capacidad del árbol (número de hijos por nodo)
    pub capacity: usize,
    /// Factor de rozamiento para rebalanceo
    pub reinsertion_factor: f32,
    /// Distancia mínima para considerar cercanas las hojas
    pub overlap_tolerance: f32,
    /// Habilitar bulk loading
    pub bulk_loading: bool,
}

impl Default for SpatialIndexConfig {
    fn default() -> Self {
        Self {
            capacity: 16,
            reinsertion_factor: 0.3,
            overlap_tolerance: 0.01,
            bulk_loading: true,
        }
    }
}

/// Resultado de una consulta espacial
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialQueryResult {
    /// Items encontrados
    pub items: Vec<SpatialItem>,
    /// Tiempo de consulta en microsegundos
    pub query_time_us: u128,
    /// Número de nodos visitados
    pub nodes_visited: u32,
}

/// Tipo de consulta espacial
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpatialQueryType {
    /// Obtener todos los items
    All,
    /// Consultar por viewport
    Viewport,
    /// Consultar por punto
    Point,
    /// Consultar por rectángulo
    Rectangle,
    /// Consultar por círculo
    Circle { radius: f32 },
    /// Consultar intersecting
    Intersecting { bounds: Rect },
}

/// Configuración de consulta
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialQueryConfig {
    /// Tipo de consulta
    pub query_type: SpatialQueryType,
    /// Incluir items parcialmente visibles
    pub include_partial: bool,
    /// Máximo número de resultados (0 = sin límite)
    pub max_results: usize,
    /// Orden de resultados
    pub order: SpatialQueryOrder,
    /// Filtro por tipo de entidad
    pub entity_type_filter: Option<HashSet<String>>,
}

/// Orden de resultados
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialQueryOrder {
    /// Sin orden específico
    None,
    /// Por distancia ascendente
    DistanceAsc,
    /// Por distancia descendente
    DistanceDesc,
    /// Por prioridad de renderizado
    Priority,
    /// Por área ascendente
    AreaAsc,
    /// Por área descendente
    AreaDesc,
}

impl Default for SpatialQueryConfig {
    fn default() -> Self {
        Self {
            query_type: SpatialQueryType::All,
            include_partial: true,
            max_results: 0,
            order: SpatialQueryOrder::None,
            entity_type_filter: None,
        }
    }
}

/// Estado del dirty tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyState {
    /// Limpio
    Clean,
    /// Modificado
    Dirty,
    /// Eliminado
    Deleted,
}

/// Registro de dirty tracking
#[derive(Debug, Clone, PartialEq)]
pub struct DirtyRecord {
    /// ID de la entidad
    pub id: EntityId,
    /// Estado anterior
    pub previous_bounds: Rect,
    /// Estado actual
    pub current_bounds: Rect,
    /// Timestamp de la modificación
    pub timestamp: std::time::SystemTime,
    /// Tipo de modificación
    pub change_type: DirtyChangeType,
}

/// Tipo de cambio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirtyChangeType {
    /// Movimiento (sin cambio de tamaño)
    Move,
    /// Cambio de tamaño
    Resize,
    /// Movimiento y cambio de tamaño
    Transform,
    /// Creación
    Create,
    /// Eliminación
    Delete,
}

/// Índice espacial principal
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    /// El R-Tree interno
    tree: RTree<SpatialItem>,
    /// Mapa de ID a item (para búsquedas rápidas por ID)
    id_map: HashMap<EntityId, SpatialItem>,
    /// Configuración
    config: SpatialIndexConfig,
    /// Dirty tracking
    dirty_records: Vec<DirtyRecord>,
    /// Set de entidades marcadas como dirty
    dirty_ids: HashSet<EntityId>,
    /// Versión para observers
    version: u64,
    /// Métricas
    metrics: SpatialMetrics,
}

/// Métricas del índice espacial
#[derive(Debug, Clone, Default)]
pub struct SpatialMetrics {
    /// Total de items
    pub total_items: u32,
    /// Total de nodos
    pub total_nodes: u32,
    /// Profundidad del árbol
    pub tree_depth: u32,
    /// Promedio de fill factor
    pub avg_fill_factor: f32,
    /// Último tiempo de query
    pub last_query_time_us: u128,
    /// Total de queries
    pub total_queries: u64,
}

impl SpatialIndex {
    /// Crear nuevo índice espacial
    pub fn new() -> Self {
        Self::with_config(SpatialIndexConfig::default())
    }

    /// Crear con configuración
    pub fn with_config(config: SpatialIndexConfig) -> Self {
        Self {
            tree: RTree::new(),
            id_map: HashMap::new(),
            config,
            dirty_records: Vec::new(),
            dirty_ids: HashSet::new(),
            version: 0,
            metrics: SpatialMetrics::default(),
        }
    }

    /// Insertar un item
    pub fn insert(&mut self, item: SpatialItem) -> bool {
        // Si ya existe, remover primero
        if self.id_map.contains_key(&item.id) {
            self.remove(&item.id);
        }

        let item_id = item.id;
        let item_bounds = item.bounds;

        self.tree.insert(item.clone());
        self.id_map.insert(item_id, item);
        self.mark_dirty(item_id, item_bounds, DirtyChangeType::Create);
        self.version = self.version.wrapping_add(1);

        true
    }

    /// Insertar múltiples items (bulk)
    pub fn insert_bulk(&mut self, items: &[SpatialItem]) -> usize {
        for item in items {
            if !self.id_map.contains_key(&item.id) {
                self.tree.insert(item.clone());
                self.id_map.insert(item.id, item.clone());
            }
        }
        self.version = self.version.wrapping_add(1);
        items.len()
    }

    /// Remover un item por ID
    pub fn remove(&mut self, id: &EntityId) -> Option<SpatialItem> {
        if let Some(item) = self.id_map.remove(id) {
            self.tree.remove(&item);

            self.dirty_records.push(DirtyRecord {
                id: *id,
                previous_bounds: item.bounds,
                current_bounds: Rect::default(),
                timestamp: std::time::SystemTime::now(),
                change_type: DirtyChangeType::Delete,
            });
            self.dirty_ids.insert(*id);

            self.version = self.version.wrapping_add(1);
            Some(item)
        } else {
            None
        }
    }

    /// Actualizar un item existente
    pub fn update(&mut self, id: &EntityId, new_bounds: Rect) -> bool {
        if let Some(old_bounds) = self.get_bounds(id) {
            if old_bounds == new_bounds {
                return false;
            }

            self.remove(id);
            self.insert(SpatialItem::new(*id, new_bounds));

            self.dirty_records.push(DirtyRecord {
                id: *id,
                previous_bounds: old_bounds,
                current_bounds: new_bounds,
                timestamp: std::time::SystemTime::now(),
                change_type: DirtyChangeType::Transform,
            });

            true
        } else {
            false
        }
    }

    /// Obtener bounds de un item
    pub fn get_bounds(&self, id: &EntityId) -> Option<Rect> {
        self.id_map.get(id).map(|item| item.bounds)
    }

    /// Obtener un item por ID
    pub fn get(&self, id: &EntityId) -> Option<SpatialItem> {
        self.id_map.get(id).cloned()
    }

    /// Consultar todos los items
    pub fn query_all(&self) -> SpatialQueryResult {
        let start_time = std::time::Instant::now();
        let items: Vec<SpatialItem> = self.tree.iter().cloned().collect();
        let query_time = start_time.elapsed().as_micros();

        SpatialQueryResult {
            items,
            query_time_us: query_time,
            nodes_visited: self.tree.size() as u32,
        }
    }

    /// Convertir Rect a AABB para consultas
    fn rect_to_aabb(rect: &Rect) -> AABB<[f32; 2]> {
        AABB::from_corners([rect.min.x, rect.min.y], [rect.max.x, rect.max.y])
    }

    /// Consultar por viewport
    pub fn query_viewport(&self, viewport: Rect, config: SpatialQueryConfig) -> SpatialQueryResult {
        let start_time = std::time::Instant::now();
        let viewport_aabb = Self::rect_to_aabb(&viewport);

        let items: Vec<SpatialItem> = self
            .tree
            .locate_in_envelope_intersecting(&viewport_aabb)
            .filter(|item| {
                // Verificar filtro de tipo
                if let Some(ref filter) = config.entity_type_filter {
                    if let Some(ref data) = item.data {
                        if !filter.contains(&data.entity_type) {
                            return false;
                        }
                    }
                }

                // Verificar visibilidad parcial
                if config.include_partial {
                    viewport.intersects(&item.bounds)
                } else {
                    viewport.contains_rect(&item.bounds)
                }
            })
            .cloned()
            .take(if config.max_results > 0 {
                config.max_results
            } else {
                usize::MAX
            })
            .collect();

        // Ordenar resultados
        let mut items = match config.order {
            SpatialQueryOrder::None => items,
            SpatialQueryOrder::DistanceAsc => {
                let center = viewport.center();
                let mut sorted: Vec<_> = items
                    .iter()
                    .map(|item| {
                        let dist = (item.bounds.center() - center).length();
                        (dist, item)
                    })
                    .collect();
                sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                sorted.into_iter().map(|(_, i)| i.clone()).collect()
            }
            SpatialQueryOrder::DistanceDesc => {
                let center = viewport.center();
                let mut sorted: Vec<_> = items
                    .iter()
                    .map(|item| {
                        let dist = (item.bounds.center() - center).length();
                        (dist, item)
                    })
                    .collect();
                sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                sorted.into_iter().map(|(_, i)| i.clone()).collect()
            }
            SpatialQueryOrder::Priority => {
                let mut sorted: Vec<_> = items
                    .iter()
                    .map(|item| {
                        let priority = item.data.as_ref().map(|d| d.render_priority).unwrap_or(0);
                        (priority, item)
                    })
                    .collect();
                sorted.sort_by(|a, b| b.0.cmp(&a.0));
                sorted.into_iter().map(|(_, i)| i.clone()).collect()
            }
            SpatialQueryOrder::AreaAsc => {
                let mut sorted: Vec<_> = items
                    .iter()
                    .map(|item| (item.bounds.area(), item))
                    .collect();
                sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                sorted.into_iter().map(|(_, i)| i.clone()).collect()
            }
            SpatialQueryOrder::AreaDesc => {
                let mut sorted: Vec<_> = items
                    .iter()
                    .map(|item| (item.bounds.area(), item))
                    .collect();
                sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                sorted.into_iter().map(|(_, i)| i.clone()).collect()
            }
        };

        let query_time = start_time.elapsed().as_micros();

        SpatialQueryResult {
            items,
            query_time_us: query_time,
            nodes_visited: self.tree.size() as u32,
        }
    }

    /// Consultar por punto
    pub fn query_point(&self, point: Vec2, config: SpatialQueryConfig) -> SpatialQueryResult {
        let start_time = std::time::Instant::now();
        let point_aabb = AABB::from_point([point.x, point.y]);

        let items: Vec<SpatialItem> = self
            .tree
            .locate_in_envelope_intersecting(&point_aabb)
            .filter(|item| item.bounds.contains(point))
            .filter(|item| {
                if let Some(ref filter) = config.entity_type_filter {
                    if let Some(ref data) = item.data {
                        if !filter.contains(&data.entity_type) {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .take(if config.max_results > 0 {
                config.max_results
            } else {
                usize::MAX
            })
            .collect();

        let query_time = start_time.elapsed().as_micros();

        SpatialQueryResult {
            items,
            query_time_us: query_time,
            nodes_visited: self.tree.size() as u32,
        }
    }

    /// Consultar por área
    pub fn query_area(&self, area: Rect, config: SpatialQueryConfig) -> SpatialQueryResult {
        self.query_viewport(area, config)
    }

    /// Obtener todos los IDs
    pub fn all_ids(&self) -> Vec<EntityId> {
        self.id_map.keys().cloned().collect()
    }

    /// Verificar si contiene un ID
    pub fn contains(&self, id: &EntityId) -> bool {
        self.id_map.contains_key(id)
    }

    /// Obtener número de items
    pub fn len(&self) -> usize {
        self.tree.size()
    }

    /// Verificar si está vacío
    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }

    /// Marcar como dirty
    fn mark_dirty(&mut self, id: EntityId, bounds: Rect, change_type: DirtyChangeType) {
        self.dirty_ids.insert(id);
        self.dirty_records.push(DirtyRecord {
            id,
            previous_bounds: Rect::default(),
            current_bounds: bounds,
            timestamp: std::time::SystemTime::now(),
            change_type,
        });
    }

    /// Obtener registros dirty
    pub fn dirty_records(&self) -> &[DirtyRecord] {
        &self.dirty_records
    }

    /// Obtener IDs dirty
    pub fn dirty_ids(&self) -> impl Iterator<Item = &EntityId> {
        self.dirty_ids.iter()
    }

    /// Limpiar estado dirty
    pub fn clear_dirty(&mut self) {
        self.dirty_ids.clear();
    }

    /// Limpiar todos los registros dirty
    pub fn flush_dirty_records(&mut self) {
        self.dirty_records.clear();
    }

    /// Obtener versión
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Obtener métricas
    pub fn metrics(&self) -> &SpatialMetrics {
        &self.metrics
    }

    /// Construir desde items (bulk loading)
    pub fn build_from(items: &[SpatialItem]) -> Self {
        let mut index = Self::new();
        index.insert_bulk(items);
        index
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Extensión de Rect para consultas espaciales
pub trait RectExtensions {
    fn contains_rect(&self, other: &Rect) -> bool;
    fn intersects(&self, other: &Rect) -> bool;
    fn area(&self) -> f32;
}

impl RectExtensions for Rect {
    fn contains_rect(&self, other: &Rect) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    fn intersects(&self, other: &Rect) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }

    fn area(&self) -> f32 {
        let size = self.size();
        size.x * size.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_item(id: u128, x: f32, y: f32, width: f32, height: f32) -> SpatialItem {
        SpatialItem::new(
            EntityId::from_u128(id),
            Rect::from_pos_size(Vec2::new(x, y), Vec2::new(width, height)),
        )
    }

    #[test]
    fn test_spatial_index_new() {
        let index = SpatialIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_insert_single() {
        let mut index = SpatialIndex::new();
        let item = create_test_item(1, 0.0, 0.0, 100.0, 50.0);

        assert!(index.insert(item.clone()));
        assert_eq!(index.len(), 1);
        assert!(index.contains(&item.id));
    }

    #[test]
    fn test_insert_duplicate() {
        let mut index = SpatialIndex::new();
        let item = create_test_item(1, 0.0, 0.0, 100.0, 50.0);

        assert!(index.insert(item.clone()));
        // Duplicado debería reemplazar
        assert!(index.insert(item.clone()));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut index = SpatialIndex::new();
        let item = create_test_item(1, 0.0, 0.0, 100.0, 50.0);

        index.insert(item.clone());
        assert_eq!(index.len(), 1);

        let removed = index.remove(&item.id);
        assert!(removed.is_some());
        assert!(index.is_empty());
    }

    #[test]
    fn test_update() {
        let mut index = SpatialIndex::new();
        let item = create_test_item(1, 0.0, 0.0, 100.0, 50.0);

        index.insert(item);

        let new_bounds = Rect::from_pos_size(Vec2::new(50.0, 50.0), Vec2::new(150.0, 100.0));
        assert!(index.update(&EntityId::from_u128(1), new_bounds));

        let bounds = index.get_bounds(&EntityId::from_u128(1)).unwrap();
        assert_eq!(bounds.min, Vec2::new(50.0, 50.0));
        assert_eq!(bounds.size(), Vec2::new(150.0, 100.0));
    }

    #[test]
    fn test_query_all() {
        let mut index = SpatialIndex::new();
        index.insert(create_test_item(1, 0.0, 0.0, 100.0, 50.0));
        index.insert(create_test_item(2, 50.0, 50.0, 100.0, 50.0));
        index.insert(create_test_item(3, 200.0, 200.0, 50.0, 50.0));

        let result = index.query_all();
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_query_viewport() {
        let mut index = SpatialIndex::new();
        index.insert(create_test_item(1, 0.0, 0.0, 100.0, 50.0));
        index.insert(create_test_item(2, 50.0, 50.0, 100.0, 50.0));
        index.insert(create_test_item(3, 200.0, 200.0, 50.0, 50.0));

        let viewport = Rect::from_pos_size(Vec2::new(0.0, 0.0), Vec2::new(150.0, 100.0));
        let result = index.query_viewport(viewport, SpatialQueryConfig::default());

        // Los items 1 y 2 están parcialmente visibles
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn test_query_point() {
        let mut index = SpatialIndex::new();
        index.insert(create_test_item(1, 0.0, 0.0, 100.0, 50.0));
        index.insert(create_test_item(2, 200.0, 200.0, 50.0, 50.0));

        let result = index.query_point(Vec2::new(50.0, 25.0), SpatialQueryConfig::default());

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, EntityId::from_u128(1));
    }

    #[test]
    fn test_dirty_tracking() {
        let mut index = SpatialIndex::new();
        index.insert(create_test_item(1, 0.0, 0.0, 100.0, 50.0));

        // Verificar que hay un registro dirty
        assert!(!index.dirty_ids().next().is_none());

        // Limpiar dirty
        index.clear_dirty();
        assert!(index.dirty_ids().next().is_none());
    }

    #[test]
    fn test_insert_bulk() {
        let items = vec![
            create_test_item(1, 0.0, 0.0, 100.0, 50.0),
            create_test_item(2, 50.0, 50.0, 100.0, 50.0),
            create_test_item(3, 200.0, 200.0, 50.0, 50.0),
        ];

        let mut index = SpatialIndex::new();
        let count = index.insert_bulk(&items);

        assert_eq!(count, 3);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_build_from() {
        let items = vec![
            create_test_item(1, 0.0, 0.0, 100.0, 50.0),
            create_test_item(2, 50.0, 50.0, 100.0, 50.0),
        ];

        let index = SpatialIndex::build_from(&items);

        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_query_with_filter() {
        let mut index = SpatialIndex::new();
        let mut item1 = create_test_item(1, 0.0, 0.0, 100.0, 50.0);
        item1.data = Some(SpatialData::new("rectangle", 0));

        let mut item2 = create_test_item(2, 50.0, 50.0, 100.0, 50.0);
        item2.data = Some(SpatialData::new("ellipse", 1));

        index.insert(item1);
        index.insert(item2);

        let mut config = SpatialQueryConfig::default();
        let mut filter = HashSet::new();
        filter.insert("rectangle".to_string());
        config.entity_type_filter = Some(filter);

        let viewport = Rect::from_pos_size(Vec2::new(0.0, 0.0), Vec2::new(300.0, 300.0));
        let result = index.query_viewport(viewport, config);

        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_query_ordering() {
        let mut index = SpatialIndex::new();
        index.insert(create_test_item(1, 0.0, 0.0, 100.0, 50.0));
        index.insert(create_test_item(2, 200.0, 200.0, 50.0, 50.0));
        index.insert(create_test_item(3, 100.0, 100.0, 10.0, 10.0));

        let viewport = Rect::from_pos_size(Vec2::new(0.0, 0.0), Vec2::new(300.0, 300.0));
        let mut config = SpatialQueryConfig::default();
        config.order = SpatialQueryOrder::AreaAsc;

        let result = index.query_viewport(viewport, config);

        assert_eq!(result.items.len(), 3);
        // El más pequeño primero
        assert_eq!(result.items[0].id, EntityId::from_u128(3));
    }
}
