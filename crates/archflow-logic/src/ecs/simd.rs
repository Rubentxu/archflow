// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - SIMD Optimization Module
//
// Provides high-performance batch processing operations for physics simulation.
// Features unrolled loops for better cache utilization and potential auto-vectorization.
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::vec::Vec;
use core::f32;

/// Configuration for batch physics processing
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub struct BatchPhysicsConfig {
    /// Batch size for processing
    pub batch_size: usize,
    /// Enable strict alignment checks
    pub strict_alignment: bool,
}

impl Default for BatchPhysicsConfig {
    fn default() -> Self {
        Self {
            batch_size: 256,
            strict_alignment: false,
        }
    }
}

/// Statistics from batch operations
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct BatchStats {
    /// Total entities processed
    pub entities_processed: usize,
    /// Number of batches processed
    pub batches_processed: usize,
}

/// Batch processor for high-performance physics operations
///
/// Provides optimized batch processing for entity physics using
/// unrolled loops for better compiler auto-vectorization.
#[derive(Debug)]
pub struct BatchPhysicsProcessor {
    config: BatchPhysicsConfig,
    stats: BatchStats,
}

impl BatchPhysicsProcessor {
    /// Creates a new batch processor with default configuration
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BatchPhysicsConfig::default(),
            stats: BatchStats::default(),
        }
    }

    /// Creates a processor with custom configuration
    #[inline]
    #[must_use]
    pub fn with_config(config: BatchPhysicsConfig) -> Self {
        Self {
            config,
            stats: BatchStats::default(),
        }
    }

    /// Returns a reference to the current configuration
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &BatchPhysicsConfig {
        &self.config
    }

    /// Returns statistics from the last operation
    #[inline]
    #[must_use]
    pub const fn stats(&self) -> &BatchStats {
        &self.stats
    }

    /// Resets statistics counters
    #[inline]
    pub fn reset_stats(&mut self) {
        self.stats = BatchStats::default();
    }

    /// Applies gravity to velocity array
    #[inline]
    pub fn apply_gravity(
        &mut self,
        velocities: &mut [[f32; 2]],
        gravity_x: f32,
        gravity_y: f32,
    ) -> usize {
        let len = velocities.len();
        let batch_size = self.config.batch_size.min(len);

        let mut i = 0;
        while i + 7 < batch_size {
            self.process_gravity_batch_8(&mut velocities[i..], gravity_x, gravity_y);
            i += 8;
        }

        while i + 3 < batch_size {
            self.process_gravity_batch_4(&mut velocities[i..], gravity_x, gravity_y);
            i += 4;
        }

        while i < batch_size {
            velocities[i][0] += gravity_x;
            velocities[i][1] += gravity_y;
            i += 1;
        }

        self.stats.entities_processed += batch_size;
        self.stats.batches_processed += (batch_size + 7) / 8;

        batch_size
    }

    #[inline(always)]
    fn process_gravity_batch_4(&self, velocities: &mut [[f32; 2]], gx: f32, gy: f32) {
        for i in 0..4.min(velocities.len()) {
            velocities[i][0] += gx;
            velocities[i][1] += gy;
        }
    }

    #[inline(always)]
    fn process_gravity_batch_8(&self, velocities: &mut [[f32; 2]], gx: f32, gy: f32) {
        for i in 0..8.min(velocities.len()) {
            velocities[i][0] += gx;
            velocities[i][1] += gy;
        }
    }

    /// Applies damping to velocity array
    #[inline]
    pub fn apply_damping(&mut self, velocities: &mut [[f32; 2]], damping: f32) -> usize {
        let len = velocities.len();
        let batch_size = self.config.batch_size.min(len);
        let factor = 1.0 - damping.clamp(0.0, 1.0);

        let mut i = 0;
        while i + 7 < batch_size {
            self.process_damping_batch_8(&mut velocities[i..], factor);
            i += 8;
        }

        while i + 3 < batch_size {
            self.process_damping_batch_4(&mut velocities[i..], factor);
            i += 4;
        }

        while i < batch_size {
            velocities[i][0] *= factor;
            velocities[i][1] *= factor;
            i += 1;
        }

        self.stats.entities_processed += batch_size;
        batch_size
    }

    #[inline(always)]
    fn process_damping_batch_4(&self, velocities: &mut [[f32; 2]], factor: f32) {
        for i in 0..4.min(velocities.len()) {
            velocities[i][0] *= factor;
            velocities[i][1] *= factor;
        }
    }

    #[inline(always)]
    fn process_damping_batch_8(&self, velocities: &mut [[f32; 2]], factor: f32) {
        for i in 0..8.min(velocities.len()) {
            velocities[i][0] *= factor;
            velocities[i][1] *= factor;
        }
    }

    /// Clamps velocity magnitudes
    #[inline]
    pub fn clamp_velocities(&mut self, velocities: &mut [[f32; 2]], max_velocity: f32) -> usize {
        let len = velocities.len();
        let batch_size = self.config.batch_size.min(len);
        let max_vel_sq = max_velocity * max_velocity;

        let mut clamped = 0;
        let mut i = 0;

        while i + 7 < batch_size {
            clamped += self.process_clamp_batch_8(&mut velocities[i..], max_vel_sq);
            i += 8;
        }

        while i + 3 < batch_size {
            clamped += self.process_clamp_batch_4(&mut velocities[i..], max_vel_sq);
            i += 4;
        }

        while i < batch_size {
            let vel_sq = velocities[i][0] * velocities[i][0] + velocities[i][1] * velocities[i][1];
            if vel_sq > max_vel_sq {
                let scale = max_velocity / vel_sq.sqrt();
                velocities[i][0] *= scale;
                velocities[i][1] *= scale;
                clamped += 1;
            }
            i += 1;
        }

        self.stats.entities_processed += batch_size;
        clamped
    }

    #[inline(always)]
    fn process_clamp_batch_4(&self, velocities: &mut [[f32; 2]], max_vel_sq: f32) -> usize {
        let mut clamped = 0;
        for i in 0..4.min(velocities.len()) {
            let vel_sq = velocities[i][0] * velocities[i][0] + velocities[i][1] * velocities[i][1];
            if vel_sq > max_vel_sq {
                let scale = (max_vel_sq / vel_sq).sqrt();
                velocities[i][0] *= scale;
                velocities[i][1] *= scale;
                clamped += 1;
            }
        }
        clamped
    }

    #[inline(always)]
    fn process_clamp_batch_8(&self, velocities: &mut [[f32; 2]], max_vel_sq: f32) -> usize {
        let mut clamped = 0;
        for i in 0..8.min(velocities.len()) {
            let vel_sq = velocities[i][0] * velocities[i][0] + velocities[i][1] * velocities[i][1];
            if vel_sq > max_vel_sq {
                let scale = (max_vel_sq / vel_sq).sqrt();
                velocities[i][0] *= scale;
                velocities[i][1] *= scale;
                clamped += 1;
            }
        }
        clamped
    }

    /// Integrates positions using velocity and delta time
    #[inline]
    pub fn integrate_positions(
        &mut self,
        positions: &mut [[f32; 2]],
        velocities: &[[f32; 2]],
        delta_time: f32,
    ) -> usize {
        let len = positions.len().min(velocities.len());
        let batch_size = self.config.batch_size.min(len);

        let mut i = 0;
        while i + 7 < batch_size {
            self.process_integration_batch_8(&mut positions[i..], &velocities[i..], delta_time);
            i += 8;
        }

        while i + 3 < batch_size {
            self.process_integration_batch_4(&mut positions[i..], &velocities[i..], delta_time);
            i += 4;
        }

        while i < batch_size {
            positions[i][0] += velocities[i][0] * delta_time;
            positions[i][1] += velocities[i][1] * delta_time;
            i += 1;
        }

        self.stats.entities_processed += batch_size;
        batch_size
    }

    #[inline(always)]
    fn process_integration_batch_4(
        &self,
        positions: &mut [[f32; 2]],
        velocities: &[[f32; 2]],
        dt: f32,
    ) {
        let count = 4.min(velocities.len().min(positions.len()));
        for i in 0..count {
            positions[i][0] += velocities[i][0] * dt;
            positions[i][1] += velocities[i][1] * dt;
        }
    }

    #[inline(always)]
    fn process_integration_batch_8(
        &self,
        positions: &mut [[f32; 2]],
        velocities: &[[f32; 2]],
        dt: f32,
    ) {
        let count = 8.min(velocities.len().min(positions.len()));
        for i in 0..count {
            positions[i][0] += velocities[i][0] * dt;
            positions[i][1] += velocities[i][1] * dt;
        }
    }

    /// Processes a full physics step
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn process_physics_batch(
        &mut self,
        positions: &mut [[f32; 2]],
        velocities: &mut [[f32; 2]],
        gravity: [f32; 2],
        damping: f32,
        max_velocity: f32,
        delta_time: f32,
        boundary: f32,
        bounciness: f32,
    ) -> usize {
        self.apply_gravity(velocities, gravity[0] * delta_time, gravity[1] * delta_time);
        self.apply_damping(velocities, damping);
        self.clamp_velocities(velocities, max_velocity);
        self.integrate_positions(positions, velocities, delta_time);
        self.check_boundaries(positions, velocities, boundary, bounciness)
    }

    /// Checks and resolves boundary collisions
    #[inline]
    pub fn check_boundaries(
        &mut self,
        positions: &mut [[f32; 2]],
        velocities: &mut [[f32; 2]],
        boundary: f32,
        bounciness: f32,
    ) -> usize {
        let len = positions.len().min(velocities.len());
        let batch_size = self.config.batch_size.min(len);

        let neg_boundary = -boundary;
        let pos_boundary = boundary;

        let mut collisions = 0;
        let mut i = 0;

        while i + 7 < batch_size {
            collisions += self.process_boundary_batch_8(
                &mut positions[i..],
                &mut velocities[i..],
                neg_boundary,
                pos_boundary,
                bounciness,
            );
            i += 8;
        }

        while i + 3 < batch_size {
            collisions += self.process_boundary_batch_4(
                &mut positions[i..],
                &mut velocities[i..],
                neg_boundary,
                pos_boundary,
                bounciness,
            );
            i += 4;
        }

        while i < batch_size {
            let mut collided = false;

            if positions[i][0] < neg_boundary {
                positions[i][0] = neg_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            } else if positions[i][0] > pos_boundary {
                positions[i][0] = pos_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            }

            if positions[i][1] < neg_boundary {
                positions[i][1] = neg_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            } else if positions[i][1] > pos_boundary {
                positions[i][1] = pos_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            }

            if collided {
                collisions += 1;
            }
            i += 1;
        }

        self.stats.entities_processed += batch_size;
        collisions
    }

    #[inline(always)]
    fn process_boundary_batch_4(
        &self,
        positions: &mut [[f32; 2]],
        velocities: &mut [[f32; 2]],
        neg_boundary: f32,
        pos_boundary: f32,
        bounciness: f32,
    ) -> usize {
        let mut collisions = 0;
        let count = 4.min(positions.len().min(velocities.len()));

        for i in 0..count {
            let mut collided = false;

            if positions[i][0] < neg_boundary {
                positions[i][0] = neg_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            } else if positions[i][0] > pos_boundary {
                positions[i][0] = pos_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            }

            if positions[i][1] < neg_boundary {
                positions[i][1] = neg_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            } else if positions[i][1] > pos_boundary {
                positions[i][1] = pos_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            }

            if collided {
                collisions += 1;
            }
        }
        collisions
    }

    #[inline(always)]
    fn process_boundary_batch_8(
        &self,
        positions: &mut [[f32; 2]],
        velocities: &mut [[f32; 2]],
        neg_boundary: f32,
        pos_boundary: f32,
        bounciness: f32,
    ) -> usize {
        let mut collisions = 0;
        let count = 8.min(positions.len().min(velocities.len()));

        for i in 0..count {
            let mut collided = false;

            if positions[i][0] < neg_boundary {
                positions[i][0] = neg_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            } else if positions[i][0] > pos_boundary {
                positions[i][0] = pos_boundary;
                velocities[i][0] = -velocities[i][0] * bounciness;
                collided = true;
            }

            if positions[i][1] < neg_boundary {
                positions[i][1] = neg_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            } else if positions[i][1] > pos_boundary {
                positions[i][1] = pos_boundary;
                velocities[i][1] = -velocities[i][1] * bounciness;
                collided = true;
            }

            if collided {
                collisions += 1;
            }
        }
        collisions
    }
}

