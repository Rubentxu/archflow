// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - Advanced Components
//
// This module provides advanced ECS components for JavaScript:
// - Audio: sound playback and spatial audio
// - Physics: colliders and rigid bodies
// - UI: buttons, text, images
// - Named: entity naming for debugging
//
// Architecture: EPIC-WASM-105 - Advanced Components
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use wasm_bindgen::prelude::*;

// ============================================================================
// Audio Component
// ============================================================================

/// Audio component for sound playback and spatial audio
#[wasm_bindgen]
pub struct AudioComponent {
    /// Audio URL or identifier
    url: String,
    /// Whether audio should loop
    looped: bool,
    /// Volume (0.0 - 1.0)
    volume: f32,
    /// Is spatial/positional audio
    positional: bool,
}

#[wasm_bindgen]
impl AudioComponent {
    /// Load audio from URL
    #[wasm_bindgen]
    pub fn load(url: &str) -> Self {
        Self {
            url: url.to_string(),
            looped: false,
            volume: 1.0,
            positional: false,
        }
    }

    /// Set loop mode
    #[wasm_bindgen]
    pub fn looped(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }

    /// Set volume
    #[wasm_bindgen]
    pub fn volume(mut self, vol: f32) -> Self {
        self.volume = vol.max(0.0).min(1.0);
        self
    }

    /// Enable positional audio
    #[wasm_bindgen]
    pub fn positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Get component type identifier
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "audio".to_string()
    }
}

/// Audio factory
#[wasm_bindgen]
pub struct AudioFactory;

#[wasm_bindgen]
impl AudioFactory {
    /// Load audio from URL
    #[wasm_bindgen]
    pub fn load(url: &str) -> AudioComponent {
        AudioComponent::load(url)
    }

    /// Create positional audio
    #[wasm_bindgen]
    pub fn positional(url: &str) -> AudioComponent {
        AudioComponent::load(url).positional(true)
    }
}

// ============================================================================
// Physics Component
// ============================================================================

/// Physics collider component
#[wasm_bindgen]
pub struct PhysicsCollider {
    /// Collider type: 0 = box, 1 = circle, 2 = polygon
    collider_type: u8,
    /// Width (for box)
    width: f32,
    /// Height (for box)
    height: f32,
    /// Radius (for circle)
    radius: f32,
    /// Is trigger (no collision response)
    is_trigger: bool,
}

#[wasm_bindgen]
impl PhysicsCollider {
    /// Create box collider
    #[wasm_bindgen]
    pub fn box_shape(width: f32, height: f32) -> Self {
        Self {
            collider_type: 0,
            width,
            height,
            radius: 0.0,
            is_trigger: false,
        }
    }

    /// Create circle collider
    #[wasm_bindgen]
    pub fn circle(radius: f32) -> Self {
        Self {
            collider_type: 1,
            width: 0.0,
            height: 0.0,
            radius,
            is_trigger: false,
        }
    }

    /// Set as trigger
    #[wasm_bindgen]
    pub fn trigger(mut self, is_trigger: bool) -> Self {
        self.is_trigger = is_trigger;
        self
    }

    /// Get component type
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "physics_collider".to_string()
    }
}

/// Physics rigid body component
#[wasm_bindgen]
pub struct PhysicsBody {
    /// Mass (0 = static)
    mass: f32,
    /// Restitution (bounciness)
    restitution: f32,
    /// Friction
    friction: f32,
    /// Is static
    is_static: bool,
}

#[wasm_bindgen]
impl PhysicsBody {
    /// Create rigid body
    #[wasm_bindgen]
    pub fn new() -> Self {
        Self {
            mass: 1.0,
            restitution: 0.3,
            friction: 0.5,
            is_static: false,
        }
    }

