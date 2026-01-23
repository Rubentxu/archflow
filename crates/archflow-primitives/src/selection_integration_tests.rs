//! Integration tests for Selection System
//!
//! Tests the interaction between SelectionManager, DragSelectionBox,
//! and visual feedback components.

#[cfg(test)]
mod selection_integration_tests {
    use crate::{
        DragSelectionBox, DragSelectionConfig, HandleType, HitTestResult, Primitive, PrimitiveType,
        SelectionConfig, SelectionEntry, SelectionManager, SelectionManagerBuilder, SelectionMode,
    };
    use archflow_core::{EntityId, Rect, Transform, Vec2};

    /// Primitiva de prueba con posición configurable
    struct TestPrimitive {
        id: EntityId,
        position: Vec2,
        size: Vec2,
    }

    impl TestPrimitive {
        fn new(id: u128, position: Vec2, size: Vec2) -> Self {
            Self {
                id: EntityId::from_u128(id),
                position,
                size,
            }
        }
    }

    impl Primitive for TestPrimitive {
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
            Rect::from_pos_size(self.position, self.size)
        }

        fn global_bounds(&self) -> Rect {
            self.local_bounds()
        }

        fn contains_point(&self, point: Vec2) -> bool {
            self.local_bounds().contains(point)
        }
    }

    /// Crear manager de selección con configuración personalizada
    fn create_test_manager(mode: SelectionMode) -> SelectionManager {
        SelectionManagerBuilder::new().mode(mode).build()
    }

    // ========== Tests de integración SelectionManager + Primitives ==========

    #[test]
    fn test_selection_manager_with_multiple_primitives() {
        let mut manager = create_test_manager(SelectionMode::Multiple);

        // Crear primitivas en diferentes posiciones
        let primitives: Vec<TestPrimitive> = vec![
            TestPrimitive::new(1, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(2, Vec2::new(200.0, 0.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(3, Vec2::new(400.0, 0.0), Vec2::new(100.0, 100.0)),
        ];

        // Seleccionar primitivas 1 y 3
        manager.select(primitives[0].id());
        manager.select(primitives[2].id());

        assert_eq!(manager.count(), 2);
        assert!(manager.is_selected(primitives[0].id()));
        assert!(!manager.is_selected(primitives[1].id()));
        assert!(manager.is_selected(primitives[2].id()));
    }

    #[test]
    fn test_selection_manager_hit_test_integration() {
        let manager = create_test_manager(SelectionMode::Single);

        let primitive = TestPrimitive::new(1, Vec2::new(100.0, 100.0), Vec2::new(100.0, 100.0));

        // Hit test dentro de la primitiva
        let result = manager.hit_test(Vec2::new(150.0, 150.0), &primitive);
        assert!(result.primitive.is_some());
        assert_eq!(result.primitive.unwrap(), primitive.id());

        // Hit test fuera de la primitiva
        let result = manager.hit_test(Vec2::new(500.0, 500.0), &primitive);
        assert!(result.primitive.is_none());
    }

    #[test]
    fn test_selection_manager_with_handles() {
        let mut manager = create_test_manager(SelectionMode::Single);

        // Seleccionar una primitiva
        let primitive = TestPrimitive::new(1, Vec2::new(100.0, 100.0), Vec2::new(100.0, 100.0));
        manager.select(primitive.id());

        // Verificar que hit test devuelve handle
        let bounds = primitive.local_bounds();
        let top_left = HandleType::TopLeft.position(bounds);

        let result = manager.hit_test(top_left, &primitive);
        assert!(result.handle.is_some());
        assert_eq!(result.handle.unwrap(), HandleType::TopLeft);
    }

    // ========== Tests de integración DragSelectionBox + SelectionManager ==========

    #[test]
    fn test_drag_selection_integration() {
        let mut manager = create_test_manager(SelectionMode::Multiple);

        let primitives: Vec<TestPrimitive> = vec![
            TestPrimitive::new(1, Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(2, Vec2::new(300.0, 50.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(3, Vec2::new(550.0, 50.0), Vec2::new(100.0, 100.0)),
        ];

        // Crear drag selection box que abarca primitivas 1 y 2
        let mut drag_box = DragSelectionBox::new(Vec2::new(0.0, 0.0));
        drag_box.update(Vec2::new(350.0, 150.0));

        // Usar select_in_rect para seleccionar
        let found = manager.select_in_rect(
            drag_box.rect(),
            primitives.iter().map(|p| p as &dyn Primitive),
        );

        assert_eq!(found.len(), 2);
        assert!(manager.is_selected(primitives[0].id()));
        assert!(manager.is_selected(primitives[1].id()));
        assert!(!manager.is_selected(primitives[2].id()));
    }

    #[test]
    fn test_drag_selection_add_mode() {
        let mut manager = create_test_manager(SelectionMode::Multiple);

        // Seleccionar primitiva 1 primero
        let primitives: Vec<TestPrimitive> = vec![
            TestPrimitive::new(1, Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(2, Vec2::new(300.0, 50.0), Vec2::new(100.0, 100.0)),
        ];

        manager.select(primitives[0].id());
        assert_eq!(manager.count(), 1);

        // Simular drag selection con add mode
        let mut drag_box = DragSelectionBox::new(Vec2::new(250.0, 0.0));
        drag_box.set_add_mode(true);
        drag_box.update(Vec2::new(400.0, 150.0));

        // Seleccionar en modo add
        let found = manager.select_in_rect(
            drag_box.rect(),
            primitives.iter().map(|p| p as &dyn Primitive),
        );

        assert_eq!(found.len(), 1); // Solo encontró la primitiva 2
        assert_eq!(manager.count(), 2); // Ahora hay 2 seleccionadas
        assert!(manager.is_selected(primitives[0].id()));
        assert!(manager.is_selected(primitives[1].id()));
    }

    // ========== Tests de integración SelectionConfig ==========

    #[test]
    fn test_selection_config_customization() {
        let custom_config = SelectionConfig {
            highlight_color: [1.0, 0.0, 0.0, 1.0], // Rojo
            highlight_width: 3.0,
            handle_color: [0.0, 1.0, 0.0, 1.0], // Verde
            handle_size: 12.0,
            show_transform_handles: true,
            show_bounding_box: true,
            enable_animation: true,
            animation_duration_ms: 300,
        };

        let mut manager = SelectionManagerBuilder::new().config(custom_config).build();

        let primitive = TestPrimitive::new(1, Vec2::ZERO, Vec2::new(100.0, 50.0));
        manager.select(primitive.id());

        assert!(manager.is_selected(primitive.id()));
        assert_eq!(manager.mode(), SelectionMode::Single);
    }

    // ========== Tests de integración completa ==========

    #[test]
    fn test_complete_selection_workflow() {
        // Workflow completo: crear primitivas, seleccionar, drag select, verificar
        let mut manager = create_test_manager(SelectionMode::Multiple);

        let primitives: Vec<TestPrimitive> = vec![
            TestPrimitive::new(1, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(2, Vec2::new(150.0, 0.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(3, Vec2::new(300.0, 0.0), Vec2::new(100.0, 100.0)),
            TestPrimitive::new(4, Vec2::new(450.0, 0.0), Vec2::new(100.0, 100.0)),
        ];

        // 1. Click simple en primitiva 1
        let hit1 = manager.hit_test(Vec2::new(50.0, 50.0), &primitives[0]);
        if hit1.primitive.is_some() {
            manager.select(hit1.primitive.unwrap());
        }
        assert_eq!(manager.count(), 1);

        // 2. Shift+Click en primitiva 3 (añadir)
        let hit3 = manager.hit_test(Vec2::new(350.0, 50.0), &primitives[2]);
        if hit3.primitive.is_some() {
            manager.add_to_selection(hit3.primitive.unwrap());
        }
        assert_eq!(manager.count(), 2);

        // 3. Drag selection para añadir primitivas 2 y 4
        let mut drag_box = DragSelectionBox::new(Vec2::new(100.0, -50.0));
        drag_box.set_add_mode(true);
        drag_box.update(Vec2::new(500.0, 150.0));

        let found = manager.select_in_rect(
            drag_box.rect(),
            primitives.iter().map(|p| p as &dyn Primitive),
        );

        // El drag box contiene todas las primitivas (100,-50) a (500,150)
        assert_eq!(found.len(), 4); // Encontró 1, 2, 3, 4
        assert_eq!(manager.count(), 4); // Ahora todas están seleccionadas

        // 4. Toggle para deseleccionar primitiva 2
        manager.toggle(primitives[1].id());
        assert_eq!(manager.count(), 3);
        assert!(!manager.is_selected(primitives[1].id()));

        // 5. Verificar selección en orden
        let ordered: Vec<EntityId> = manager.selected_in_order().cloned().collect();
        assert_eq!(ordered.len(), 3);

        // 6. Clear para deseleccionar todo
        manager.clear();
        assert!(!manager.has_selection());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_selection_manager_versioning() {
        let mut manager = create_test_manager(SelectionMode::Single);
        let initial_version = manager.version();

        let primitive = TestPrimitive::new(1, Vec2::ZERO, Vec2::new(100.0, 50.0));
        manager.select(primitive.id());
        assert_ne!(manager.version(), initial_version);

        let version_after_select = manager.version();

        manager.clear();
        assert_ne!(manager.version(), version_after_select);
    }

    // ========== Tests de Edge Cases ==========

    #[test]
    fn test_empty_selection() {
        let manager = create_test_manager(SelectionMode::Multiple);

        assert!(!manager.has_selection());
        assert_eq!(manager.count(), 0);
        assert!(manager.selected_in_order().next().is_none());
    }

    #[test]
    fn test_selection_mode_transition() {
        let mut manager = create_test_manager(SelectionMode::Multiple);

        // Añadir 3 elementos en modo Multiple
        for i in 1..=3 {
            manager.add_to_selection(EntityId::from_u128(i));
        }
        assert_eq!(manager.count(), 3);

        // Cambiar a Single - solo debe quedar 1
        manager.set_mode(SelectionMode::Single);
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.mode(), SelectionMode::Single);
    }

    #[test]
    fn test_drag_selection_minimal_area() {
        let mut drag_box = DragSelectionBox::new(Vec2::new(100.0, 100.0));

        // Sin movimiento - sin área
        assert!(!drag_box.has_area());

        // Movimiento muy pequeño - sin área
        drag_box.update(Vec2::new(100.5, 100.5));
        assert!(!drag_box.has_area());

        // Movimiento suficiente - tiene área
        drag_box.update(Vec2::new(105.0, 105.0));
        assert!(drag_box.has_area());
    }

    #[test]
    fn test_drag_selection_reverse_drag() {
        let mut drag_box = DragSelectionBox::new(Vec2::new(200.0, 200.0));

        // Drag de derecha a izquierda
        drag_box.update(Vec2::new(100.0, 100.0));

        let rect = drag_box.rect();
        assert_eq!(rect.min, Vec2::new(100.0, 100.0));
        assert_eq!(rect.max, Vec2::new(200.0, 200.0));
        assert!(drag_box.has_area());
    }
}
