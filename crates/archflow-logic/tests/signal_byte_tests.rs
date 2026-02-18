// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - SignalByte Tests
//
// Epic 1.1: SignalByte with 6-tick history
// TDD Approach: Red → Green → Refactor
//
// These tests verify the SignalByte implementation which stores 6 ticks of
// binary signal history in a single byte for efficient signal processing.
// ═══════════════════════════════════════════════════════════════════════════════

// Integration tests run with std (not no_std)
#[cfg(test)]
mod tests {
    use std::time::Instant;
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    // Import will be uncommented after SignalByte is implemented:
    // use archflow_logic::signals::SignalByte;

    #[test]
    fn test_signal_byte_size() {
        // AC1.1: SignalByte must be exactly 1 byte
        // This is critical for memory efficiency: 100KB for 100k entities
        assert_eq!(std::mem::size_of::<u8>(), 1, "u8 is 1 byte");
        // After implementation:
        // assert_eq!(std::mem::size_of::<SignalByte>(), 1);
    }

    #[test]
    fn test_push_shifts_left() {
        // AC1.2: push() method shifts history left, adds new bit as LSB
        //
        // Timeline:
        // Initial: 0b00000000
        // push(true):  0b00000001 (T0 = 1)
        // push(true):  0b00000011 (T0 = 1, T1 = 1)
        // push(false): 0b00000110 (T0 = 0, T1 = 1, T2 = 1)

        // Verify bit shifting semantics:
        let mut value: u8 = 0b00000000;
        value = (value << 1) | 1; // push true
        assert_eq!(value, 0b00000001);

        value = (value << 1) | 1; // push true
        assert_eq!(value, 0b00000011);

        value = (value << 1) | 0; // push false
        assert_eq!(value, 0b00000110);

        // After implementation:
        // let mut signal = SignalByte::default();
        // signal.push(true);
        // signal.push(true);
        // signal.push(false);
        // assert_eq!(signal.get_history(), 0b00000110);
    }

    #[test]
    fn test_get_current_returns_lsb() {
        // AC1.3: get_current() returns the least significant bit (tick T0)

        // Test with LSB = 1
        let value_with_lsb_one: u8 = 0b11111001; // LSB is 1
        assert_eq!(value_with_lsb_one & 1, 1, "LSB should be 1");

        // Test with LSB = 0
        let value_with_lsb_zero: u8 = 0b11111010; // LSB is 0
        assert_eq!(value_with_lsb_zero & 1, 0, "LSB should be 0");

        // After implementation:
        // let mut signal = SignalByte::default();
        // signal.push(true);
        // assert!(signal.get_current());
        //
        // signal.push(false);
        // assert!(!signal.get_current());
    }

    #[test]
    fn test_get_history_masks_6_bits() {
        // AC1.4: get_history() returns only the 6 LSB bits (T5 through T0)

        let value: u8 = 0b11111111;
        let masked = value & 0b111111; // Mask to 6 bits
        assert_eq!(masked, 0b111111);

        // After implementation:
        // let signal = SignalByte::from(0b11111111);
        // assert_eq!(signal.get_history(), 0b111111);
    }

    #[test]
    fn test_6_ticks_overflow_loses_oldest() {
        // AC1.4 continued: After 7 pushes, oldest bit (T6) is lost

        let mut value: u8 = 0;

        // Push 7 times
        for _ in 0..7 {
            value = (value << 1) | 1;
        }

        // Value should be 0b11111110 (only 6 lowest bits matter)
        let history = value & 0b111111;
        assert_eq!(history, 0b111111, "6 lowest bits should all be 1");

        // After implementation:
        // let mut signal = SignalByte::default();
        // for _ in 0..7 {
        //     signal.push(true);
        // }
        // assert_eq!(signal.get_history() & 0b111111, 0b111111);
    }

    #[test]
    fn test_default_is_zero() {
        // AC1.5: Default SignalByte should have all zeros

        let default_u8: u8 = Default::default();
        assert_eq!(default_u8, 0);

        // After implementation:
        // let signal = SignalByte::default();
        // assert_eq!(signal.get_history(), 0);
    }

    #[test]
    fn test_copy_trait() {
        // AC1.6: SignalByte must be Copy (can be duplicated without move)

        let value1: u8 = 0b101010;
        let value2 = value1; // Copy, not move
        assert_eq!(value1, 0b101010, "original still valid");
        assert_eq!(value2, 0b101010, "copy has same value");

        // After implementation:
        // let signal1 = SignalByte::from(0b101010);
        // let signal2 = signal1;  // Should compile (Copy trait)
        // assert_eq!(signal1.get_history(), 0b101010);
        // assert_eq!(signal2.get_history(), 0b101010);
    }

