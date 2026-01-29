//! WASM bindings for keyboard module
//!
//! Provides WebAssembly bindings for keyboard nudge and shortcut operations

use crate::canvas::Canvas;
use crate::keyboard::{AutoRepeatConfig, KeyboardNudgeSystem, NudgeDirection, PrecisionLevel};
use archflow_core::Vec2;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// WASM-exposed keyboard nudge system
#[wasm_bindgen]
pub struct JsKeyboardNudgeSystem {
    inner: KeyboardNudgeSystem,
}

#[wasm_bindgen]
impl JsKeyboardNudgeSystem {
    /// Creates a new keyboard nudge system with default settings
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: KeyboardNudgeSystem::new(),
        }
    }

    /// Sets the precision level
    #[wasm_bindgen]
    pub fn set_precision(&mut self, level: &str) {
        let precision = match level {
            "fast" => PrecisionLevel::Fast,
            "precise" => PrecisionLevel::Precise,
            _ => PrecisionLevel::Normal,
        };
        self.inner.set_precision(precision);
    }

    /// Gets the current precision level
    #[wasm_bindgen]
    pub fn get_precision(&self) -> String {
        match self.inner.precision_level() {
            PrecisionLevel::Normal => "normal".to_string(),
            PrecisionLevel::Fast => "fast".to_string(),
            PrecisionLevel::Precise => "precise".to_string(),
        }
    }

    /// Updates precision based on modifier keys
    #[wasm_bindgen]
    pub fn update_precision(&mut self, shift_pressed: bool, alt_pressed: bool) {
        self.inner.update_precision(shift_pressed, alt_pressed);
    }

    /// Sets the auto-repeat configuration
    #[wasm_bindgen(js_name = setAutoRepeatConfig)]
    pub fn set_auto_repeat_config(&mut self, initial_delay_ms: u32, repeat_interval_ms: u32) {
        let config = AutoRepeatConfig {
            initial_delay: std::time::Duration::from_millis(initial_delay_ms as u64),
            repeat_interval: std::time::Duration::from_millis(repeat_interval_ms as u64),
        };
        self.inner.set_auto_repeat_config(config);
    }

    /// Gets distance for a precision level
    #[wasm_bindgen(js_name = getPrecisionDistance)]
    pub fn get_precision_distance(level: &str) -> f32 {
        match level {
            "normal" => PrecisionLevel::Normal.distance(),
            "fast" => PrecisionLevel::Fast.distance(),
            "precise" => PrecisionLevel::Precise.distance(),
            _ => 1.0,
        }
    }
}

/// WASM-exposed keyboard shortcut handler
#[wasm_bindgen]
pub struct JsKeyboardHandler {
    canvas: Option<RefCell<Canvas>>,
    nudge_system: JsKeyboardNudgeSystem,
}

#[wasm_bindgen]
impl JsKeyboardHandler {
    /// Creates a new keyboard handler
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            canvas: None,
            nudge_system: JsKeyboardNudgeSystem::new(),
        }
    }

    /// Sets the canvas reference for keyboard operations
    #[wasm_bindgen]
    pub fn set_canvas(&mut self, canvas: &Canvas) {
        self.canvas = Some(RefCell::new(canvas.clone()));
    }

    /// Handles a key down event
    #[wasm_bindgen]
    pub fn handle_keydown(&mut self, key: &str, shift: bool, ctrl: bool) -> bool {
        // Handle common shortcuts
        match key {
            "z" if ctrl && shift => {
                // Redo
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().redo();
                }
                return true;
            }
            "z" if ctrl => {
                // Undo
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().undo();
                }
                return true;
            }
            "y" if ctrl => {
                // Redo
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().redo();
                }
                return true;
            }
            "a" if ctrl => {
                // Select all
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().select_all();
                }
                return true;
            }
            "Delete" | "Backspace" => {
                // Delete selected
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().delete_selected();
                }
                return true;
            }
            "Escape" => {
                // Clear selection
                if let Some(ref canvas) = self.canvas {
                    canvas.borrow_mut().clear_selection();
                }
                return true;
            }
            "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => {
                // Nudge selection
                self.nudge_system.update_precision(shift, false);
                let direction = match key {
                    "ArrowUp" => NudgeDirection::Up,
                    "ArrowDown" => NudgeDirection::Down,
                    "ArrowLeft" => NudgeDirection::Left,
                    "ArrowRight" => NudgeDirection::Right,
                    _ => NudgeDirection::Up,
                };
                if let Some(ref canvas) = self.canvas {
                    self.inner
                        .nudge_selection(&mut canvas.borrow_mut(), direction);
                }
                return true;
            }
            _ => {}
        }
        false
    }

    /// Handles a key up event
    #[wasm_bindgen]
    pub fn handle_keyup(&mut self, key: &str) {
        // Reset precision on key up
        self.nudge_system.update_precision(false, false);
    }

    /// Nudges the selection in a direction
    #[wasm_bindgen]
    pub fn nudge(&mut self, direction: String, times: i32) {
        let dir = match direction.as_str() {
            "up" => NudgeDirection::Up,
            "down" => NudgeDirection::Down,
            "left" => NudgeDirection::Left,
            "right" => NudgeDirection::Right,
            _ => return,
        };

        if let Some(ref canvas) = self.canvas {
            for _ in 0..times.abs() as usize {
                self.inner.nudge_selection(&mut canvas.borrow_mut(), dir);
            }
        }
    }

    /// Gets the nudge system for external use
    #[wasm_bindgen(getter = nudgeSystem)]
    pub fn get_nudge_system(&self) -> &JsKeyboardNudgeSystem {
        &self.nudge_system
    }
}

impl JsKeyboardHandler {
    /// Internal reference to the canvas
    fn canvas_ref(&self) -> Option<&RefCell<Canvas>> {
        self.canvas.as_ref()
    }
}

/// TypeScript definitions
pub const KEYBOARD_TYPES: &str = r#"
/**
 * Keyboard Nudge System for WASM
 */
export class JsKeyboardNudgeSystem {
    constructor();
    setPrecision(level: 'normal' | 'fast' | 'precise'): void;
    getPrecision(): 'normal' | 'fast' | 'precise';
    updatePrecision(shiftPressed: boolean, altPressed: boolean): void;
    setAutoRepeatConfig(initialDelayMs: number, repeatIntervalMs: number): void;
    static getPrecisionDistance(level: 'normal' | 'fast' | 'precise'): number;
}

/**
 * Keyboard Handler for WASM
 */
export class JsKeyboardHandler {
    constructor();
    setCanvas(canvas: Canvas): void;
    handleKeydown(key: string, shift: boolean, ctrl: boolean): boolean;
    handleKeyup(key: string): void;
    nudge(direction: 'up' | 'down' | 'left' | 'right', times: number): void;
    readonly nudgeSystem: JsKeyboardNudgeSystem;
}
"#;

/// Get TypeScript definitions for keyboard
#[wasm_bindgen]
pub fn get_keyboard_typescript_definitions() -> String {
    KEYBOARD_TYPES.to_string()
}
