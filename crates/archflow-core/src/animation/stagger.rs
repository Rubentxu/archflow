//! Staggering - Wave-based animation delays
//!
//! Provides GSAP/Anime.js-style staggering for creating wave effects across multiple elements:
//!
//! # Example
//!
//! ```text
//! // Simple linear stagger
//! let stagger = Stagger::new(100.0).from_first();
//!
//! // Grid-based stagger from center
//! let stagger = Stagger::new(50.0)
//!     .grid(4, 4)
//!     .from_center()
//!     .with_easing(EasingFunction::CubicOut);
//!
//! // Apply to timeline
//! for (i, shape) in shapes.iter().enumerate() {
//!     let delay = stagger.calculate_delay(i, &grid_position);
//!     timeline.add(shape.animate().to(100.0, 100.0).duration(500), delay);
//! }
//! ```

use super::EasingFunction;
use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stagger origin - where the wave starts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerFrom {
    /// Wave starts from the first element (index 0)
    First,
    /// Wave starts from the last element
    Last,
    /// Wave starts from the center and spreads outward
    Center,
    /// Wave starts from a specific index
    Index { index: usize },
}

/// Stagger axis for grid-based staggering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerAxis {
    /// No axis - treat as linear list
    None,
    /// Stagger along X axis (rows)
    X,
    /// Stagger along Y axis (columns)
    Y,
    /// Stagger along both axes (diagonal wave)
    Both,
}

/// Grid position for 2D staggering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridPosition {
    /// Row index (0-based)
    pub row: usize,
    /// Column index (0-based)
    pub col: usize,
}

impl GridPosition {
    /// Create a new grid position
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    /// Calculate distance from center (for Center-based staggering)
    pub fn distance_from_center(&self, grid_cols: usize, grid_rows: usize) -> f64 {
        let center_row = (grid_rows - 1) as f64 / 2.0;
        let center_col = (grid_cols - 1) as f64 / 2.0;

        let dr = self.row as f64 - center_row;
        let dc = self.col as f64 - center_col;

        (dr * dr + dc * dc).sqrt()
    }

    /// Calculate Manhattan distance from a position
    pub fn manhattan_distance(&self, other: &GridPosition) -> f64 {
        let dr = (self.row as isize - other.row as isize).abs() as f64;
        let dc = (self.col as isize - other.col as isize).abs() as f64;
        dr + dc
    }
}

/// Stagger configuration for wave-based animation delays
///
/// Provides flexible staggering strategies inspired by GSAP and Anime.js:
/// - Linear staggering (from first/last/center/index)
/// - Grid-based staggering with axis control
/// - Easing support for non-linear delay curves
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stagger {
    /// Delay between each element in milliseconds
    delay_ms: f64,
    /// Base start delay in milliseconds
    start_ms: f64,
    /// Where the wave starts
    from: StaggerFrom,
    /// Grid dimensions (rows, cols)
    grid: Option<(usize, usize)>,
    /// Axis for grid-based staggering
    axis: StaggerAxis,
    /// Optional easing for delay calculation
    easing: Option<EasingFunction>,
    /// Cached grid positions for entities
    grid_positions: HashMap<EntityId, GridPosition>,
}

