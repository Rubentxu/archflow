//! Timeline - Animation sequencing system
//!
//! Provides GSAP Timeline-style sequencing for coordinating multiple animations:
//!
//! # Example
//!
//! ```text
//! let timeline = Timeline::new()
//!     .add(shape1.animate().to(100.0, 100.0).duration(500))
//!     .add(shape2.animate().to(200.0, 200.0).duration(300), "-=200") // overlap
//!     .add_label("halfway")
//!     .add(shape3.animate().rotate(90.0).duration(400), "halfway+=50")
//!     .play();
//! ```

use super::{AnimationManager, FloatAnimation, PositionAnimation};
use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Position marker in a timeline
///
/// Represents a labeled point that can be referenced for positioning animations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineLabel {
    /// Label name
    pub name: String,
    /// Position in timeline (seconds from start)
    pub position: f64,
}

impl TimelineLabel {
    /// Create a new timeline label
    pub fn new(name: impl Into<String>, position: f64) -> Self {
        Self {
            name: name.into(),
            position,
        }
    }
}

/// Animation entry in a timeline
///
/// Represents an animation with its timing information relative to the timeline
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    /// Animation ID
    pub id: EntityId,
    /// Target entity ID
    pub target_id: EntityId,
    /// Start time relative to timeline (seconds)
    pub start_time: f64,
    /// Duration (seconds)
    pub duration: f64,
    /// Whether this entry has been started
    pub started: bool,
    /// Whether this entry is complete
    pub complete: bool,
    /// Entry type (position or float animation)
    pub entry_type: TimelineEntryType,
}

/// Type of timeline entry
#[derive(Debug, Clone)]
pub enum TimelineEntryType {
    /// Position animation
    Position(PositionAnimation),
    /// Float animation (scale, rotation, opacity, etc.)
    Float(FloatAnimation),
}

/// Timeline position specification
///
/// Allows flexible positioning of animations relative to other points
#[derive(Debug, Clone, PartialEq)]
pub enum TimelinePosition {
    /// Absolute position in seconds
    Absolute(f64),
    /// Relative to previous animation end (+=seconds)
    Relative(f64),
    /// Relative to previous animation start (-=seconds for overlap)
    Overlap(f64),
    /// Relative to a label
    Label {
        /// Label name
        name: String,
        /// Offset from label (+=seconds or -=seconds)
        offset: f64,
    },
}

impl TimelinePosition {
    /// Parse a position string (GSAP-style)
    ///
    /// # Examples
    /// ```
    /// # use archflow_core::TimelinePosition;
    /// // Absolute position
    /// let pos = TimelinePosition::parse("1.5").unwrap();
    ///
    /// // Relative to previous end
    /// let pos = TimelinePosition::parse("+=0.5").unwrap();
    ///
    /// // Overlap with previous
    /// let pos = TimelinePosition::parse("-=0.2").unwrap();
    ///
    /// // Relative to label
    /// let pos = TimelinePosition::parse("myLabel+=0.3").unwrap();
    /// ```
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // Check for relative/overlap prefixes
        if input.starts_with("+=") {
            let offset: f64 = input[2..].parse().ok()?;
            return Some(Self::Relative(offset));
        }

        if input.starts_with("-=") {
            let offset: f64 = input[2..].parse().ok()?;
            return Some(Self::Overlap(offset));
        }

        // Check for label references
        if let Some(pos) = input.find('+') {
            let label_name = input[..pos].to_string();
            let offset_str = &input[pos..];
            if offset_str.starts_with("+=") {
                let offset: f64 = offset_str[2..].parse().ok()?;
                return Some(Self::Label {
                    name: label_name,
                    offset,
                });
            }
        }

        if let Some(pos) = input.find('-') {
            let label_name = input[..pos].to_string();
            let offset_str = &input[pos..];
            if offset_str.starts_with("-=") {
                let offset: f64 = offset_str[2..].parse().ok()?;
                return Some(Self::Label {
                    name: label_name,
                    offset,
                });
            }
        }

        // Try absolute position
        if let Ok(pos) = input.parse::<f64>() {
            return Some(Self::Absolute(pos));
        }

