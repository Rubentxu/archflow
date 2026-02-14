// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Audio Actuator
//
// Actuator for audio playback with per-entity control.
// Works with AudioComponent for entity-specific settings while handling
// the actual audio context through the system.
//
// Architecture:
// - AudioActuator: Manages playback commands (play, stop, pause)
// - AudioComponent: Stores per-entity settings (volume, pitch, loop)
// - AudioSystem: Handles actual Web Audio API calls
//
// Usage:
// 1. Add AudioComponent to entity with desired settings
// 2. Connect sensor to AudioActuator via Logic Bricks
// 3. When triggered, AudioActuator sends command to AudioSystem
// ═══════════════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use alloc::vec::Vec;
use archflow_core::EntityId;

/// Audio playback command for the queue
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioCommand {
    /// Play a sound
    Play {
        /// Entity that triggered the sound
        entity: EntityId,
        /// Sound ID to play
        sound_id: u32,
    },
    /// Stop a sound
    Stop {
        /// Entity that triggered the stop
        entity: EntityId,
    },
    /// Pause a sound
    Pause {
        /// Entity that triggered the pause
        entity: EntityId,
    },
    /// Resume a paused sound
    Resume {
        /// Entity that triggered the resume
        entity: EntityId,
    },
}

/// AudioActuator for triggering sound playback
///
/// This actuator responds to sensor pulses by sending audio commands
/// to the AudioSystem for playback.
///
/// # Example
///
/// ```rust
/// use archflow_logic::actuators::AudioActuator;
/// use archflow_core::EntityId;
///
/// let mut actuator = AudioActuator::new();
/// let entity = EntityId::new(1);
///
/// // Trigger play
/// actuator.play(entity, 0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct AudioActuator {
    /// Queue of audio commands to process
    command_queue: Vec<AudioCommand>,
    /// Master volume (0.0 to 1.0)
    master_volume: f32,
    /// Is audio muted
    muted: bool,
}

impl AudioActuator {
    /// Create a new AudioActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a play command
    ///
    /// The command is queued and will be processed by AudioSystem.
    pub fn play(&mut self, entity: EntityId, sound_id: u32) {
        self.command_queue.push(AudioCommand::Play { entity, sound_id });
    }

    /// Queue a stop command
    pub fn stop(&mut self, entity: EntityId) {
        self.command_queue.push(AudioCommand::Stop { entity });
    }

    /// Queue a pause command
    pub fn pause(&mut self, entity: EntityId) {
        self.command_queue.push(AudioCommand::Pause { entity });
    }

    /// Queue a resume command
    pub fn resume(&mut self, entity: EntityId) {
        self.command_queue.push(AudioCommand::Resume { entity });
    }

    /// Get all queued commands and clear the queue
    ///
    /// This should be called by AudioSystem each frame.
    #[inline(always)]
    pub fn drain_commands(&mut self) -> Vec<AudioCommand> {
        core::mem::take(&mut self.command_queue)
    }

    /// Check if there are pending commands
    #[inline(always)]
    pub fn has_commands(&self) -> bool {
        !self.command_queue.is_empty()
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

    /// Mute/unmute audio
    #[inline(always)]
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if muted
    #[inline(always)]
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Clear all pending commands
    #[inline(always)]
    pub fn clear(&mut self) {
        self.command_queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::EntityId;

    #[test]
    fn test_audio_actuator_play() {
        let mut actuator = AudioActuator::new();
        let entity = EntityId::new(1);
        
        actuator.play(entity, 0);
        
        assert!(actuator.has_commands());
        let commands = actuator.drain_commands();
        assert_eq!(commands.len(), 1);
        if let AudioCommand::Play { entity: e, sound_id: s } = commands[0] {
            assert_eq!(e, entity);
            assert_eq!(s, 0);
        } else {
            panic!("Expected Play command");
        }
    }

    #[test]
    fn test_audio_actuator_drain() {
        let mut actuator = AudioActuator::new();
        actuator.play(EntityId::new(1), 0);
        actuator.play(EntityId::new(2), 1);
        
        let commands = actuator.drain_commands();
        assert_eq!(commands.len(), 2);
        assert!(!actuator.has_commands());
    }

    #[test]
    fn test_master_volume() {
        let mut actuator = AudioActuator::new();
        
        actuator.set_master_volume(0.5);
        assert_eq!(actuator.master_volume(), 0.5);
        
        // Clamp test
        actuator.set_master_volume(1.5);
        assert_eq!(actuator.master_volume(), 1.0);
        
        actuator.set_master_volume(-0.5);
        assert_eq!(actuator.master_volume(), 0.0);
    }

    #[test]
    fn test_mute() {
        let mut actuator = AudioActuator::new();
        
        assert!(!actuator.is_muted());
        actuator.set_muted(true);
        assert!(actuator.is_muted());
    }
}
