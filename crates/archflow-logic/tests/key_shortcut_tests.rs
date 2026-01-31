// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - KeyShortcut Sensor Tests
//
// Epic 2.4: KeyShortcut Sensor
// TDD Approach: Red → Green → Refactor
//
// These tests verify the KeyShortcut sensor implementation which detects
// keyboard key presses and combinations with modifiers.
//
// Note: Integration tests run with std (not no_std)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    use archflow_logic::sensors::key_shortcut::{
        KeyCode, KeyEvent, KeyModifiers, KeyShortcutSensor,
    };

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.4.1: Detecta teclas específicas (KeyCode enum)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_detects_single_key_press() {
        // AC2.4.1: Debe detectar press de tecla específica
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        assert!(sensor.on_key_press(KeyCode::KeyA));
    }

    #[test]
    fn test_no_press_when_released() {
        // AC2.4.1: No debe detectar press cuando tecla está liberada
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: false,
        });

        assert!(!sensor.on_key_press(KeyCode::KeyA));
    }

    #[test]
    fn test_detects_multiple_different_keys() {
        // AC2.4.1: Debe detectar diferentes teclas simultáneamente
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        sensor.sample(KeyEvent {
            key: KeyCode::KeyB,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        assert!(sensor.on_key_press(KeyCode::KeyA));
        assert!(sensor.on_key_press(KeyCode::KeyB));
    }

    #[test]
    fn test_all_key_codes_defined() {
        // AC2.4.1: Verificar que todos los KeyCode están definidos
        // Letters
        let _ = KeyCode::KeyA;
        let _ = KeyCode::KeyB;
        let _ = KeyCode::KeyC;
        let _ = KeyCode::KeyD;
        let _ = KeyCode::KeyE;
        let _ = KeyCode::KeyF;
        let _ = KeyCode::KeyG;
        let _ = KeyCode::KeyH;
        let _ = KeyCode::KeyI;
        let _ = KeyCode::KeyJ;
        let _ = KeyCode::KeyK;
        let _ = KeyCode::KeyL;
        let _ = KeyCode::KeyM;
        let _ = KeyCode::KeyN;
        let _ = KeyCode::KeyO;
        let _ = KeyCode::KeyP;
        let _ = KeyCode::KeyQ;
        let _ = KeyCode::KeyR;
        let _ = KeyCode::KeyS;
        let _ = KeyCode::KeyT;
        let _ = KeyCode::KeyU;
        let _ = KeyCode::KeyV;
        let _ = KeyCode::KeyW;
        let _ = KeyCode::KeyX;
        let _ = KeyCode::KeyY;
        let _ = KeyCode::KeyZ;

        // Numbers
        let _ = KeyCode::Digit0;
        let _ = KeyCode::Digit1;
        let _ = KeyCode::Digit2;
        let _ = KeyCode::Digit3;
        let _ = KeyCode::Digit4;
        let _ = KeyCode::Digit5;
        let _ = KeyCode::Digit6;
        let _ = KeyCode::Digit7;
        let _ = KeyCode::Digit8;
        let _ = KeyCode::Digit9;

        // Special
        let _ = KeyCode::Enter;
        let _ = KeyCode::Escape;
        let _ = KeyCode::Space;
        let _ = KeyCode::Tab;
        let _ = KeyCode::Backspace;
        let _ = KeyCode::Delete;

        // Arrows
        let _ = KeyCode::ArrowUp;
        let _ = KeyCode::ArrowDown;
        let _ = KeyCode::ArrowLeft;
        let _ = KeyCode::ArrowRight;

        // Function keys
        let _ = KeyCode::F1;
        let _ = KeyCode::F2;
        let _ = KeyCode::F3;
        let _ = KeyCode::F4;
        let _ = KeyCode::F5;
        let _ = KeyCode::F6;
        let _ = KeyCode::F7;
        let _ = KeyCode::F8;
        let _ = KeyCode::F9;
        let _ = KeyCode::F10;
        let _ = KeyCode::F11;
        let _ = KeyCode::F12;
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.4.2: Soporta modifiers (Ctrl, Shift, Alt, Meta)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_key_modifiers_empty() {
        // AC2.4.2: Debe soportar modifiers vacíos
        let modifiers = KeyModifiers::EMPTY;
        assert_eq!(modifiers.bits(), 0);
        assert!(!modifiers.contains(KeyModifiers::SHIFT));
        assert!(!modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_shift_modifier() {
        // AC2.4.2: Debe soportar Shift
        let modifiers = KeyModifiers::SHIFT;
        assert!(modifiers.contains(KeyModifiers::SHIFT));
        assert!(!modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_control_modifier() {
        // AC2.4.2: Debe soportar Control
        let modifiers = KeyModifiers::CONTROL;
        assert!(!modifiers.contains(KeyModifiers::SHIFT));
        assert!(modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_alt_modifier() {
        // AC2.4.2: Debe soportar Alt
        let modifiers = KeyModifiers::ALT;
        assert!(modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_meta_modifier() {
        // AC2.4.2: Debe soportar Meta (Windows/Cmd)
        let modifiers = KeyModifiers::META;
        assert!(modifiers.contains(KeyModifiers::META));
    }

    #[test]
    fn test_multiple_modifiers() {
        // AC2.4.2: Debe soportar combinación de modificadores
        let modifiers = KeyModifiers::SHIFT | KeyModifiers::CONTROL;
        assert!(modifiers.contains(KeyModifiers::SHIFT));
        assert!(modifiers.contains(KeyModifiers::CONTROL));
        assert!(!modifiers.contains(KeyModifiers::ALT));
    }

    #[test]
    fn test_all_modifiers() {
        // AC2.4.2: Debe soportar todos los modificadores
        let modifiers = KeyModifiers::all();
        assert!(modifiers.contains(KeyModifiers::SHIFT));
        assert!(modifiers.contains(KeyModifiers::CONTROL));
        assert!(modifiers.contains(KeyModifiers::ALT));
        assert!(modifiers.contains(KeyModifiers::META));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.4.3: on_key_press(key) → bool (rising edge)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_on_key_press_detects_rising_edge() {
        // AC2.4.3: on_key_press debe detectar rising edge (0 → 1)
        let mut sensor = KeyShortcutSensor::new();

        // Frame 1: Tecla liberada
        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: false,
        });
        assert!(!sensor.on_key_press(KeyCode::KeyA));

        // Frame 2: Tecla presionada (rising edge)
        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });
        assert!(sensor.on_key_press(KeyCode::KeyA));

        // Frame 3: Tecla sigue presionada (steady, NOT rising edge)
        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });
        assert!(!sensor.on_key_press(KeyCode::KeyA));
    }

    #[test]
    fn test_on_key_press_returns_false_for_different_key() {
        // AC2.4.3: on_key_press debe retornar false para teclas no presionadas
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        assert!(!sensor.on_key_press(KeyCode::KeyB));
        assert!(!sensor.on_key_press(KeyCode::Enter));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.4.4: on_key_combo(keys[], modifiers) → bool (AND lógico)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ctrl_delete_shortcut() {
        // AC2.4.4: Debe detectar Ctrl+Delete
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::Delete,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        assert!(sensor.on_key_combo(&[KeyCode::Delete], KeyModifiers::CONTROL));
    }

    #[test]
    fn test_ctrl_s_shortcut_for_save() {
        // AC2.4.4: Debe detectar Ctrl+S
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyS,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        assert!(sensor.on_key_combo(&[KeyCode::KeyS], KeyModifiers::CONTROL));
    }

    #[test]
    fn test_shift_ctrl_s_shortcut() {
        // AC2.4.4: Debe detectar Ctrl+Shift+S (Save As)
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyS,
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            pressed: true,
        });

        assert!(sensor.on_key_combo(
            &[KeyCode::KeyS],
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));
    }

    #[test]
    fn test_key_combo_requires_all_keys() {
        // AC2.4.4: Debe requerir que TODAS las teclas estén presionadas
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        sensor.sample(KeyEvent {
            key: KeyCode::KeyB,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        // Solo KeyA y KeyB presionados, pero el combo requiere ambos
        assert!(sensor.on_key_combo(&[KeyCode::KeyA, KeyCode::KeyB], KeyModifiers::CONTROL,));

        // Pero si solo se presiona KeyA, el combo NO debe detectarse
        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::CONTROL,
            pressed: false,
        });

        assert!(!sensor.on_key_combo(&[KeyCode::KeyA, KeyCode::KeyB], KeyModifiers::CONTROL,));
    }

    #[test]
    fn test_key_combo_requires_correct_modifiers() {
        // AC2.4.4: Debe verificar modificadores correctos
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyS,
            modifiers: KeyModifiers::SHIFT, // Shift en lugar de Control
            pressed: true,
        });

        // Con Shift, no debe detectar Ctrl+S
        assert!(!sensor.on_key_combo(&[KeyCode::KeyS], KeyModifiers::CONTROL));

        // Pero SÍ debe detectar Shift+S
        assert!(sensor.on_key_combo(&[KeyCode::KeyS], KeyModifiers::SHIFT));
    }

    #[test]
    fn test_key_combo_with_no_modifiers() {
        // AC2.4.4: Debe soportar combos sin modificadores
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::Enter,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        assert!(sensor.on_key_combo(&[KeyCode::Enter], KeyModifiers::EMPTY));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_consecutive_key_presses() {
        // Múltiples presses de la misma tecla
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::Space,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });
        assert!(sensor.on_key_press(KeyCode::Space));

        sensor.sample(KeyEvent {
            key: KeyCode::Space,
            modifiers: KeyModifiers::EMPTY,
            pressed: false,
        });

        sensor.sample(KeyEvent {
            key: KeyCode::Space,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });
        assert!(sensor.on_key_press(KeyCode::Space));
    }

    #[test]
    fn test_different_keys_independent() {
        // Teclas diferentes son independientes
        let mut sensor = KeyShortcutSensor::new();

        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        sensor.sample(KeyEvent {
            key: KeyCode::KeyB,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        sensor.sample(KeyEvent {
            key: KeyCode::KeyC,
            modifiers: KeyModifiers::EMPTY,
            pressed: true,
        });

        assert!(sensor.on_key_press(KeyCode::KeyA));
        assert!(sensor.on_key_press(KeyCode::KeyB));
        assert!(sensor.on_key_press(KeyCode::KeyC));
    }

    #[test]
    fn test_modifier_tracking_across_keys() {
        // Los modificadores se mantienen entre diferentes teclas
        let mut sensor = KeyShortcutSensor::new();

        // Ctrl+A
        sensor.sample(KeyEvent {
            key: KeyCode::KeyA,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        // Ctrl+B (mismo modifier, diferente tecla)
        sensor.sample(KeyEvent {
            key: KeyCode::KeyB,
            modifiers: KeyModifiers::CONTROL,
            pressed: true,
        });

        assert!(sensor.on_key_combo(&[KeyCode::KeyA], KeyModifiers::CONTROL));
        assert!(sensor.on_key_combo(&[KeyCode::KeyB], KeyModifiers::CONTROL));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing KeyShortcutSensor in src/sensors/key_shortcut.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test key_shortcut_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
