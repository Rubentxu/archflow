use archflow_wasm_bridge::logic::{LogicSystemWasm, PulseWasm};

#[test]
fn test_pulse_wasm() {
    // state: 1 = Positive (active), 0 = None (inactive)
    let pulse = PulseWasm::new(123, 5, 1, 1000);
    assert_eq!(pulse.entity_id(), 123);
    assert_eq!(pulse.sensor_id(), 5);
    assert_eq!(pulse.state(), 1); // Positive state
    assert_eq!(pulse.timestamp(), 1000);
}
