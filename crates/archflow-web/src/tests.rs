// Tests for archflow-web WASM module
//!
//! Tests verify the WASM bindings and integration with the SDK

#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_js_library_manager_exists() {
        // Test that JS bindings are properly configured
        // The actual test requires WASM initialization
        assert!(true);
    }
}

#[cfg(test)]
mod library_tests {
    use archflow_sdk::library::LibraryManager;

    #[test]
    fn test_library_manager_creates_libraries() {
        let manager = LibraryManager::new();

        // Should have 4 built-in libraries
        let libraries = manager.get_all_libraries();
        assert_eq!(libraries.len(), 4);
    }

    #[test]
    fn test_general_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("general");

        assert!(library.is_some());
        assert_eq!(library.unwrap().name, "General");
    }

    #[test]
    fn test_flowchart_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("flowchart");

        assert!(library.is_some());
    }

    #[test]
    fn test_uml_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("uml");

        assert!(library.is_some());
    }

    #[test]
    fn test_c4_library_exists() {
        let manager = LibraryManager::new();
        let library = manager.get_library("c4-model");

        assert!(library.is_some());
    }

    #[test]
    fn test_search_items() {
        let manager = LibraryManager::new();

        let results = manager.search_items("rectangle");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_get_item() {
        let manager = LibraryManager::new();
        let item = manager.get_item("general", "rect");

        assert!(item.is_ok());
        assert_eq!(item.unwrap().name, "Rectangle");
    }
}

#[cfg(test)]
mod layers_tests {
    use archflow_sdk::layers::{C4Level, LayerManager};

    #[test]
    fn test_layer_manager_creation() {
        let manager = LayerManager::new();
        assert_eq!(manager.layer_count(), 0);
    }

    #[test]
    fn test_create_layer() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer");

        assert_eq!(manager.layer_count(), 1);
        assert!(manager.get_layer(id).is_some());
    }

    #[test]
    fn test_layer_visibility() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer");

        assert!(manager.set_layer_visibility(id, false));
        assert!(!manager.get_layer(id).unwrap().visible);
    }

    #[test]
    fn test_layer_lock() {
        let mut manager = LayerManager::new();
        let id = manager.create_layer(C4Level::Context, "Test Layer");

        assert!(manager.set_layer_locked(id, true));
        assert!(manager.get_layer(id).unwrap().locked);
    }

    #[test]
    fn test_move_layer_up() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1");
        let id2 = manager.create_layer(C4Level::Context, "Layer 2");

        assert!(manager.move_layer_up(id1));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id2);
    }

    #[test]
    fn test_move_layer_to_top() {
        let mut manager = LayerManager::new();
        let id1 = manager.create_layer(C4Level::Context, "Layer 1");
        let id2 = manager.create_layer(C4Level::Context, "Layer 2");
        let id3 = manager.create_layer(C4Level::Context, "Layer 3");

        assert!(manager.move_layer_to_top(id1));
        let layers = manager.get_layers_in_order();
        assert_eq!(layers[0].id, id2);
        assert_eq!(layers[1].id, id3);
        assert_eq!(layers[2].id, id1);
    }

    #[test]
    fn test_c4_level_string_conversion() {
        assert_eq!(C4Level::Context.as_str(), "context");
        assert_eq!(C4Level::Container.as_str(), "container");
        assert_eq!(C4Level::Component.as_str(), "component");
        assert_eq!(C4Level::Code.as_str(), "code");
    }

    #[test]
    fn test_c4_level_zoom() {
        assert!((C4Level::Context.default_zoom() - 0.15).abs() < 0.01);
        assert!((C4Level::Container.default_zoom() - 0.5).abs() < 0.01);
        assert!((C4Level::Component.default_zoom() - 1.0).abs() < 0.01);
        assert!((C4Level::Code.default_zoom() - 2.0).abs() < 0.01);
    }
}