impl Stagger {
    /// Create a new stagger configuration
    ///
    /// # Arguments
    /// * `delay_ms` - Delay between each element in milliseconds
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0); // 100ms between each element
    /// ```
    pub fn new(delay_ms: f64) -> Self {
        Self {
            delay_ms,
            start_ms: 0.0,
            from: StaggerFrom::First,
            grid: None,
            axis: StaggerAxis::None,
            easing: None,
            grid_positions: HashMap::new(),
        }
    }

    /// Set the starting delay offset
    ///
    /// # Arguments
    /// * `start_ms` - Base start delay in milliseconds
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(500.0); // Start after 500ms
    /// ```
    pub fn with_start(mut self, start_ms: f64) -> Self {
        self.start_ms = start_ms;
        self
    }

    /// Set where the wave starts
    ///
    /// # Arguments
    /// * `from` - Stagger origin
    ///
    /// # Examples
    /// ```
    /// use archflow_core::{Stagger, StaggerFrom};
    ///
    /// let stagger = Stagger::new(100.0).from_last();
    /// ```
    pub fn with_from(mut self, from: StaggerFrom) -> Self {
        self.from = from;
        self
    }

    /// Wave starts from the first element
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0).from_first();
    /// ```
    pub fn from_first(mut self) -> Self {
        self.from = StaggerFrom::First;
        self
    }

    /// Wave starts from the last element
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0).from_last();
    /// ```
    pub fn from_last(mut self) -> Self {
        self.from = StaggerFrom::Last;
        self
    }

    /// Wave starts from the center
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0).from_center();
    /// ```
    pub fn from_center(mut self) -> Self {
        self.from = StaggerFrom::Center;
        self
    }

    /// Wave starts from a specific index
    ///
    /// # Arguments
    /// * `index` - The index to start from
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0).from_index(5);
    /// ```
    pub fn from_index(mut self, index: usize) -> Self {
        self.from = StaggerFrom::Index { index };
        self
    }

    /// Configure grid-based staggering
    ///
    /// # Arguments
    /// * `rows` - Number of rows in the grid
    /// * `cols` - Number of columns in the grid
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(50.0).grid(4, 4);
    /// ```
    pub fn grid(mut self, rows: usize, cols: usize) -> Self {
        self.grid = Some((rows, cols));
        self
    }

    /// Set the axis for grid-based staggering
    ///
    /// # Arguments
    /// * `axis` - Stagger axis
    ///
    /// # Examples
    /// ```
    /// use archflow_core::{Stagger, StaggerAxis};
    ///
    /// let stagger = Stagger::new(50.0).grid(4, 4).with_axis(StaggerAxis::X);
    /// ```
    pub fn with_axis(mut self, axis: StaggerAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Apply easing to the delay calculation
    ///
    /// # Arguments
    /// * `easing` - Easing function
    ///
    /// # Examples
    /// ```
    /// use archflow_core::{Stagger, EasingFunction};
    ///
    /// let stagger = Stagger::new(100.0).with_easing(EasingFunction::CubicOut);
    /// ```
    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = Some(easing);
        self
    }

    /// Register a grid position for an entity
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier
    /// * `position` - Grid position
    ///
    /// # Examples
    /// ```
    /// use archflow_core::{Stagger, GridPosition};
    /// use archflow_core::EntityId;
    ///
    /// let mut stagger = Stagger::new(50.0).grid(4, 4);
    /// let id = EntityId::from_u128(1);
    /// stagger.set_grid_position(id, GridPosition::new(1, 2));
    /// ```
    pub fn set_grid_position(&mut self, entity_id: EntityId, position: GridPosition) {
        self.grid_positions.insert(entity_id, position);
    }

    /// Calculate the delay for a specific index
    ///
    /// # Arguments
    /// * `index` - Element index
    /// * `total` - Total number of elements
    ///
    /// # Returns
    /// Delay in milliseconds
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0).from_first();
    /// let delay = stagger.calculate_delay_for_index(0, 10); // 0ms
    /// let delay = stagger.calculate_delay_for_index(5, 10); // 500ms
    /// ```
    pub fn calculate_delay_for_index(&self, index: usize, total: usize) -> f64 {
        let raw_delay = match self.from {
            StaggerFrom::First => index as f64 * self.delay_ms,
            StaggerFrom::Last => (total - 1 - index) as f64 * self.delay_ms,
            StaggerFrom::Center => {
                let center = (total - 1) as f64 / 2.0;
                let distance = (index as f64 - center).abs();
                distance * self.delay_ms
            }
            StaggerFrom::Index { index: start_index } => {
                let distance = (index as isize - start_index as isize).abs() as f64;
                distance * self.delay_ms
            }
        };

        // Apply easing if specified
        let eased_delay = if let Some(easing) = self.easing {
            let max_delay = (total - 1) as f64 * self.delay_ms;
            if max_delay > 0.0 {
                let t = ((raw_delay / max_delay).min(1.0)) as f32;
                self.start_ms + (easing.apply(t) as f64) * max_delay
            } else {
                self.start_ms
            }
        } else {
            self.start_ms + raw_delay
        };

        eased_delay
    }

    /// Calculate the delay for an entity with grid position
    ///
    /// # Arguments
    /// * `entity_id` - Entity identifier
    /// * `grid_position` - Optional grid position (overrides cached)
    ///
    /// # Returns
    /// Delay in milliseconds, or None if no grid position available
    ///
    /// # Examples
    /// ```
    /// use archflow_core::{Stagger, GridPosition};
    /// use archflow_core::EntityId;
    ///
    /// let stagger = Stagger::new(50.0).grid(4, 4).from_center();
    /// let id = EntityId::from_u128(1);
    /// let pos = GridPosition::new(2, 2);
    /// let delay = stagger.calculate_delay(id, Some(pos));
    /// ```
    pub fn calculate_delay(
        &self,
        entity_id: EntityId,
        grid_position: Option<GridPosition>,
    ) -> Option<f64> {
        let pos = grid_position.or_else(|| self.grid_positions.get(&entity_id).copied())?;

        let (rows, cols) = self.grid?;

        let raw_delay = match (self.from, self.axis) {
            (StaggerFrom::Center, _) => {
                // Distance from center
                pos.distance_from_center(cols, rows) * self.delay_ms
            }
            (StaggerFrom::First, StaggerAxis::X) => {
                // Stagger by rows
                pos.row as f64 * self.delay_ms
            }
            (StaggerFrom::First, StaggerAxis::Y) => {
                // Stagger by columns
                pos.col as f64 * self.delay_ms
            }
            (StaggerFrom::First, StaggerAxis::Both) => {
                // Diagonal wave
                (pos.row + pos.col) as f64 * self.delay_ms
            }
            (StaggerFrom::Last, StaggerAxis::X) => {
                // Reverse rows
                (rows - 1 - pos.row) as f64 * self.delay_ms
            }
            (StaggerFrom::Last, StaggerAxis::Y) => {
                // Reverse columns
                (cols - 1 - pos.col) as f64 * self.delay_ms
            }
            (StaggerFrom::Last, StaggerAxis::Both) => {
                // Reverse diagonal
                ((rows - 1 - pos.row) + (cols - 1 - pos.col)) as f64 * self.delay_ms
            }
            (StaggerFrom::Index { index }, _) => {
                // Calculate linear index and apply index-based stagger
                let linear_index = pos.row * cols + pos.col;
                let distance = (linear_index as isize - index as isize).abs() as f64;
                distance * self.delay_ms
            }
            _ => {
                // Default: linear by row-major order
                let linear_index = pos.row * cols + pos.col;
                linear_index as f64 * self.delay_ms
            }
        };

        // Apply easing if specified
        let total_cells = rows * cols;
        let eased_delay = if let Some(easing) = self.easing {
            let max_delay = (total_cells - 1) as f64 * self.delay_ms;
            if max_delay > 0.0 {
                let t = ((raw_delay / max_delay).min(1.0)) as f32;
                self.start_ms + (easing.apply(t) as f64) * max_delay
            } else {
                self.start_ms
            }
        } else {
            self.start_ms + raw_delay
        };

        Some(eased_delay)
    }

    /// Calculate delays for a range of indices
    ///
    /// # Arguments
    /// * `total` - Total number of elements
    ///
    /// # Returns
    /// Vector of delays in milliseconds
    ///
    /// # Examples
    /// ```
    /// use archflow_core::Stagger;
    ///
    /// let stagger = Stagger::new(100.0);
    /// let delays = stagger.calculate_delays(5); // [0.0, 100.0, 200.0, 300.0, 400.0]
    /// ```
    pub fn calculate_delays(&self, total: usize) -> Vec<f64> {
        (0..total)
            .map(|i| self.calculate_delay_for_index(i, total))
            .collect()
    }

    /// Get the base delay value
    pub fn delay(&self) -> f64 {
        self.delay_ms
    }

    /// Get the start delay
    pub fn start(&self) -> f64 {
        self.start_ms
    }

    /// Check if grid-based staggering is configured
    pub fn is_grid(&self) -> bool {
        self.grid.is_some()
    }

    /// Get grid dimensions if configured
    pub fn grid_size(&self) -> Option<(usize, usize)> {
        self.grid
    }
}

