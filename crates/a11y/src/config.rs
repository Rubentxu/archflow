//! Accessibility configuration and verbosity settings

use serde::{Deserialize, Serialize};

/// Accessibility configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct A11yConfig {
    /// Enable ARIA attributes
    pub enable_aria: bool,
    /// Enable keyboard navigation
    pub enable_keyboard: bool,
    /// Enable screen reader support
    pub enable_screen_reader: bool,
    /// Enable focus indicators
    pub enable_focus_indicators: bool,
    /// Enable high contrast mode
    pub high_contrast_mode: bool,
    /// Focus indicator color
    pub focus_indicator_color: String,
    /// Focus indicator width
    pub focus_indicator_width: f32,
    /// Minimum touch target size
    pub min_touch_target_size: f32,
    /// Enable reduced motion
    pub reduced_motion: bool,
    /// Screen reader verbosity
    pub verbosity: A11yVerbosity,
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            enable_aria: true,
            enable_keyboard: true,
            enable_screen_reader: true,
            enable_focus_indicators: true,
            high_contrast_mode: false,
            focus_indicator_color: "#0066cc".to_string(),
            focus_indicator_width: 2.0,
            min_touch_target_size: 44.0,
            reduced_motion: false,
            verbosity: A11yVerbosity::Normal,
        }
    }
}

/// Verbosity level for screen reader announcements
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum A11yVerbosity {
    /// Minimal announcements
    Minimal,
    /// Normal announcements
    Normal,
    /// Verbose announcements
    Verbose,
}

/// Type of live region
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveRegionType {
    /// Polite live region (announced when idle)
    Polite,
    /// Assertive live region (announced immediately)
    Assertive,
}
