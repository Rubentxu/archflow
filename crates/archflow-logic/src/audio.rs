// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Audio System
//
// System for processing audio commands and managing sound playback state.
// This module provides the logic layer; actual Web Audio API integration
// happens in the WASM bridge.
//
// Architecture:
// - AudioSystem: Process commands from AudioActuator
// - AudioContext: Managed by WASM bridge (web-sys)
// - Sound registry: Maps sound IDs to loaded audio buffers
//
// Usage:
// 1. AudioActuator queues commands based on sensor triggers
// 2. Each frame, AudioSystem processes the command queue
// 3. Commands are forwarded to Web Audio API via bridge callbacks
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use alloc::string::String;
use archflow_core::EntityId;

use crate::actuators::audio::AudioCommand;

/// Sound information for the registry
#[derive(Clone, Debug)]
pub struct SoundInfo {
    /// Unique sound identifier
    pub id: u32,
    /// Human-readable name
    pub name: String,
    /// Duration in seconds
    pub duration: f32,
}

/// AudioSystem for processing audio commands
///
/// This system processes audio commands each frame and maintains
/// the sound registry. Actual audio playback is handled by the
/// WASM bridge through callbacks.
///
/// # Example
///
/// ```rust
/// use archflow_logic::audio::AudioSystem;
/// use archflow_core::EntityId;
///
/// let mut system = AudioSystem::new();
/// system.update();
/// ```
#[derive(Clone, Debug, Default)]
pub struct AudioSystem {
    /// Registered sounds
    sounds: Vec<SoundInfo>,
    /// Currently playing sounds (entity -> sound_id)
    playing: Vec<(EntityId, u32)>,
    /// Paused sounds (entity -> sound_id)
    paused: Vec<(EntityId, u32)>,
    /// Master volume
    master_volume: f32,
    /// Is audio muted
    muted: bool,
    /// Callback for play (entity_id, sound_id, volume, pitch, loop)
    play_callback: Option<fn(EntityId, u32, f32, f32, bool)>,
    /// Callback for stop (entity_id)
    stop_callback: Option<fn(EntityId)>,
    /// Callback for pause (entity_id)
    pause_callback: Option<fn(EntityId)>,
    /// Callback for resume (entity_id)
    resume_callback: Option<fn(EntityId)>,
}

impl AudioSystem {
    /// Create a new AudioSystem
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a sound for playback
    ///
    /// Returns the sound ID that can be used to play this sound.
    pub fn register_sound(&mut self, name: &str, duration: f32) -> u32 {
        let id = self.sounds.len() as u32;
        self.sounds.push(SoundInfo {
            id,
            name: String::from(name),
            duration,
        });
        id
    }

    /// Get sound info by ID
    #[inline(always)]
    pub fn get_sound(&self, id: u32) -> Option<&SoundInfo> {
        self.sounds.get(id as usize)
    }

    /// Get all registered sounds
    #[inline(always)]
    pub fn sounds(&self) -> &[SoundInfo] {
        &self.sounds
    }

    /// Process audio commands
    ///
    /// This should be called each frame with commands from AudioActuator.
    pub fn process_commands(&mut self, commands: Vec<AudioCommand>) {
        for cmd in commands {
            match cmd {
                AudioCommand::Play { entity, sound_id } => {
                    self.handle_play(entity, sound_id);
                }
                AudioCommand::Stop { entity } => {
                    self.handle_stop(entity);
                }
                AudioCommand::Pause { entity } => {
                    self.handle_pause(entity);
                }
                AudioCommand::Resume { entity } => {
                    self.handle_resume(entity);
                }
            }
        }
    }

    fn handle_play(&mut self, entity: EntityId, sound_id: u32) {
        // Remove from paused if there
        self.paused.retain(|(e, _)| *e != entity);
        // Add to playing
        self.playing.retain(|(e, _)| *e != entity);
        self.playing.push((entity, sound_id));
        
        // Trigger callback
        if let Some(callback) = self.play_callback {
            callback(entity, sound_id, self.master_volume, 1.0, false);
        }
    }