impl Default for Stagger {
    fn default() -> Self {
        Self::new(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Stagger Creation Tests ===

    #[test]
    fn test_stagger_default() {
        let stagger = Stagger::default();
        assert_eq!(stagger.delay(), 100.0);
        assert_eq!(stagger.start(), 0.0);
    }

    #[test]
    fn test_stagger_new() {
        let stagger = Stagger::new(50.0);
        assert_eq!(stagger.delay(), 50.0);
        assert_eq!(stagger.start(), 0.0);
    }

    #[test]
    fn test_stagger_with_start() {
        let stagger = Stagger::new(100.0).with_start(500.0);
        assert_eq!(stagger.start(), 500.0);
    }

    #[test]
    fn test_stagger_from_first() {
        let stagger = Stagger::new(100.0).from_first();
        let delays = stagger.calculate_delays(5);
        assert_eq!(delays, vec![0.0, 100.0, 200.0, 300.0, 400.0]);
    }

    #[test]
    fn test_stagger_from_last() {
        let stagger = Stagger::new(100.0).from_last();
        let delays = stagger.calculate_delays(5);
        assert_eq!(delays, vec![400.0, 300.0, 200.0, 100.0, 0.0]);
    }

    #[test]
    fn test_stagger_from_center() {
        let stagger = Stagger::new(100.0).from_center();
        let delays = stagger.calculate_delays(5);
        // Center is at index 2
        // Index 0: distance 2 → 200ms
        // Index 1: distance 1 → 100ms
        // Index 2: distance 0 → 0ms
        // Index 3: distance 1 → 100ms
        // Index 4: distance 2 → 200ms
        assert_eq!(delays, vec![200.0, 100.0, 0.0, 100.0, 200.0]);
    }

    #[test]
    fn test_stagger_from_index() {
        let stagger = Stagger::new(50.0).from_index(3);
        let delays = stagger.calculate_delays(6);
        // Index 3 is the center
        // 0: distance 3 → 150ms
        // 1: distance 2 → 100ms
        // 2: distance 1 → 50ms
        // 3: distance 0 → 0ms
        // 4: distance 1 → 50ms
        // 5: distance 2 → 100ms
        assert_eq!(delays, vec![150.0, 100.0, 50.0, 0.0, 50.0, 100.0]);
    }

    // === Easing Tests ===

    #[test]
    fn test_stagger_with_easing() {
        let stagger = Stagger::new(100.0)
            .from_first()
            .with_easing(EasingFunction::CubicOut);
        let delays = stagger.calculate_delays(5);

        // With easing, delays should be non-linear
        // First delays should be relatively fast, then slow down
        assert!(delays[0] < delays[1]);
        assert!(delays[1] < delays[2]);
        assert!(delays[2] < delays[3]);
        assert!(delays[3] < delays[4]);

        // Total should be close to max delay
        assert!(delays[4] > 350.0); // easing compressed curve
    }

    // === Grid Tests ===

    #[test]
    fn test_grid_position_new() {
        let pos = GridPosition::new(2, 3);
        assert_eq!(pos.row, 2);
        assert_eq!(pos.col, 3);
    }

    #[test]
    fn test_grid_position_distance_from_center() {
        let pos = GridPosition::new(0, 0);
        let distance = pos.distance_from_center(4, 4);
        // Center is at (1.5, 1.5)
        // Distance = sqrt((0-1.5)^2 + (0-1.5)^2) = sqrt(4.5) ≈ 2.12
        assert!((distance - 2.121).abs() < 0.01);
    }

    #[test]
    fn test_grid_position_manhattan_distance() {
        let pos1 = GridPosition::new(0, 0);
        let pos2 = GridPosition::new(2, 3);
        let distance = pos1.manhattan_distance(&pos2);
        assert_eq!(distance, 5.0); // |0-2| + |0-3| = 5
    }

    #[test]
    fn test_stagger_grid_from_first_x() {
        let stagger = Stagger::new(50.0).grid(3, 3).with_axis(StaggerAxis::X);

        // Test first column
        let pos1 = GridPosition::new(0, 0);
        let pos2 = GridPosition::new(1, 0);
        let pos3 = GridPosition::new(2, 0);

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id3 = EntityId::from_u128(3);

        let delay1 = stagger.calculate_delay(id1, Some(pos1));
        let delay2 = stagger.calculate_delay(id2, Some(pos2));
        let delay3 = stagger.calculate_delay(id3, Some(pos3));

        assert_eq!(delay1, Some(0.0));
        assert_eq!(delay2, Some(50.0));
        assert_eq!(delay3, Some(100.0));
    }

    #[test]
    fn test_stagger_grid_from_first_y() {
        let stagger = Stagger::new(50.0).grid(3, 3).with_axis(StaggerAxis::Y);

        // Test first row
        let pos1 = GridPosition::new(0, 0);
        let pos2 = GridPosition::new(0, 1);
        let pos3 = GridPosition::new(0, 2);

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id3 = EntityId::from_u128(3);

        let delay1 = stagger.calculate_delay(id1, Some(pos1));
        let delay2 = stagger.calculate_delay(id2, Some(pos2));
        let delay3 = stagger.calculate_delay(id3, Some(pos3));

        assert_eq!(delay1, Some(0.0));
        assert_eq!(delay2, Some(50.0));
        assert_eq!(delay3, Some(100.0));
    }

    #[test]
    fn test_stagger_grid_from_center() {
        let stagger = Stagger::new(50.0).grid(3, 3).from_center();

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id_center = EntityId::from_u128(3);

        let pos1 = GridPosition::new(0, 0); // Corner - farthest
        let pos2 = GridPosition::new(0, 1); // Edge - medium
        let pos_center = GridPosition::new(1, 1); // Center - closest

        let delay1 = stagger.calculate_delay(id1, Some(pos1));
        let delay2 = stagger.calculate_delay(id2, Some(pos2));
        let delay_center = stagger.calculate_delay(id_center, Some(pos_center));

        assert!(delay1.unwrap() > delay2.unwrap());
        assert!(delay2.unwrap() > delay_center.unwrap());
        assert_eq!(delay_center, Some(0.0));
    }

    #[test]
    fn test_stagger_grid_diagonal() {
        let stagger = Stagger::new(50.0).grid(3, 3).with_axis(StaggerAxis::Both);

        let pos1 = GridPosition::new(0, 0);
        let pos2 = GridPosition::new(1, 1);
        let pos3 = GridPosition::new(2, 2);

        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id3 = EntityId::from_u128(3);

        let delay1 = stagger.calculate_delay(id1, Some(pos1));
        let delay2 = stagger.calculate_delay(id2, Some(pos2));
        let delay3 = stagger.calculate_delay(id3, Some(pos3));

        // Diagonal wave: (row + col) * delay
        assert_eq!(delay1, Some(0.0)); // 0+0 = 0
        assert_eq!(delay2, Some(100.0)); // 1+1 = 2
        assert_eq!(delay3, Some(200.0)); // 2+2 = 4
    }

    // === Grid Position Cache Tests ===

    #[test]
    fn test_stagger_set_grid_position() {
        let mut stagger = Stagger::new(50.0).grid(3, 3);
        let id = EntityId::from_u128(1);
        let pos = GridPosition::new(1, 2);

        stagger.set_grid_position(id, pos);

        let delay = stagger.calculate_delay(id, None);
        // row=1, col=2, cols=3, linear_index = 1*3 + 2 = 5
        // delay = 5 * 50 = 250
        assert_eq!(delay, Some(250.0));
    }

    #[test]
    fn test_stagger_set_grid_position_correct() {
        let mut stagger = Stagger::new(50.0).grid(3, 3);
        let id = EntityId::from_u128(1);
        let pos = GridPosition::new(1, 1); // Center of 3x3

        stagger.set_grid_position(id, pos);

        let delay = stagger.calculate_delay(id, None);
        // row=1, col=1, cols=3, linear_index = 1*3 + 1 = 4
        assert_eq!(delay, Some(200.0)); // 4 * 50 = 200
    }

    // === Property Access Tests ===

    #[test]
    fn test_stagger_is_grid() {
        let stagger1 = Stagger::new(100.0);
        assert!(!stagger1.is_grid());

        let stagger2 = Stagger::new(100.0).grid(3, 3);
        assert!(stagger2.is_grid());
    }

    #[test]
    fn test_stagger_grid_size() {
        let stagger = Stagger::new(100.0).grid(4, 5);
        assert_eq!(stagger.grid_size(), Some((4, 5)));
    }

    // === Edge Cases ===

    #[test]
    fn test_stagger_single_element() {
        let stagger = Stagger::new(100.0).from_first();
        let delays = stagger.calculate_delays(1);
        assert_eq!(delays, vec![0.0]);
    }

    #[test]
    fn test_stagger_with_start_offset() {
        let stagger = Stagger::new(100.0).with_start(500.0);
        let delays = stagger.calculate_delays(3);
        assert_eq!(delays, vec![500.0, 600.0, 700.0]);
    }

    #[test]
    fn test_stagger_even_count_from_center() {
        let stagger = Stagger::new(100.0).from_center();
        let delays = stagger.calculate_delays(4);
        // Center is at index 1.5
        // Index 0: distance 1.5 → 150ms
        // Index 1: distance 0.5 → 50ms
        // Index 2: distance 0.5 → 50ms
        // Index 3: distance 1.5 → 150ms
        assert_eq!(delays, vec![150.0, 50.0, 50.0, 150.0]);
    }
}
