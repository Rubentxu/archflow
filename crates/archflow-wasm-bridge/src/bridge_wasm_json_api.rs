// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - JSON API Extensions
//
// This file contains the JSON declarative API extensions for WasmBridge
// as specified in the Developer Manual.
//
// These functions should be added to the WasmBridge impl block in bridge_wasm.rs
// ═══════════════════════════════════════════════════════════════════════════════

// Add these imports to bridge_wasm.rs:
// use crate::behavior_json::{BehaviorDefinition, BehaviorRegistry, BehaviorError};

// Add this field to WasmBridge struct:
// behavior_registry: RefCell<BehaviorRegistry>,

// Add to WasmBridge::new():
// behavior_registry: RefCell::new(BehaviorRegistry::new()),

// ═══════════════════════════════════════════════════════════════════════════════
// JSON BEHAVIOR API - Add these methods to WasmBridge impl block
// ═══════════════════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════════════════
    // SECTION 10: JSON DECLARATIVE API
    // Developer Manual: API Declarativa estilo A-Frame
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Register a behavior from JSON string
    ///
    /// This implements the declarative JSON API as specified in the Developer Manual.
    ///
    /// # Example
    ///
    /// ```javascript
    /// const behaviorJson = JSON.stringify({
    ///   id: 'hover-highlight',
    ///   name: 'Hover Highlight',
    ///   components: [
    ///     { type: 'sensor-mouse', config: { mode: 'hover' } },
    ///     { type: 'actuator-highlight', config: { color: '#ffff00', opacity: 0.5 } }
    ///   ]
    /// });
    /// bridge.register_behavior(behaviorJson);
    /// ```
    #[wasm_bindgen]
    pub fn register_behavior(&self, json: &str) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::json", "Registering behavior from JSON");

        let registry = &self.behavior_registry;

        // Ensure logic system is set
        if registry.logic_system.borrow().is_none() {
            if let Some(engine) = self.engine.borrow().as_ref() {
                registry.set_logic_system(engine.logic_bricks.clone());
            }
        }

        registry.register_behavior(json)
            .map_err(|e| JsError::new(&alloc::format!("Behavior registration failed: {}", e)))
    }

    /// Register multiple behaviors from JSON array string
    ///
    /// This is the batch version of register_behavior for better performance.
    ///
    /// # Example
    ///
    /// ```javascript
    /// const behaviorsJson = JSON.stringify([
    ///   { id: 'hover-1', name: 'Hover', components: [...] },
    ///   { id: 'click-1', name: 'Click', components: [...] }
    /// ]);
    /// bridge.register_behaviors_json(behaviorsJson);
    /// ```
    #[wasm_bindgen]
    pub fn register_behaviors_json(&self, json_array: &str) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::json", "Registering behaviors from JSON array");

        let registry = &self.behavior_registry;

        // Ensure logic system is set
        if registry.logic_system.borrow().is_none() {
            if let Some(engine) = self.engine.borrow().as_ref() {
                registry.set_logic_system(engine.logic_bricks.clone());
            }
        }

        registry.register_behaviors(json_array)
            .map_err(|e| JsError::new(&alloc::format!("Batch behavior registration failed: {}", e)))
    }

    /// Create a behavior from JSON and return its ID
    ///
    /// This is useful for dynamic behavior creation where you need the ID back.
    #[wasm_bindgen]
    pub fn create_behavior(&self, json: &str) -> Result<String, JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::json", "Creating behavior from JSON");

        let definition: BehaviorDefinition = serde_json::from_str(json)
            .map_err(|e| JsError::new(&alloc::format!("JSON parse error: {}", e)))?;

        Ok(definition.id)
    }

    /// Validate behavior JSON without registering it
    ///
    /// Returns true if the JSON is valid, false otherwise.
    #[wasm_bindgen]
    pub fn validate_behavior(&self, json: &str) -> bool {
        let _: Result<BehaviorDefinition, _> = serde_json::from_str(json);
        _.is_ok()
    }

    /// Get behavior template as JSON string
    ///
    /// Returns predefined behavior templates that can be customized.
    ///
    /// # Templates
    ///
    /// - "hover-highlight": Hover with highlight
    /// - "click-select": Click to select
    /// - "draggable": Drag to move
    /// - "hoverable-clickable": Combined hover + click
    /// - "deletable": Delete key to remove
    /// - "editable": Double-click to edit
    #[wasm_bindgen]
    pub fn get_behavior_template(&self, template_name: &str) -> Result<String, JsValue> {
        let template = match template_name {
            "hover-highlight" => r#"{
                "id": "hover-highlight",
                "name": "Hover Highlight",
                "description": "Highlights entity on hover",
                "components": [
                    { "type": "sensor-mouse", "config": { "mode": "hover" } },
                    { "type": "actuator-highlight", "config": { "color": "#ffff00", "opacity": 0.5 } }
                ]
            }"#,

            "click-select" => r#"{
                "id": "click-select",
                "name": "Click Select",
                "description": "Selects entity on click",
                "components": [
                    { "type": "sensor-mouse", "config": { "mode": "click", "button": 0 } },
                    { "type": "actuator-select", "config": { "mode": "single" } }
                ]
            }"#,

            "draggable" => r#"{
                "id": "draggable",
                "name": "Draggable",
                "description": "Makes entity draggable",
                "components": [
                    { "type": "sensor-mouse", "config": { "mode": "drag", "button": 0 } },
                    { "type": "controller-debounce", "config": { "ticks": 3 } },
                    { "type": "actuator-move", "config": { "mode": "follow-cursor", "speed": 5.0 } }
                ]
            }"#,

            "hoverable-clickable" => r#"{
                "id": "hoverable-clickable",
                "name": "Hoverable and Clickable",
                "description": "Highlights on hover and selects on click",
                "components": [
                    { "type": "sensor-mouse", "config": { "mode": "hover" } },
                    { "type": "actuator-highlight", "config": { "color": "#ffff00", "opacity": 0.3 } },
                    { "type": "sensor-mouse", "config": { "mode": "click", "button": 0 } },
                    { "type": "actuator-select", "config": { "mode": "toggle" } }
                ]
            }"#,

            "deletable" => r#"{
                "id": "deletable",
                "name": "Deletable",
                "description": "Delete entity with Delete key",
                "components": [
                    { "type": "sensor-keyboard", "config": { "keys": [46], "modifiers": 0 } },
                    { "type": "actuator-delete", "config": {} }
                ]
            }"#,

            "editable" => r#"{
                "id": "editable",
                "name": "Editable",
                "description": "Double-click to edit",
                "components": [
                    { "type": "sensor-mouse", "config": { "mode": "dblclick" } },
                    { "type": "actuator-event", "config": { "name": "editStart", "data": {} } }
                ]
            }"#,

            _ => {
                return Err(JsError::new(&alloc::format!("Unknown template: {}", template_name)));
            }
        };

        Ok(template.to_string())
    }

    /// List available behavior templates
    #[wasm_bindgen]
    pub fn list_behavior_templates(&self) -> Result<js_sys::Array, JsValue> {
        let templates = js_sys::Array::new();
        templates.push(&JsValue::from_str("hover-highlight"));
        templates.push(&JsValue::from_str("click-select"));
        templates.push(&JsValue::from_str("draggable"));
        templates.push(&JsValue::from_str("hoverable-clickable"));
        templates.push(&JsValue::from_str("deletable"));
        templates.push(&JsValue::from_str("editable"));
        Ok(templates)
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EVENT CALLBACKS - Developer Manual: Reactive responses to WASM events
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Register a callback for behavior events
    ///
    /// This implements the reactive event system as specified in the manual.
    ///
    /// # Example
    ///
    /// ```javascript
    /// bridge.on_event((event) => {
    ///   console.log('Event:', event.type, event.entityId, event.data);
    /// });
    /// ```
    #[wasm_bindgen]
    pub fn on_event(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::events", "Registering event callback");

        // Store callback for later use in poll_events()
        self.event_callbacks.borrow_mut().push(callback.clone());

        Ok(())
    }

    /// Remove all event callbacks
    #[wasm_bindgen]
    pub fn clear_event_callbacks(&self) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::events", "Clearing event callbacks");

        self.event_callbacks.borrow_mut().clear();

        Ok(())
    }

    /// Get event count since last poll
    ///
    /// This is useful for checking if there are events without polling them.
    #[wasm_bindgen]
    pub fn get_event_count(&self) -> Result<u32, JsValue> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            Ok(engine.logic_bricks.event_buffer_count() as u32)
        } else {
            Err(JsError::new("Engine not initialized").into())
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // BATCH PROCESSING - Developer Manual: Batch processing for performance
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Begin a batch operation
    ///
    /// Use this for batch registration of multiple behaviors to minimize
    /// WASM bridge overhead.
    #[wasm_bindgen]
    pub fn begin_batch(&self) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::batch", "Beginning batch operation");

        self.in_batch.set(true);

        Ok(())
    }

    /// End a batch operation and flush all pending operations
    #[wasm_bindgen]
    pub fn end_batch(&self) -> Result<(), JsValue> {
        #[cfg(feature = "tracing-logging")]
        info!(target: "archflow::wasm::batch", "Ending batch operation");

        self.in_batch.set(false);

        // Flush any pending batch operations
        if self.batch_queue.borrow().len() > 0 {
            let queue = self.batch_queue.borrow_mut();
            let json_array = serde_json::to_string(&*queue).unwrap();
            drop(queue);

            self.register_behaviors_json(&json_array)?;
            self.batch_queue.borrow_mut().clear();
        }

        Ok(())
    }

    /// Add behavior to batch queue
    ///
    /// Behaviors added via this method are queued until end_batch() is called.
    #[wasm_bindgen]
    pub fn add_to_batch(&self, json: &str) -> Result<(), JsValue> {
        if !self.in_batch.get() {
            return Err(JsError::new("Not in batch mode. Call begin_batch() first.").into());
        }

        let definition: BehaviorDefinition = serde_json::from_str(json)
            .map_err(|e| JsError::new(&alloc::format!("JSON parse error: {}", e)))?;

        self.batch_queue.borrow_mut().push(definition);

        Ok(())
    }