        // Check if it's just a label name
        if !input.is_empty() && !input.contains('+') && !input.contains('-') {
            return Some(Self::Label {
                name: input.to_string(),
                offset: 0.0,
            });
        }

        None
    }

    /// Calculate actual position given current timeline state
    pub fn calculate_position(
        &self,
        previous_end: f64,
        labels: &HashMap<String, f64>,
    ) -> Option<f64> {
        match self {
            Self::Absolute(pos) => Some(*pos),
            Self::Relative(offset) => Some(previous_end + offset),
            Self::Overlap(offset) => Some(previous_end - offset),
            Self::Label { name, offset } => {
                let label_pos = labels.get(name)?;
                Some(label_pos + offset)
            }
        }
    }
}

/// Timeline for sequencing animations
///
/// Manages a sequence of animations with precise timing control,
/// inspired by GSAP Timeline and Anime.js.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// Animation manager reference
    manager: Arc<Mutex<AnimationManager>>,
    /// Timeline entries
    entries: Vec<TimelineEntry>,
    /// Timeline labels
    labels: Vec<TimelineLabel>,
    /// Current timeline position (seconds)
    current_time: f64,
    /// Total timeline duration
    duration: f64,
    /// Whether timeline is playing
    playing: bool,
    /// Timeline playback speed
    time_scale: f32,
    /// Number of loops
    loop_count: u32,
    /// Current loop iteration
    current_loop: u32,
}

