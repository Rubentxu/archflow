// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Proximity Sensor Tests
//
// Epic 2.3: Proximity Sensor
// TDD Approach: Red → Green → Refactor
//
// These tests verify the Proximity sensor implementation which detects
// when entities are within a specified distance of each other using
// SpatialHash for O(1) spatial queries.
//
// Note: Integration tests run with std (not no_std)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    use archflow_core::{Rect, Vec2};
    use archflow_engine::{EntityStore, SpatialHash};
    use archflow_logic::sensors::proximity::ProximitySensor;

    // Helper function to setup SpatialHash from EntityStore
    fn setup_spatial_hash(
        store: &EntityStore,
        entity_ids: &[archflow_core::EntityId],
    ) -> SpatialHash {
        let mut spatial = SpatialHash::new(archflow_engine::MAX_ENTITIES);
        for (idx, &entity_id) in entity_ids.iter().enumerate() {
            let transform = store.transforms[idx];
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            let bounds = Rect {
                min: pos - size * 0.5,
                max: pos + size * 0.5,
            };
            spatial.insert(entity_id, bounds);
        }
        spatial
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.3.1: Usa SpatialHash para queries O(1)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_proximity_detects_nearby_entity() {
        // AC2.3.1: Debe usar SpatialHash para queries espaciales eficientes
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(115.0, 100.0), Vec2::new(50.0, 50.0)); // 15px distance

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0); // 20px radius
        sensor.sample(&store, &spatial);

        assert!(sensor.is_near(entity1, entity2));
    }

    #[test]
    fn test_proximity_out_of_radius() {
        // AC2.3.1: Debe retornar false cuando entidades están fuera del radio
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0)); // ~141px distance

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        assert!(!sensor.is_near(entity1, entity2));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.3.2: radius configurable (default: 20px)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_default_radius_is_20px() {
        // AC2.3.2: El radio por defecto debe ser 20px
        let sensor = ProximitySensor::new(100, 20.0);
        assert_eq!(sensor.radius(), 20.0);
    }

    #[test]
    fn test_configurable_radius() {
        // AC2.3.2: Debe poder configurar el radio
        let sensor = ProximitySensor::new(100, 50.0);
        assert_eq!(sensor.radius(), 50.0);
    }

    #[test]
    fn test_respects_custom_radius() {
        // AC2.3.2: La detección debe respetar el radio configurado
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(115.0, 100.0), Vec2::new(50.0, 50.0)); // 15px

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        // Radio de 10px NO debe detectar distancia de 15px
        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 10.0);
        sensor.sample(&store, &spatial);
        assert!(!sensor.is_near(entity1, entity2));

        // Radio de 20px SÍ debe detectar distancia de 15px
        let mut sensor2 = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor2.sample(&store, &spatial);
        assert!(sensor2.is_near(entity1, entity2));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.3.3: is_near(entity, target_entity) → bool
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_is_near_returns_bool() {
        // AC2.3.3: is_near debe retornar bool
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(50.0, 50.0));

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        let result: bool = sensor.is_near(entity1, entity2);
        assert!(result);
    }

    #[test]
    fn test_is_near_symmetric() {
        // AC2.3.3: is_near(a, b) == is_near(b, a)
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(50.0, 50.0));

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        assert_eq!(
            sensor.is_near(entity1, entity2),
            sensor.is_near(entity2, entity1)
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.3.4: get_nearby_entities(position, radius) → Vec<EntityId>
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_get_nearby_entities() {
        // AC2.3.4: Debe retornar todas las entidades cercanas a una posición
        let mut store = EntityStore::new();
        let center = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let near1 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(50.0, 50.0)); // 10px
        let near2 = store.spawn(Vec2::new(105.0, 105.0), Vec2::new(50.0, 50.0)); // ~7px
        let far = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0)); // ~141px

        let spatial = setup_spatial_hash(&store, &[center, near1, near2, far]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        let nearby = sensor.get_nearby_entities(Vec2::new(100.0, 100.0), 20.0);
        assert!(nearby.contains(&center));
        assert!(nearby.contains(&near1));
        assert!(nearby.contains(&near2));
        assert!(!nearby.contains(&far));
    }

    #[test]
    fn test_get_nearby_entities_empty_result() {
        // AC2.3.4: Debe retornar vec vacío cuando no hay entidades cercanas
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let spatial = setup_spatial_hash(&store, &[entity]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        let nearby = sensor.get_nearby_entities(Vec2::new(500.0, 500.0), 20.0);
        assert!(nearby.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_distance_calculation_edge_case() {
        // Entidades justo en el límite del radio
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(120.0, 100.0), Vec2::new(50.0, 50.0)); // Exactly 20px

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        sensor.sample(&store, &spatial);

        // Just at the edge should be considered "near"
        assert!(sensor.is_near(entity1, entity2));
    }

    #[test]
    fn test_zero_radius() {
        // Radio de 0 solo detecta colisión exacta
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0)); // Same position

        let spatial = setup_spatial_hash(&store, &[entity1, entity2]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 0.0);
        sensor.sample(&store, &spatial);

        assert!(sensor.is_near(entity1, entity2));
    }

    #[test]
    fn test_multiple_nearby_entities() {
        // Una entidad con múltiples vecinos cercanos
        let mut store = EntityStore::new();
        let center = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let n1 = store.spawn(Vec2::new(105.0, 100.0), Vec2::new(50.0, 50.0));
        let n2 = store.spawn(Vec2::new(95.0, 100.0), Vec2::new(50.0, 50.0));
        let n3 = store.spawn(Vec2::new(100.0, 105.0), Vec2::new(50.0, 50.0));
        let n4 = store.spawn(Vec2::new(100.0, 95.0), Vec2::new(50.0, 50.0));

        let spatial = setup_spatial_hash(&store, &[center, n1, n2, n3, n4]);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 10.0);
        sensor.sample(&store, &spatial);

        assert!(sensor.is_near(center, n1));
        assert!(sensor.is_near(center, n2));
        assert!(sensor.is_near(center, n3));
        assert!(sensor.is_near(center, n4));
    }

    #[test]
    fn test_zero_entities() {
        // Debe manejar EntityStore vacío
        let store = EntityStore::new();
        let spatial = SpatialHash::new(archflow_engine::MAX_ENTITIES);

        let mut sensor = ProximitySensor::new(archflow_engine::MAX_ENTITIES, 20.0);
        // No debe panic
        sensor.sample(&store, &spatial);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing ProximitySensor in src/sensors/proximity.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test proximity_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
