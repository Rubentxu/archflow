// ═══════════════════════════════════════════════════════════════════════════════
// PostProcessSystem - ECS System for Post-Processing Effects
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use crate::ecs::{System, World};

use super::{PostEffect, PostProcessPipeline};

// ═══════════════════════════════════════════════════════════════════════════════
// GpuPostProcessData
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU-ready data for a post-process effect
///
/// This structure is designed to be uploaded directly to GPU uniform buffers.
/// All effects use the same data layout for shader simplicity.
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct GpuPostProcessData {
    /// Effect type: 0 = Bloom, 1 = ColorGrading, 2 = Grayscale
    pub effect_type: u32,
    /// Effect-specific parameter 1
    pub param1: f32,
    /// Effect-specific parameter 2
    pub param2: f32,
    /// Effect-specific parameter 3
    pub param3: f32,
    /// Effect-specific parameter 4
    pub param4: f32,
    /// Padding for alignment (16-byte aligned)
    pub _padding: [f32; 3],
}

impl GpuPostProcessData {
    /// Creates a GpuPostProcessData from a PostEffect
    #[inline]
    #[must_use]
    pub fn from_effect(effect: &PostEffect) -> Self {
        match effect {
            PostEffect::Bloom {
                threshold,
                intensity,
                radius,
            } => Self {
                effect_type: 0,
                param1: *threshold,
                param2: *intensity,
                param3: *radius,
                param4: 0.0,
                _padding: [0.0; 3],
            },
            PostEffect::ColorGrading {
                brightness,
                contrast,
                saturation,
                temperature,
            } => Self {
                effect_type: 1,
                param1: *brightness,
                param2: *contrast,
                param3: *saturation,
                param4: *temperature,
                _padding: [0.0; 3],
            },
            PostEffect::Grayscale { intensity } => Self {
                effect_type: 2,
                param1: *intensity,
                param2: 0.0,
                param3: 0.0,
                param4: 0.0,
                _padding: [0.0; 3],
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PostProcessStats
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics for post-process system processing
#[derive(Clone, Debug, Default)]
pub struct PostProcessStats {
    /// Total number of effects in the pipeline
    pub total_effects: usize,
    /// Number of active (enabled) effects
    pub active_effects: usize,
    /// Number of bloom effects
    pub bloom_count: usize,
    /// Number of color grading effects
    pub color_grading_count: usize,
    /// Number of grayscale effects
    pub grayscale_count: usize,
    /// Whether the pipeline is enabled
    pub pipeline_enabled: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PostProcessSystem
// ═══════════════════════════════════════════════════════════════════════════════

/// ECS System that processes global post-processing effects
///
/// This system is different from other systems because:
/// 1. It processes a GLOBAL resource (PostProcessPipeline), not per-entity components
/// 2. It runs LAST (priority 200) after all rendering is prepared
/// 3. It prepares effect parameters for the GPU post-process pass
///
/// The PostProcessPipeline is stored as a resource in the World.
/// The system retrieves it, processes all enabled effects, and prepares GPU data.
///
/// Priority 200 = runs after ShapeRenderSystem (150), ensuring all rendering is complete
#[derive(Clone, Debug)]
pub struct PostProcessSystem {
    /// Internal buffer for GPU post-process data
    gpu_data: Vec<GpuPostProcessData>,
    /// Statistics
    stats: PostProcessStats,
    /// Local copy of the pipeline (used when World doesn't have it as a resource)
    local_pipeline: PostProcessPipeline,
}

impl PostProcessSystem {
    /// Creates a new PostProcessSystem with default capacity
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            gpu_data: Vec::with_capacity(8), // Most pipelines have < 8 effects
            stats: PostProcessStats::default(),
            local_pipeline: PostProcessPipeline::new(),
        }
    }

    /// Creates a new PostProcessSystem with specified capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            gpu_data: Vec::with_capacity(capacity),
            stats: PostProcessStats::default(),
            local_pipeline: PostProcessPipeline::new(),
        }
    }

    /// Returns the GPU post-process data prepared for rendering
    #[inline]
    #[must_use]
    pub fn gpu_data(&self) -> &[GpuPostProcessData] {
        &self.gpu_data
    }

    /// Returns processing statistics
    #[inline]
    #[must_use]
    pub fn stats(&self) -> &PostProcessStats {
        &self.stats
    }

    /// Clears the internal buffers
    #[inline]
    pub fn clear(&mut self) {
        self.gpu_data.clear();
        self.stats = PostProcessStats::default();
    }

    /// Reserves capacity for GPU data
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.gpu_data.reserve(capacity);
    }

