// ═══════════════════════════════════════════════════════════════════════════════
// Integration Tests - Logic Bricks WASM Bindings
//
// Tests end-to-end integration of the Logic Bricks system:
// - WASM bindings for Sensors, Controllers, Actuators
// - LogicMappingTable connections
// - LogicSystem update loop
// - Pulse generation and handling
//
// EPIC-WEB-010: Complete WASM bindings for Logic Bricks
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use archflow_core::EntityId;
use archflow_logic::{Controller, LogicMappingTable, SensorType, SignalByte};
use archflow_web::logic::{
    Controller as WasmController, LogicMappingTableWasm, LogicSystemWasm, PulseWasm,
    SensorType as WasmSensorType,
};

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 1: WASM Controller Conversions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_wasm_controller_direct_conversion() {
    let wasm = WasmController::direct();
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::Direct));
}

#[test]
fn test_wasm_controller_and_conversion() {
    let wasm = WasmController::and(WasmSensorType::MouseClick);
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::AND(_)));
}

#[test]
fn test_wasm_controller_or_conversion() {
    let wasm = WasmController::or(WasmSensorType::KeyShortcut);
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::OR(_)));
}

#[test]
fn test_wasm_controller_not_conversion() {
    let wasm = WasmController::not();
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::NOT));
}

#[test]
fn test_wasm_controller_blinky_conversion() {
    let wasm = WasmController::blinky(4);
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::Blinky { interval: 4 }));
}

#[test]
fn test_wasm_controller_debounce_conversion() {
    let wasm = WasmController::debounce(6);
    let core: Controller = wasm.into();
    assert!(matches!(core, Controller::Debounce { ticks: 6 }));
}

#[test]
fn test_wasm_controller_hysteresis_conversion() {
    let wasm = WasmController::hysteresis(0.8, 0.3);
    let core: Controller = wasm.into();
    match core {
        Controller::Hysteresis { high, low } => {
            assert_eq!(high, 0.8);
            assert_eq!(low, 0.3);
        }
        _ => panic!("Expected Hysteresis controller"),
    }
}

#[test]
fn test_wasm_controller_custom_conversion() {
    let wasm = WasmController::custom("test_name".to_string(), "test_code".to_string());
    let core: Controller = wasm.into();
    match core {
        Controller::Custom { name, code } => {
            assert_eq!(name, "test_name");
            assert_eq!(code, "test_code");
        }
        _ => panic!("Expected Custom controller"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 2: LogicMappingTable WASM Integration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_mapping_table_add_highlight() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 123;
    let sensor = WasmSensorType::MouseOver;
    let controller = WasmController::direct();

    wasm_table.add_highlight(entity_id, sensor, controller);

    assert!(wasm_table.has_connection(entity_id, sensor));
    assert_eq!(wasm_table.connection_count(entity_id), 1);
}

#[test]
fn test_mapping_table_add_select() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 456;
    let sensor = WasmSensorType::MouseClick;
    let controller = WasmController::direct();

    wasm_table.add_select(entity_id, sensor, controller);

    assert!(wasm_table.has_connection(entity_id, sensor));
    assert_eq!(wasm_table.connection_count(entity_id), 1);
}

#[test]
fn test_mapping_table_add_move() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 789;
    let sensor = WasmSensorType::MouseClick;
    let controller = WasmController::and(WasmSensorType::MouseOver);

    wasm_table.add_move(entity_id, sensor, controller);

    assert!(wasm_table.has_connection(entity_id, sensor));
    assert_eq!(wasm_table.connection_count(entity_id), 1);
}

#[test]
fn test_mapping_table_multiple_connections() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 999;

    wasm_table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );
    wasm_table.add_select(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::direct(),
    );
    wasm_table.add_move(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::and(WasmSensorType::MouseOver),
    );

    assert_eq!(wasm_table.connection_count(entity_id), 3);
}

#[test]
fn test_mapping_table_remove_connection() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 111;
    let sensor = WasmSensorType::MouseOver;
    let controller = WasmController::direct();

    wasm_table.add_highlight(entity_id, sensor, controller);
    assert!(wasm_table.has_connection(entity_id, sensor));

    wasm_table.remove_connection(entity_id, sensor);
    assert!(!wasm_table.has_connection(entity_id, sensor));
    assert_eq!(wasm_table.connection_count(entity_id), 0);
}