// ═══════════════════════════════════════════════════════════════════════════════
// ADD THESE FIELDS TO WasmBridge STRUCT
// ═══════════════════════════════════════════════════════════════════════════════

// Add these fields to WasmBridge struct:
// behavior_registry: RefCell<BehaviorRegistry>,
// event_callbacks: RefCell<Vec<js_sys::Function>>,
// in_batch: Cell<bool>,
// batch_queue: RefCell<Vec<BehaviorDefinition>>,

// ═══════════════════════════════════════════════════════════════════════════════
// UPDATE WasmBridge::new() CONSTRUCTOR
// ═══════════════════════════════════════════════════════════════════════════════

// Update the constructor to initialize new fields:
// pub fn new() -> Self {
//     init_tracing();
//
//     #[cfg(feature = "tracing-logging")]
//     debug!(target: "archflow::wasm", "WasmBridge created");
//
//     Self {
//         engine: RefCell::new(None),
//         input_processor: RefCell::new(None),
//         #[cfg(target_arch = "wasm32")]
//         on_context_lost: Cell::new(None),
//         #[cfg(target_arch = "wasm32")]
//         on_context_restored: Cell::new(None),
//         #[cfg(target_arch = "wasm32")]
//         is_recovering: Cell::new(false),
//         #[cfg(target_arch = "wasm32")]
//         pending_canvas: Cell::new(None),
//         #[cfg(target_arch = "wasm32")]
//         canvas: RefCell::new(None),
//         behavior_registry: RefCell::new(BehaviorRegistry::new()),
//         event_callbacks: RefCell::new(Vec::new()),
//         in_batch: Cell::new(false),
//         batch_queue: RefCell::new(Vec::new()),
//     }
// }

// ═══════════════════════════════════════════════════════════════════════════════
// UPDATE poll_events() TO CALL CALLBACKS
// ═══════════════════════════════════════════════════════════════════════════════

// Update the poll_events() method to dispatch events to callbacks:
// #[wasm_bindgen]
// pub fn poll_events(&self) -> usize {
//     let event_count = if let Some(engine) = self.engine.borrow_mut().as_mut() {
//         engine.logic_bricks.poll_events()
//     } else {
//         0
//     };
//
//     // Dispatch events to registered callbacks
//     if event_count > 0 {
//         if let Some(engine) = self.engine.borrow().as_ref() {
//             let events = engine.logic_bricks.get_events();
//             for callback in self.event_callbacks.borrow().iter() {
//                 for event in &events {
//                     let js_event = event.to_js_value();
//                     let _ = callback.call1(&JsValue::NULL, &js_event);
//                 }
//             }
//         }
//     }
//
//     event_count
// }