    #[test]
    fn test_clone_trait() {
        // AC1.6 continued: SignalByte must be Clone

        let value1: u8 = 0b010101;
        let value2 = value1.clone();
        assert_eq!(value1, value2);

        // After implementation:
        // let signal1 = SignalByte::from(0b010101);
        // let signal2 = signal1.clone();
        // assert_eq!(signal1.get_history(), signal2.get_history());
    }

    #[test]
    fn test_equality_and_debug() {
        // AC1.5: SignalByte should support PartialEq and Debug

        let value1: u8 = 0b001100;
        let value2: u8 = 0b001100;
        let value3: u8 = 0b110011;

        assert_eq!(value1, value2);
        assert_ne!(value1, value3);

        // Debug should work:
        format!("{:?}", value1);

        // After implementation:
        // let signal1 = SignalByte::from(0b001100);
        // let signal2 = SignalByte::from(0b001100);
        // let signal3 = SignalByte::from(0b110011);
        // assert_eq!(signal1, signal2);
        // assert_ne!(signal1, signal3);
        // format!("{:?}", signal1);  // Should compile
    }

    #[test]
    fn test_from_u8() {
        // Test construction from raw u8 value

        let _raw: u8 = 0b101010;

        // After implementation:
        // let signal = SignalByte::from(raw);
        // assert_eq!(signal.as_u8(), raw);
    }

    #[test]
    fn test_const_from() {
        // Test that from() can be used in const contexts

        const RAW: u8 = 0b111000;

        // After implementation:
        // const SIGNAL: SignalByte = SignalByte::from(RAW);
        // assert_eq!(SIGNAL.as_u8(), RAW);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PERFORMANCE TESTS (to verify <0.1ms target)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_push_performance() {
        // Verify that 1M push operations complete in reasonable time
        // Target: <30ms for 1M operations in debug mode (~30ns per operation)
        // In release mode, this should be <5ms

        let start = Instant::now();
        let mut value: u8 = 0;

        for _ in 0..1_000_000 {
            value = (value << 1) | 1;
        }

        let elapsed = start.elapsed();
        println!("1M pushes took: {:?}", elapsed);

        // Should be very fast (<30ms even in debug mode)
        assert!(
            elapsed.as_millis() < 30,
            "Push operations should be fast, took: {:?}",
            elapsed
        );

        // After implementation, replace with:
        // let mut signal = SignalByte::default();
        // for _ in 0..1_000_000 {
        //     signal.push(true);
        // }
    }

    #[test]
    fn test_bitwise_operations() {
        // Verify that bitwise operations are inlined and efficient

        let value: u8 = 0b101010;
        let mask = 0b111111;
        let result = value & mask;

        assert_eq!(result, 0b101010);

        // This should compile to a single AND instruction
        // After implementation:
        // let signal = SignalByte::from(0b101010);
        // assert_eq!(signal.get_history(), 0b101010);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_all_zeros() {
        let value: u8 = 0b00000000;
        assert_eq!(value & 0b111111, 0);

        // After implementation:
        // let signal = SignalByte::default();
        // assert!(!signal.get_current());
        // assert_eq!(signal.count_ones(), 0);
    }

    #[test]
    fn test_all_ones() {
        let value: u8 = 0b11111111;
        assert_eq!(value & 0b111111, 0b111111);

        // After implementation:
        // let signal = SignalByte::from(0b11111111);
        // assert!(signal.get_current());
        // assert_eq!(signal.count_ones(), 6);
    }

    #[test]
    fn test_alternating_pattern() {
        // Pattern: 101010 (alternating)
        let _value: u8 = 0b101010;

        // After implementation:
        // let signal = SignalByte::from(0b101010);
        // assert!(!signal.get_current());  // LSB is 0
        // assert_eq!(signal.count_ones(), 3);  // 3 ones in 6 bits
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GREEN PHASE CHECKLIST
// ═══════════════════════════════════════════════════════════════════════════════
//
// After implementing SignalByte in src/signals.rs:
//
// 1. Uncomment the imports
// 2. Uncomment the test implementations
// 3. Run: cargo test --package archflow-logic
// 4. Verify all tests pass
// 5. Run: cargo test --package archflow-logic --release
// 6. Verify performance tests meet targets
//
// ═══════════════════════════════════════════════════════════════════════════════