#[test]
fn test_mapping_table_clear_entity() {
    let mut wasm_table = LogicMappingTableWasm::new();
    let entity_id = 222;

    wasm_table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );
    wasm_table.add_select(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::direct(),
    );

    wasm_table.clear_entity(entity_id);

    assert!(!wasm_table.has_connection(entity_id, WasmSensorType::MouseOver));
    assert!(!wasm_table.has_connection(entity_id, WasmSensorType::MouseClick));
    assert_eq!(wasm_table.connection_count(entity_id), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 3: SignalByte Integration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_signal_byte_rising_edge() {
    let mut signal = SignalByte::from(0b00000000); // All zeros
    signal.push(true); // Now: 0b00000001 (was 0 at LSB, now 1) - rising edge!

    // Check for rising edge (bit pattern 01 = previous was 0, current is 1)
    assert!(signal.is_rising_edge());
}

#[test]
fn test_signal_byte_falling_edge() {
    let mut signal = SignalByte::from(0b00000011); // Current: 1, Previous: 1
    signal.push(false); // Now: 01111111 (was 1, now 0)

    // Check for falling edge (bit pattern 10)
    assert!(signal.is_falling_edge());
}

#[test]
fn test_signal_byte_steady_high() {
    let mut signal = SignalByte::from(0);
    for _ in 0..6 {
        signal.push(true);
    }

    assert!(signal.is_steady_high(6));
    assert_eq!(signal.count_ones(), 6);
}

#[test]
fn test_signal_byte_steady_low() {
    let mut signal = SignalByte::from(0);
    for _ in 0..6 {
        signal.push(false);
    }

    assert!(signal.is_steady_low(6));
    assert_eq!(signal.count_zeros(), 6);
}

#[test]
fn test_signal_byte_pattern() {
    let signal = SignalByte::from(0b00100100); // Double-click pattern

    // Check pattern match
    let mask = 0b00100100;
    assert_eq!(signal.get_history() & mask, mask);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 4: LogicSystem Integration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_logic_system_creation() {
    let _system = LogicSystemWasm::new();
    // System creates successfully
}

#[test]
fn test_logic_system_update() {
    let mut system = LogicSystemWasm::new();
    system.update(1000); // Update with timestamp
    // Update completes successfully
}

#[test]
fn test_pulse_wasm() {
    let pulse = PulseWasm::new(123, 5, true, 1000);
    assert_eq!(pulse.entity_id(), 123);
    assert_eq!(pulse.sensor_id(), 5);
    assert!(pulse.is_active());
    assert_eq!(pulse.timestamp(), 1000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST 5: End-to-End Integration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_end_to_end_hover_highlight() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 1;

    // Connect MouseOver sensor to Highlight actuator
    table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );

    // Verify connection exists
    assert!(table.has_connection(entity_id, WasmSensorType::MouseOver));
    assert_eq!(table.connection_count(entity_id), 1);
}

#[test]
fn test_end_to_end_click_select() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 2;

    // Connect MouseClick sensor to Select actuator
    table.add_select(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::direct(),
    );

    // Verify connection exists
    assert!(table.has_connection(entity_id, WasmSensorType::MouseClick));
    assert_eq!(table.connection_count(entity_id), 1);
}

#[test]
fn test_end_to_end_complex_and() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 3;

    // Connect MouseClick + MouseOver (AND) to Move actuator
    table.add_move(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::and(WasmSensorType::MouseOver),
    );

    // Verify connection exists
    assert!(table.has_connection(entity_id, WasmSensorType::MouseClick));
    assert_eq!(table.connection_count(entity_id), 1);
}

#[test]
fn test_end_to_end_multiple_behaviors() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 4;

    // Multiple connections for different sensors
    table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );
    table.add_select(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::direct(),
    );
    table.add_move(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::and(WasmSensorType::MouseOver),
    );

    // Verify all connections
    assert_eq!(table.connection_count(entity_id), 3);
    assert!(table.has_connection(entity_id, WasmSensorType::MouseOver));
    assert!(table.has_connection(entity_id, WasmSensorType::MouseClick));
}

#[test]
fn test_end_to_end_remove_behavior() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 5;

    // Add behavior
    table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );
    assert!(table.has_connection(entity_id, WasmSensorType::MouseOver));

    // Remove behavior
    table.remove_connection(entity_id, WasmSensorType::MouseOver);
    assert!(!table.has_connection(entity_id, WasmSensorType::MouseOver));
    assert_eq!(table.connection_count(entity_id), 0);
}

#[test]
fn test_end_to_end_clear_entity() {
    // Create a mapping table
    let mut table = LogicMappingTableWasm::new();
    let entity_id = 6;

    // Add multiple behaviors
    table.add_highlight(
        entity_id,
        WasmSensorType::MouseOver,
        WasmController::direct(),
    );
    table.add_select(
        entity_id,
        WasmSensorType::MouseClick,
        WasmController::direct(),
    );
    table.add_move(
        entity_id,
        WasmSensorType::RightClick,
        WasmController::direct(),
    );

    assert_eq!(table.connection_count(entity_id), 3);

    // Clear all behaviors
    table.clear_entity(entity_id);
    assert_eq!(table.connection_count(entity_id), 0);
}