    /// Returns the number of prepared GPU data entries
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.gpu_data.len()
    }

    /// Returns true if no GPU data is prepared
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gpu_data.is_empty()
    }

    /// Returns the capacity of the internal GPU data buffer
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.gpu_data.capacity()
    }

    /// Sets the pipeline for this system
    ///
    /// This replaces the internal pipeline. The system will use this pipeline
    /// when processing unless a pipeline resource is found in the World.
    #[inline]
    pub fn set_pipeline(&mut self, pipeline: PostProcessPipeline) {
        self.local_pipeline = pipeline;
    }

    /// Gets a reference to the internal pipeline
    #[inline]
    #[must_use]
    pub fn pipeline(&self) -> &PostProcessPipeline {
        &self.local_pipeline
    }

    /// Gets a mutable reference to the internal pipeline
    #[inline]
    #[must_use]
    pub fn pipeline_mut(&mut self) -> &mut PostProcessPipeline {
        &mut self.local_pipeline
    }

    /// Processes the pipeline and prepares GPU data
    ///
    /// This is the core logic that converts PostEffect structs into GPU-ready data.
    fn process_pipeline(&mut self, pipeline: &PostProcessPipeline) {
        // Reset stats
        self.stats = PostProcessStats {
            total_effects: pipeline.len(),
            pipeline_enabled: pipeline.is_enabled(),
            active_effects: 0,
            bloom_count: 0,
            color_grading_count: 0,
            grayscale_count: 0,
        };

        // Clear GPU data buffer
        self.gpu_data.clear();

        // Skip if pipeline is disabled
        if !pipeline.is_enabled() {
            return;
        }

        // Process each effect
        for effect in pipeline.effects() {
            // Convert to GPU data
            let gpu_data = GpuPostProcessData::from_effect(effect);
            self.gpu_data.push(gpu_data);

            // Track stats
            self.stats.active_effects += 1;
            match effect {
                PostEffect::Bloom { .. } => self.stats.bloom_count += 1,
                PostEffect::ColorGrading { .. } => self.stats.color_grading_count += 1,
                PostEffect::Grayscale { .. } => self.stats.grayscale_count += 1,
            }
        }
    }
}

impl Default for PostProcessSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement System trait for PostProcessSystem
impl System for PostProcessSystem {
    /// Returns the system name
    #[inline]
    fn name(&self) -> &str {
        "PostProcessSystem"
    }

    /// Returns the system priority
    ///
    /// Priority 200 = runs LAST after ShapeRenderSystem (150)
    /// This ensures all rendering is complete before post-processing
    #[inline]
    fn priority(&self) -> i32 {
        200
    }

    /// Runs the post-process system
    ///
    /// Processes the PostProcessPipeline resource and prepares GPU data.
    /// Uses the internal pipeline if no resource is found in World.
    fn run(&mut self, _world: &mut World, _delta_time: f32) {
        // Note: In the future, PostProcessPipeline could be stored as a World resource.
        // For now, we use the internal pipeline.
        // The _world parameter is kept for API consistency with the System trait.
        self.process_pipeline(&self.local_pipeline.clone());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    // ═══════════════════════════════════════════════════════════════════════════════
    // GpuPostProcessData Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gpu_post_process_data_from_bloom() {
        let effect = PostEffect::bloom(0.8, 0.5, 0.3);
        let gpu_data = GpuPostProcessData::from_effect(&effect);

        assert_eq!(gpu_data.effect_type, 0);
        assert_eq!(gpu_data.param1, 0.8);
        assert_eq!(gpu_data.param2, 0.5);
        assert_eq!(gpu_data.param3, 0.3);
        assert_eq!(gpu_data.param4, 0.0);
    }