impl Timeline {
    /// Create a new empty timeline
    ///
    /// # Example
    /// ```text
    /// let timeline = Timeline::new();
    /// ```
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(AnimationManager::new())),
            entries: Vec::new(),
            labels: Vec::new(),
            current_time: 0.0,
            duration: 0.0,
            playing: false,
            time_scale: 1.0,
            loop_count: 0,
            current_loop: 0,
        }
    }

    /// Set the animation manager (for using existing manager)
    pub fn with_manager(mut self, manager: Arc<Mutex<AnimationManager>>) -> Self {
        self.manager = manager;
        self
    }

    /// Add an animation to the timeline at the default position (after previous)
    ///
    /// # Arguments
    /// * `animation` - PositionAnimation to add
    ///
    /// # Returns
    /// Self for method chaining
    pub fn add_position(mut self, animation: PositionAnimation) -> Self {
        let duration_sec = animation.config.duration.as_secs_f64();
        let start_time = self.duration;

        let entry = TimelineEntry {
            id: animation.id,
            target_id: animation.target_id,
            start_time,
            duration: duration_sec,
            started: false,
            complete: false,
            entry_type: TimelineEntryType::Position(animation),
        };

        self.duration = (start_time + duration_sec).max(self.duration);
        self.entries.push(entry);
        self
    }

    /// Add a float animation to the timeline
    pub fn add_float(mut self, animation: FloatAnimation) -> Self {
        let duration_sec = animation.config.duration.as_secs_f64();
        let start_time = self.duration;

        let entry = TimelineEntry {
            id: animation.id,
            target_id: animation.target_id,
            start_time,
            duration: duration_sec,
            started: false,
            complete: false,
            entry_type: TimelineEntryType::Float(animation),
        };

        self.duration = (start_time + duration_sec).max(self.duration);
        self.entries.push(entry);
        self
    }

    /// Add an animation at a specific timeline position
    ///
    /// # Arguments
    /// * `animation` - PositionAnimation to add
    /// * `position` - Timeline position specifier
    ///
    /// # Example
    /// ```text
    /// timeline.add(shape1.animate().to(100.0, 100.0).duration(500))
    ///       .add(shape2.animate().to(200.0, 200.0).duration(300), "-=200")
    ///       .add(shape3.animate().rotate(90.0).duration(400), "myLabel+=50");
    /// ```
    pub fn add_position_at(mut self, animation: PositionAnimation, position: &str) -> Self {
        let pos = TimelinePosition::parse(position).unwrap_or_else(|| {
            // Default to relative if parsing fails
            TimelinePosition::Relative(0.0)
        });

        let duration_sec = animation.config.duration.as_secs_f64();
        let previous_end = self
            .entries
            .last()
            .map_or(0.0, |e| e.start_time + e.duration);

        let labels_map: HashMap<String, f64> = self
            .labels
            .iter()
            .map(|l| (l.name.clone(), l.position))
            .collect();

        let start_time = pos
            .calculate_position(previous_end, &labels_map)
            .unwrap_or(previous_end);

        let entry = TimelineEntry {
            id: animation.id,
            target_id: animation.target_id,
            start_time,
            duration: duration_sec,
            started: false,
            complete: false,
            entry_type: TimelineEntryType::Position(animation),
        };

        self.duration = (start_time + duration_sec).max(self.duration);
        self.entries.push(entry);
        self
    }

    /// Add a label at the current timeline position
    ///
    /// # Arguments
    /// * `name` - Label name
    ///
    /// # Example
    /// ```text
    /// timeline.add_label("halfway")
    ///       .add(shape.animate().to(100.0, 100.0), "halfway+=50");
    /// ```
    pub fn add_label(mut self, name: impl Into<String>) -> Self {
        let label = TimelineLabel::new(name, self.current_time);
        self.labels.push(label);
        self
    }

    /// Add a label at a specific position
    ///
    /// # Arguments
    /// * `name` - Label name
    /// * `position` - Position in timeline (seconds)
    pub fn add_label_at(mut self, name: impl Into<String>, position: f64) -> Self {
        let label = TimelineLabel::new(name, position);
        self.labels.push(label);
        self
    }

    /// Start playing the timeline
    pub fn play(mut self) -> TimelineHandle {
        self.playing = true;
        self.current_time = 0.0;
        self.current_loop = 0;

        // Clone manager reference before moving self
        let manager_ref = self.manager.clone();

        // Add all animations to the manager
        let mut manager = manager_ref.lock().unwrap();

        for entry in &mut self.entries {
            match &entry.entry_type {
                TimelineEntryType::Position(anim) => {
                    manager.add_position_animation(anim.clone());
                }
                TimelineEntryType::Float(anim) => {
                    manager.add_float_animation(anim.clone());
                }
            }
        }

        TimelineHandle {
            timeline: Arc::new(Mutex::new(self)),
        }
    }

    /// Pause the timeline
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Resume the timeline
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// Stop and reset the timeline
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
        self.current_loop = 0;

        // Remove all animations from manager
        let mut manager = self.manager.lock().unwrap();
        for entry in &self.entries {
            manager.remove_animation(entry.id);
        }

        // Reset entry states
        for entry in &mut self.entries {
            entry.started = false;
            entry.complete = false;
        }
    }

    /// Set timeline playback speed
    ///
    /// # Arguments
    /// * `scale` - Time scale (1.0 = normal, 2.0 = 2x speed, 0.5 = half speed)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get timeline playback speed
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Set number of loops
    ///
    /// # Arguments
    /// * `count` - Number of additional loops after the first (0 = play once, u32::MAX = infinite)
    pub fn set_loops(mut self, count: u32) -> Self {
        self.loop_count = count;
        self
    }

    /// Update timeline by delta time
    ///
    /// # Arguments
    /// * `delta` - Time delta in seconds
    ///
    /// # Returns
    /// true if timeline is complete
    pub fn update(&mut self, delta: f64) -> bool {
        if !self.playing {
            return false;
        }

        let scaled_delta = delta * self.time_scale as f64;
        self.current_time += scaled_delta;

        // Update animations based on current time
        let _manager = self.manager.lock().unwrap();

        for entry in &mut self.entries {
            // Check if animation should start
            if !entry.started && self.current_time >= entry.start_time {
                entry.started = true;

                // Start the animation
                match &mut entry.entry_type {
                    TimelineEntryType::Position(anim) => {
                        anim.start();
                    }
                    TimelineEntryType::Float(anim) => {
                        anim.start();
                    }
                }
            }

            // Update running animation
            if entry.started && !entry.complete {
                let elapsed = self.current_time - entry.start_time;

                // Calculate progress (0.0 to 1.0)
                let progress = (elapsed / entry.duration).min(1.0);

                if progress >= 1.0 {
                    entry.complete = true;
                }
            }
        }

        // Check for loop completion
        if self.current_time >= self.duration {
            // Check if we should loop again
            // loop_count = 0 means "play once" (no additional loops)
            // loop_count = N means "play N+1 times" (initial + N repeats)
            // loop_count = u32::MAX means infinite loops
            if self.current_loop < self.loop_count {
                // Has more loops to go - reset and continue
                self.current_time = 0.0;
                self.current_loop += 1;
                false
            } else {
                // All loops complete - this is the final completion
                true
            }
        } else {
            false
        }
    }

    /// Get current timeline position
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Get total timeline duration
    pub fn duration(&self) -> f64 {
        self.duration
    }

    /// Get timeline progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.duration > 0.0 {
            (self.current_time / self.duration).min(1.0)
        } else {
            1.0
        }
    }

    /// Check if timeline is playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Check if timeline is complete
    pub fn is_complete(&self) -> bool {
        self.current_time >= self.duration
            && self.current_loop >= self.loop_count
            && self.loop_count != u32::MAX
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for controlling a running timeline
#[derive(Clone)]
pub struct TimelineHandle {
    timeline: Arc<Mutex<Timeline>>,
}

impl TimelineHandle {
    /// Pause the timeline
    pub fn pause(&self) {
        let mut timeline = self.timeline.lock().unwrap();
        timeline.pause();
    }

    /// Resume the timeline
    pub fn resume(&self) {
        let mut timeline = self.timeline.lock().unwrap();
        timeline.resume();
    }

    /// Stop and reset the timeline
    pub fn stop(&self) {
        let mut timeline = self.timeline.lock().unwrap();
        timeline.stop();
    }

    /// Set playback speed
    pub fn set_time_scale(&self, scale: f32) {
        let mut timeline = self.timeline.lock().unwrap();
        timeline.set_time_scale(scale);
    }

    /// Get current progress
    pub fn progress(&self) -> f64 {
        let timeline = self.timeline.lock().unwrap();
        timeline.progress()
    }

    /// Check if timeline is complete
    pub fn is_complete(&self) -> bool {
        let timeline = self.timeline.lock().unwrap();
        timeline.is_complete()
    }

    /// Check if timeline is playing
    pub fn is_playing(&self) -> bool {
        let timeline = self.timeline.lock().unwrap();
        timeline.is_playing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new();

        assert_eq!(timeline.current_time, 0.0);
        assert_eq!(timeline.duration, 0.0);
        assert!(!timeline.playing);
        assert!(timeline.entries.is_empty());
    }

    #[test]
    fn test_timeline_default() {
        let timeline = Timeline::default();

        assert_eq!(timeline.current_time(), 0.0);
        assert_eq!(timeline.duration(), 0.0);
    }

    #[test]
    fn test_timeline_label_creation() {
        let label = TimelineLabel::new("test_label", 1.5);

        assert_eq!(label.name, "test_label");
        assert_eq!(label.position, 1.5);
    }

    #[test]
    fn test_timeline_position_absolute() {
        let pos = TimelinePosition::Absolute(1.5);

        assert_eq!(pos.calculate_position(0.0, &HashMap::new()), Some(1.5));
    }

    #[test]
    fn test_timeline_position_relative() {
        let pos = TimelinePosition::Relative(0.5);

        assert_eq!(pos.calculate_position(2.0, &HashMap::new()), Some(2.5));
    }

    #[test]
    fn test_timeline_position_overlap() {
        let pos = TimelinePosition::Overlap(0.3);

        assert_eq!(pos.calculate_position(2.0, &HashMap::new()), Some(1.7));
    }

    #[test]
    fn test_timeline_position_label() {
        let mut labels = HashMap::new();
        labels.insert("myLabel".to_string(), 3.0);

        let pos = TimelinePosition::Label {
            name: "myLabel".to_string(),
            offset: 0.5,
        };

        assert_eq!(pos.calculate_position(0.0, &labels), Some(3.5));
    }

    #[test]
    fn test_timeline_position_parse_absolute() {
        let pos = TimelinePosition::parse("1.5");

        assert_eq!(pos, Some(TimelinePosition::Absolute(1.5)));
    }

    #[test]
    fn test_timeline_position_parse_relative() {
        let pos = TimelinePosition::parse("+=0.5");

        assert_eq!(pos, Some(TimelinePosition::Relative(0.5)));
    }

    #[test]
    fn test_timeline_position_parse_overlap() {
        let pos = TimelinePosition::parse("-=0.2");

        assert_eq!(pos, Some(TimelinePosition::Overlap(0.2)));
    }

    #[test]
    fn test_timeline_position_parse_label() {
        let pos = TimelinePosition::parse("myLabel+=0.3");

        assert_eq!(
            pos,
            Some(TimelinePosition::Label {
                name: "myLabel".to_string(),
                offset: 0.3
            })
        );
    }

    #[test]
    fn test_timeline_add_label() {
        let timeline = Timeline::new().add_label("test");

        assert_eq!(timeline.labels.len(), 1);
        assert_eq!(timeline.labels[0].name, "test");
        assert_eq!(timeline.labels[0].position, 0.0);
    }

    #[test]
    fn test_timeline_add_label_at() {
        let timeline = Timeline::new().add_label_at("test", 2.5);

        assert_eq!(timeline.labels[0].position, 2.5);
    }

    #[test]
    fn test_timeline_pause_resume() {
        let mut timeline = Timeline::new();

        assert!(!timeline.playing);

        timeline.playing = true;
        timeline.pause();

        assert!(!timeline.playing);

        timeline.resume();

        assert!(timeline.playing);
    }

    #[test]
    fn test_timeline_stop() {
        let mut timeline = Timeline::new();
        timeline.playing = true;
        timeline.current_time = 1.0;

        timeline.stop();

        assert!(!timeline.playing);
        assert_eq!(timeline.current_time, 0.0);
    }

    #[test]
    fn test_timeline_time_scale() {
        let mut timeline = Timeline::new();

        timeline.set_time_scale(2.0);

        assert_eq!(timeline.time_scale(), 2.0);
    }

    #[test]
    fn test_timeline_set_loops() {
        let timeline = Timeline::new().set_loops(3);

        assert_eq!(timeline.loop_count, 3);
    }

    #[test]
    fn test_timeline_progress() {
        let mut timeline = Timeline::new();
        timeline.duration = 10.0;

        assert_eq!(timeline.progress(), 0.0);

        timeline.current_time = 5.0;

        assert_eq!(timeline.progress(), 0.5);
    }

    #[test]
    fn test_timeline_is_complete() {
        let mut timeline = Timeline::new();
        timeline.duration = 5.0;
        timeline.loop_count = 0; // 0 means infinite

        // Should never be complete with infinite loops
        assert!(!timeline.is_complete());

        // Test with no loops (play once)
        let mut timeline2 = Timeline::new();
        timeline2.duration = 5.0;

        assert!(!timeline2.is_complete());

        timeline2.current_time = 5.0;

        assert!(timeline2.is_complete());
    }

    #[test]
    fn test_timeline_update_basic() {
        let mut timeline = Timeline::new();
        timeline.duration = 1.0;
        timeline.playing = true;

        // First update - not complete
        let complete = timeline.update(0.5);
        assert_eq!(timeline.current_time, 0.5);
        assert!(!complete);

        // Second update - reaches exactly duration, should return complete
        let complete = timeline.update(0.5);
        assert!(complete);
    }

    #[test]
    fn test_timeline_position_parse_invalid() {
        let pos = TimelinePosition::parse("");

        assert!(pos.is_none());
    }

    #[test]
    fn test_timeline_with_manager() {
        let manager = Arc::new(Mutex::new(AnimationManager::new()));

        let timeline = Timeline::new().with_manager(manager.clone());

        assert!(Arc::ptr_eq(&timeline.manager, &manager));
    }

    #[test]
    fn test_timeline_handle_pause_resume() {
        let mut timeline = Timeline::new();
        timeline.playing = true;

        let handle = TimelineHandle {
            timeline: Arc::new(Mutex::new(timeline)),
        };

        handle.pause();

        assert!(!handle.is_playing());

        handle.resume();

        assert!(handle.is_playing());
    }

    #[test]
    fn test_timeline_handle_set_time_scale() {
        let mut timeline = Timeline::new();
        timeline.duration = 5.0;
        timeline.current_time = 2.5;

        let handle = TimelineHandle {
            timeline: Arc::new(Mutex::new(timeline)),
        };

        handle.set_time_scale(2.0);

        assert_eq!(handle.progress(), 0.5);
    }
}