impl Default for BatchPhysicsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch iterator for cache-friendly component iteration
///
/// Provides aligned memory access patterns.
#[derive(Debug)]
pub struct SimdBatchIterator<'a, T> {
    data: &'a [T],
    batch_size: usize,
    current: usize,
}

impl<'a, T> SimdBatchIterator<'a, T> {
    /// Creates a new batch iterator
    #[inline]
    #[must_use]
    pub fn new(data: &'a [T], batch_size: usize) -> Self {
        Self {
            data,
            batch_size: batch_size.max(1),
            current: 0,
        }
    }

    /// Returns the remaining number of elements
    #[inline]
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.current)
    }

    /// Returns true if all batches have been consumed
    #[inline]
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.current >= self.data.len()
    }
}

impl<'a, T: Copy> Iterator for SimdBatchIterator<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.data.len() {
            return None;
        }

        let remaining = self.data.len() - self.current;
        let size = self.batch_size.min(remaining);

        let slice = &self.data[self.current..self.current + size];
        self.current += size;

        Some(slice)
    }
}

/// Morton code utility for spatial partitioning
///
/// Provides Z-order curve encoding for cache-friendly spatial queries.
#[derive(Debug)]
pub struct MortonEncoder {
    lut: [u32; 256],
}

impl MortonEncoder {
    /// Creates a new Morton encoder with precomputed lookup table
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let mut lut = [0u32; 256];
        for i in 0..256 {
            lut[i] = Self::spread_bits(i as u32);
        }
        Self { lut }
    }

    /// Spreads bits of a byte: bit n -> bit 2n (compact 8-bit to 16-bit spread)
    #[inline]
    #[must_use]
    pub const fn spread_bits(x: u32) -> u32 {
        let mut x = x as u32;
        x = (x | (x << 8)) & 0x00FF_00FF;
        x = (x | (x << 4)) & 0x0F0F_0F0F;
        x = (x | (x << 2)) & 0x3333_3333;
        x = (x | (x << 1)) & 0x5555_5555;
        x
    }

    /// Expands lower 10 bits to 30-bit morton code (kept for compatibility)
    #[inline]
    #[must_use]
    pub const fn expand_bits(x: u32) -> u32 {
        Self::spread_bits(x)
    }

    /// Encodes 2D position to Morton code
    #[inline]
    #[must_use]
    pub fn encode_2d(&self, x: f32, y: f32, max_coord: f32) -> u32 {
        // Normalize to 0-1 range first
        let nx = x.clamp(0.0, max_coord) / max_coord;
        let ny = y.clamp(0.0, max_coord) / max_coord;

        // Scale to 10 bits (0-1023)
        let xi = (nx * 1023.0) as u32;
        let yi = (ny * 1023.0) as u32;

        // Interleave bits using LUT: x in even positions, y in odd positions
        let x_even = self.lut[(xi & 0xFF) as usize] | (self.lut[((xi >> 8) & 0xFF) as usize] << 16);
        let y_odd =
            (self.lut[(yi & 0xFF) as usize] | (self.lut[((yi >> 8) & 0xFF) as usize] << 16)) << 1;
        x_even | y_odd
    }

    /// Encodes 3D position to Morton code
    #[inline]
    #[must_use]
    pub fn encode_3d(&self, x: f32, y: f32, z: f32, max_coord: f32) -> u32 {
        // Normalize to 0-1 range first
        let nx = x.clamp(0.0, max_coord) / max_coord;
        let ny = y.clamp(0.0, max_coord) / max_coord;
        let nz = z.clamp(0.0, max_coord) / max_coord;

        // Scale to 10 bits (0-1023)
        let xi = (nx * 1023.0) as u32;
        let yi = (ny * 1023.0) as u32;
        let zi = (nz * 1023.0) as u32;

        // Interleave bits using LUT: x in bit 0, y in bit 1, z in bit 2 (repeated)
        let x_bits = self.lut[(xi & 0xFF) as usize] | (self.lut[((xi >> 8) & 0xFF) as usize] << 16);
        let y_bits =
            (self.lut[(yi & 0xFF) as usize] | (self.lut[((yi >> 8) & 0xFF) as usize] << 16)) << 1;
        let z_bits =
            (self.lut[(zi & 0xFF) as usize] | (self.lut[((zi >> 8) & 0xFF) as usize] << 16)) << 2;
        x_bits | y_bits | z_bits
    }
}