    fn handle_stop(&mut self, entity: EntityId) {
        self.playing.retain(|(e, _)| *e != entity);
        self.paused.retain(|(e, _)| *e != entity);
        
        if let Some(callback) = self.stop_callback {
            callback(entity);
        }
    }

    fn handle_pause(&mut self, entity: EntityId) {
        if let Some(pos) = self.playing.iter().position(|(e, _)| *e == entity) {
            let (_, sound_id) = self.playing.remove(pos);
            self.paused.push((entity, sound_id));
            
            if let Some(callback) = self.pause_callback {
                callback(entity);
            }
        }
    }

    fn handle_resume(&mut self, entity: EntityId) {
        if let Some(pos) = self.paused.iter().position(|(e, _)| *e == entity) {
            let (_, sound_id) = self.paused.remove(pos);
            self.playing.push((entity, sound_id));
            
            if let Some(callback) = self.resume_callback {
                callback(entity);
            }
        }
    }

    /// Check if a sound is currently playing for an entity
    #[inline(always)]
    pub fn is_playing(&self, entity: EntityId) -> bool {
        self.playing.iter().any(|(e, _)| *e == entity)
    }

    /// Check if a sound is paused for an entity
    #[inline(always)]
    pub fn is_paused(&self, entity: EntityId) -> bool {
        self.paused.iter().any(|(e, _)| *e == entity)
    }

    /// Set master volume
    #[inline(always)]
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Get master volume
    #[inline(always)]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set muted state
    #[inline(always)]
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if muted
    #[inline(always)]
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Set play callback
    pub fn on_play(&mut self, callback: fn(EntityId, u32, f32, f32, bool)) {
        self.play_callback = Some(callback);
    }

    /// Set stop callback
    pub fn on_stop(&mut self, callback: fn(EntityId)) {
        self.stop_callback = Some(callback);
    }

    /// Set pause callback
    pub fn on_pause(&mut self, callback: fn(EntityId)) {
        self.pause_callback = Some(callback);
    }

    /// Set resume callback
    pub fn on_resume(&mut self, callback: fn(EntityId)) {
        self.resume_callback = Some(callback);
    }

    /// Get number of playing sounds
    #[inline(always)]
    pub fn playing_count(&self) -> usize {
        self.playing.len()
    }

    /// Get number of paused sounds
    #[inline(always)]
    pub fn paused_count(&self) -> usize {
        self.paused.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_sound() {
        let mut system = AudioSystem::new();
        let id = system.register_sound("click", 0.5);
        assert_eq!(id, 0);
        
        let sound = system.get_sound(0).unwrap();
        assert_eq!(sound.name, "click");
    }

    #[test]
    fn test_play_stop() {
        let mut system = AudioSystem::new();
        let entity = EntityId::new(1);
        
        system.register_sound("click", 0.5);
        
        // Play
        system.process_commands(vec![AudioCommand::Play { entity, sound_id: 0 }]);
        assert!(system.is_playing(entity));
        
        // Stop
        system.process_commands(vec![AudioCommand::Stop { entity }]);
        assert!(!system.is_playing(entity));
    }

    #[test]
    fn test_pause_resume() {
        let mut system = AudioSystem::new();
        let entity = EntityId::new(1);
        
        system.register_sound("click", 0.5);
        
        // Play then pause
        system.process_commands(vec![AudioCommand::Play { entity, sound_id: 0 }]);
        system.process_commands(vec![AudioCommand::Pause { entity }]);
        assert!(!system.is_playing(entity));
        assert!(system.is_paused(entity));
        
        // Resume
        system.process_commands(vec![AudioCommand::Resume { entity }]);
        assert!(system.is_playing(entity));
        assert!(!system.is_paused(entity));
    }

    #[test]
    fn test_master_volume() {
        let mut system = AudioSystem::new();
        
        system.set_master_volume(0.5);
        assert_eq!(system.master_volume(), 0.5);
        
        // Clamp
        system.set_master_volume(1.5);
        assert_eq!(system.master_volume(), 1.0);
    }
}