    #[test]
    fn test_gpu_post_process_data_from_color_grading() {
        let effect = PostEffect::color_grading(0.1, 1.2, 0.9, -0.2);
        let gpu_data = GpuPostProcessData::from_effect(&effect);

        assert_eq!(gpu_data.effect_type, 1);
        assert_eq!(gpu_data.param1, 0.1);
        assert_eq!(gpu_data.param2, 1.2);
        assert_eq!(gpu_data.param3, 0.9);
        assert_eq!(gpu_data.param4, -0.2);
    }

    #[test]
    fn test_gpu_post_process_data_from_grayscale() {
        let effect = PostEffect::grayscale(0.75);
        let gpu_data = GpuPostProcessData::from_effect(&effect);

        assert_eq!(gpu_data.effect_type, 2);
        assert_eq!(gpu_data.param1, 0.75);
        assert_eq!(gpu_data.param2, 0.0);
        assert_eq!(gpu_data.param3, 0.0);
        assert_eq!(gpu_data.param4, 0.0);
    }

    #[test]
    fn test_gpu_post_process_data_default() {
        let gpu_data = GpuPostProcessData::default();

        assert_eq!(gpu_data.effect_type, 0);
        assert_eq!(gpu_data.param1, 0.0);
        assert_eq!(gpu_data.param2, 0.0);
        assert_eq!(gpu_data.param3, 0.0);
        assert_eq!(gpu_data.param4, 0.0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PostProcessSystem Creation Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_post_process_system_creation() {
        let system = PostProcessSystem::new();
        assert_eq!(system.gpu_data().len(), 0);
        assert!(system.is_empty());
    }

    #[test]
    fn test_post_process_system_name() {
        let system = PostProcessSystem::new();
        assert_eq!(system.name(), "PostProcessSystem");
    }

    #[test]
    fn test_post_process_system_priority() {
        let system = PostProcessSystem::new();
        assert_eq!(system.priority(), 200);
    }

    #[test]
    fn test_post_process_system_default() {
        let system = PostProcessSystem::default();
        assert!(system.is_empty());
    }

    #[test]
    fn test_post_process_system_with_capacity() {
        let system = PostProcessSystem::with_capacity(16);
        assert!(system.capacity() >= 16);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PostProcessSystem Operations Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_post_process_system_clear() {
        let mut system = PostProcessSystem::new();
        system.gpu_data.push(GpuPostProcessData::default());
        system.stats.active_effects = 5;

        system.clear();

        assert_eq!(system.gpu_data().len(), 0);
        assert_eq!(system.stats().active_effects, 0);
    }

    #[test]
    fn test_post_process_system_reserve() {
        let mut system = PostProcessSystem::new();
        system.reserve(32);

        assert!(system.capacity() >= 32);
    }

    #[test]
    fn test_post_process_system_set_pipeline() {
        let mut system = PostProcessSystem::new();
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));

        system.set_pipeline(pipeline);

        assert_eq!(system.pipeline().len(), 1);
    }

    #[test]
    fn test_post_process_system_pipeline_mut() {
        let mut system = PostProcessSystem::new();

        system.pipeline_mut().add_effect(PostEffect::grayscale(0.5));

        assert_eq!(system.pipeline().len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PostProcessSystem Run Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_post_process_system_run_empty() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().total_effects, 0);
        assert_eq!(system.stats().active_effects, 0);
        assert!(system.stats().pipeline_enabled);
    }

    #[test]
    fn test_post_process_system_run_with_effects() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Set up pipeline with effects
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
        pipeline.add_effect(PostEffect::color_grading(0.0, 1.0, 1.0, 0.0));
        system.set_pipeline(pipeline);

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().total_effects, 2);
        assert_eq!(system.stats().active_effects, 2);
        assert_eq!(system.stats().bloom_count, 1);
        assert_eq!(system.stats().color_grading_count, 1);
        assert_eq!(system.gpu_data().len(), 2);
    }

    #[test]
    fn test_post_process_system_run_disabled_pipeline() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Set up pipeline with effects but disabled
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
        pipeline.set_enabled(false);
        system.set_pipeline(pipeline);

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().total_effects, 1);
        assert!(!system.stats().pipeline_enabled);
        assert_eq!(system.stats().active_effects, 0);
        assert_eq!(system.gpu_data().len(), 0);
    }

    #[test]
    fn test_post_process_system_run_multiple_effects() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Set up pipeline with multiple effects
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
        pipeline.add_effect(PostEffect::color_grading(0.0, 1.1, 0.9, 0.0));
        pipeline.add_effect(PostEffect::grayscale(0.3));
        system.set_pipeline(pipeline);

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().total_effects, 3);
        assert_eq!(system.stats().active_effects, 3);
        assert_eq!(system.stats().bloom_count, 1);
        assert_eq!(system.stats().color_grading_count, 1);
        assert_eq!(system.stats().grayscale_count, 1);
        assert_eq!(system.gpu_data().len(), 3);

        // Verify GPU data order matches pipeline order
        assert_eq!(system.gpu_data()[0].effect_type, 0); // Bloom
        assert_eq!(system.gpu_data()[1].effect_type, 1); // ColorGrading
        assert_eq!(system.gpu_data()[2].effect_type, 2); // Grayscale
    }

    #[test]
    fn test_post_process_system_run_clears_previous() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // First run with effects
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
        pipeline.add_effect(PostEffect::grayscale(0.5));
        system.set_pipeline(pipeline);
        system.run(&mut world, 0.016);

        assert_eq!(system.gpu_data().len(), 2);

        // Second run with empty pipeline
        system.set_pipeline(PostProcessPipeline::new());
        system.run(&mut world, 0.016);

        assert_eq!(system.gpu_data().len(), 0);
        assert_eq!(system.stats().active_effects, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PostProcessStats Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_post_process_stats_default() {
        let stats = PostProcessStats::default();

        assert_eq!(stats.total_effects, 0);
        assert_eq!(stats.active_effects, 0);
        assert_eq!(stats.bloom_count, 0);
        assert_eq!(stats.color_grading_count, 0);
        assert_eq!(stats.grayscale_count, 0);
        assert!(!stats.pipeline_enabled);
    }

    #[test]
    fn test_post_process_stats_clone() {
        let stats = PostProcessStats {
            total_effects: 5,
            active_effects: 3,
            bloom_count: 2,
            color_grading_count: 1,
            grayscale_count: 0,
            pipeline_enabled: true,
        };

        let cloned = stats.clone();

        assert_eq!(cloned.total_effects, 5);
        assert_eq!(cloned.active_effects, 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Integration Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_post_process_system_multiple_runs() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Set up pipeline
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::bloom(0.8, 0.5, 0.5));
        system.set_pipeline(pipeline);

        // Run multiple times
        for _ in 0..5 {
            system.run(&mut world, 0.016);
            assert_eq!(system.gpu_data().len(), 1);
            assert_eq!(system.stats().active_effects, 1);
        }
    }

    #[test]
    fn test_post_process_system_effect_parameter_clamping() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Create bloom with out-of-range values (should be clamped)
        let effect = PostEffect::bloom(2.0, 3.0, 2.0); // Will be clamped
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(effect);
        system.set_pipeline(pipeline);

        system.run(&mut world, 0.016);

        // Verify values were clamped during creation
        let gpu_data = &system.gpu_data()[0];
        assert_eq!(gpu_data.param1, 1.0); // threshold clamped to 1.0
        assert_eq!(gpu_data.param2, 2.0); // intensity clamped to 2.0
        assert_eq!(gpu_data.param3, 1.0); // radius clamped to 1.0
    }

    #[test]
    fn test_post_process_system_with_all_effect_types() {
        let mut system = PostProcessSystem::new();
        let mut world = World::new();

        // Set up pipeline with all effect types
        let mut pipeline = PostProcessPipeline::new();
        pipeline.add_effect(PostEffect::Bloom {
            threshold: 0.7,
            intensity: 0.6,
            radius: 0.4,
        });
        pipeline.add_effect(PostEffect::ColorGrading {
            brightness: 0.1,
            contrast: 1.1,
            saturation: 0.95,
            temperature: -0.1,
        });
        pipeline.add_effect(PostEffect::Grayscale { intensity: 0.5 });
        system.set_pipeline(pipeline);

        system.run(&mut world, 0.016);

        assert_eq!(system.gpu_data().len(), 3);
        assert_eq!(system.stats().bloom_count, 1);
        assert_eq!(system.stats().color_grading_count, 1);
        assert_eq!(system.stats().grayscale_count, 1);
    }
}