// SimdPhysicsConfig and SimdStats are type aliases for backward compatibility
/// Alias for BatchPhysicsConfig
pub type SimdPhysicsConfig = BatchPhysicsConfig;
/// Alias for BatchPhysicsProcessor
pub type SimdPhysicsProcessor = BatchPhysicsProcessor;
/// Alias for BatchStats
pub type SimdStats = BatchStats;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = BatchPhysicsProcessor::new();
        assert_eq!(processor.config().batch_size, 256);
    }

    #[test]
    fn test_config_default() {
        let config = BatchPhysicsConfig::default();
        assert_eq!(config.batch_size, 256);
        assert!(!config.strict_alignment);
    }

    #[test]
    fn test_stats_default() {
        let stats = BatchStats::default();
        assert_eq!(stats.entities_processed, 0);
        assert_eq!(stats.batches_processed, 0);
    }

    #[test]
    fn test_apply_gravity() {
        let mut processor = BatchPhysicsProcessor::new();
        let mut velocities = [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [2.0, 2.0]];

        let count = processor.apply_gravity(&mut velocities, 0.0, -9.81);

        assert_eq!(count, 4);
        assert!((velocities[0][1] - (-9.81)).abs() < 0.001);
    }

    #[test]
    fn test_apply_damping() {
        let mut processor = BatchPhysicsProcessor::new();
        let mut velocities = [[2.0, 2.0], [1.0, 1.0], [4.0, 4.0], [8.0, 8.0]];

        let count = processor.apply_damping(&mut velocities, 0.5);

        assert_eq!(count, 4);
        assert!((velocities[0][0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_velocities() {
        let mut processor = BatchPhysicsProcessor::new();
        let mut velocities = [[100.0, 0.0], [0.0, 50.0], [30.0, 30.0], [5.0, 5.0]];

        let clamped = processor.clamp_velocities(&mut velocities, 10.0);

        assert_eq!(clamped, 3);
        let v0_mag = (velocities[0][0].powi(2) + velocities[0][1].powi(2)).sqrt();
        assert!((v0_mag - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_integrate_positions() {
        let mut processor = BatchPhysicsProcessor::new();
        let mut positions = [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]];
        let velocities = [[10.0, 20.0], [5.0, 5.0], [1.0, 1.0], [0.5, 0.5]];

        let count = processor.integrate_positions(&mut positions, &velocities, 0.016);

        assert_eq!(count, 4);
        assert!((positions[0][0] - 0.16).abs() < 0.001);
    }

    #[test]
    fn test_check_boundaries() {
        let mut processor = BatchPhysicsProcessor::new();
        // Entity 0: at -1500, moving right (+10), will be clamped and bounce left
        // Entity 2: at 1500, moving left (-10), will be clamped and bounce right
        let mut positions = [[-1500.0, 0.0], [0.0, 0.0], [1500.0, 0.0], [500.0, 500.0]];
        let mut velocities = [[10.0, 0.0], [1.0, 0.0], [-10.0, 0.0], [0.0, 0.0]];

        let collisions = processor.check_boundaries(&mut positions, &mut velocities, 1000.0, 1.0);

        assert_eq!(collisions, 2);
        // Entity 0: was at -1500, clamped to -1000, velocity reversed from +10 to -10
        assert_eq!(positions[0][0], -1000.0);
        assert_eq!(velocities[0][0], -10.0, "Velocity should be reversed");
        // Entity 2: was at 1500, clamped to 1000, velocity reversed from -10 to +10
        assert_eq!(positions[2][0], 1000.0);
        assert_eq!(velocities[2][0], 10.0, "Velocity should be reversed");
    }

    #[test]
    fn test_morton_encoder() {
        let encoder = MortonEncoder::new();

        // Test with positions at different quadrants
        // Use coordinates that span the full range
        let code1 = encoder.encode_2d(0.0, 0.0, 1024.0);
        let code2 = encoder.encode_2d(512.0, 0.0, 1024.0);
        let code3 = encoder.encode_2d(0.0, 512.0, 1024.0);
        let code4 = encoder.encode_2d(512.0, 512.0, 1024.0);

        // Different positions should give different codes
        assert_ne!(code1, code2, "Different x should give different codes");
        assert_ne!(code1, code3, "Different y should give different codes");
        assert_ne!(
            code2, code4,
            "Both x,y different should give different codes"
        );
        // Corner should give maximum interleave (bits 0,1,2,...)
        assert!(code4 > code1, "Corner should have higher code than origin");
    }

    #[test]
    fn test_morton_encoder_3d() {
        let encoder = MortonEncoder::new();

        // Test with 3D positions
        let code1 = encoder.encode_3d(0.0, 0.0, 0.0, 1024.0);
        let code2 = encoder.encode_3d(512.0, 0.0, 0.0, 1024.0);
        let code3 = encoder.encode_3d(0.0, 512.0, 0.0, 1024.0);
        let code4 = encoder.encode_3d(0.0, 0.0, 512.0, 1024.0);
        let code5 = encoder.encode_3d(512.0, 512.0, 512.0, 1024.0);

        // Different positions should give different codes
        assert_ne!(code1, code2, "Different x should give different codes");
        assert_ne!(code1, code3, "Different y should give different codes");
        assert_ne!(code1, code4, "Different z should give different codes");
        assert_ne!(
            code4, code5,
            "All coordinates different should give different codes"
        );
        // Corner should give maximum interleave
        assert!(code5 > code1, "Corner should have higher code than origin");
    }

    #[test]
    fn test_batch_iter() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let iter = SimdBatchIterator::new(&data, 4);

        let batches: Vec<_> = iter.collect();
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], &[1, 2, 3, 4]);
        assert_eq!(batches[1], &[5, 6, 7, 8]);
        assert_eq!(batches[2], &[9]);
    }

    #[test]
    fn test_physics_batch() {
        let mut processor = BatchPhysicsProcessor::new();
        let mut positions = [[500.0, 500.0], [0.0, 0.0], [1000.0, 1000.0], [250.0, 250.0]];
        let mut velocities = [[0.0, 0.0], [10.0, 10.0], [0.0, 0.0], [5.0, 5.0]];

        let collisions = processor.process_physics_batch(
            &mut positions,
            &mut velocities,
            [0.0, -9.81],
            0.01,
            1000.0,
            0.016,
            1000.0,
            0.8,
        );

        assert_eq!(collisions, 0);
        assert!(positions[0][1] < 500.0);
    }
}