    /// Set mass
    #[wasm_bindgen]
    pub fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        if mass == 0.0 {
            self.is_static = true;
        }
        self
    }

    /// Set as static
    #[wasm_bindgen]
    pub fn static_body(mut self) -> Self {
        self.mass = 0.0;
        self.is_static = true;
        self
    }

    /// Set restitution
    #[wasm_bindgen]
    pub fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.max(0.0).min(1.0);
        self
    }

    /// Set friction
    #[wasm_bindgen]
    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0).min(1.0);
        self
    }

    /// Get component type
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "physics_body".to_string()
    }
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics factory
#[wasm_bindgen]
pub struct PhysicsFactory;

#[wasm_bindgen]
impl PhysicsFactory {
    /// Create box collider
    #[wasm_bindgen]
    pub fn collider_box(width: f32, height: f32) -> PhysicsCollider {
        PhysicsCollider::box_shape(width, height)
    }

    /// Create circle collider
    #[wasm_bindgen]
    pub fn collider_circle(radius: f32) -> PhysicsCollider {
        PhysicsCollider::circle(radius)
    }

    /// Create rigid body
    #[wasm_bindgen]
    pub fn rigid_body() -> PhysicsBody {
        PhysicsBody::new()
    }
}

// ============================================================================
// UI Component
// ============================================================================

/// UI Text component
#[wasm_bindgen]
pub struct UiText {
    /// Text content
    content: String,
    /// Font size
    font_size: f32,
    /// Text color
    color: u32,
}

#[wasm_bindgen]
impl UiText {
    /// Create text component
    #[wasm_bindgen]
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            font_size: 16.0,
            color: 0x000000FF, // Black
        }
    }

    /// Set font size
    #[wasm_bindgen]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set color
    #[wasm_bindgen]
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// Get component type
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "ui_text".to_string()
    }
}

/// UI Button component
#[wasm_bindgen]
pub struct UiButton {
    /// Button label
    label: String,
    /// Font size
    font_size: f32,
    /// Background color
    bg_color: u32,
    /// Text color
    text_color: u32,
}

#[wasm_bindgen]
impl UiButton {
    /// Create button component
    #[wasm_bindgen]
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            font_size: 16.0,
            bg_color: 0xCCCCCCFF,   // Light gray
            text_color: 0x000000FF, // Black
        }
    }

    /// Set font size
    #[wasm_bindgen]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Get component type
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "ui_button".to_string()
    }
}

/// UI factory
#[wasm_bindgen]
pub struct UiFactory;

#[wasm_bindgen]
impl UiFactory {
    /// Create text component
    #[wasm_bindgen]
    pub fn text(content: &str) -> UiText {
        UiText::new(content)
    }

    /// Create button component
    #[wasm_bindgen]
    pub fn button(label: &str) -> UiButton {
        UiButton::new(label)
    }
}

// ============================================================================
// Named Component
// ============================================================================

/// Named component - gives entities a name for debugging
#[wasm_bindgen]
pub struct NamedComponent {
    /// Entity name
    name: String,
}

#[wasm_bindgen]
impl NamedComponent {
    /// Create named component
    #[wasm_bindgen]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Get component type
    #[wasm_bindgen]
    pub fn component_type(&self) -> String {
        "named".to_string()
    }
}

/// Named factory
#[wasm_bindgen]
pub struct NamedFactory;

#[wasm_bindgen]
impl NamedFactory {
    /// Create named component
    #[wasm_bindgen]
    pub fn new(name: &str) -> NamedComponent {
        NamedComponent::new(name)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_factory() {
        let audio = AudioFactory::load("sounds/jump.mp3");
        assert_eq!(audio.component_type(), "audio");
    }

    #[test]
    fn test_physics_collider() {
        let collider = PhysicsFactory::collider_box(50.0, 50.0);
        assert_eq!(collider.component_type(), "physics_collider");
    }

    #[test]
    fn test_ui_text() {
        let text = UiFactory::text("Hello");
        assert_eq!(text.component_type(), "ui_text");
    }

    #[test]
    fn test_named() {
        let named = NamedFactory::new("player");
        assert_eq!(named.component_type(), "named");
    }
}
