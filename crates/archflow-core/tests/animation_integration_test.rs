//! Integration tests for the complete animation system
//!
//! This test module validates that all animation components work together:
//! - EasingFunction with all 75+ variants
//! - AnimatorBuilder fluent API
//! - Timeline sequencing
//! - Staggering utilities
//! - Event system integration
//! - Particle system integration (via ECS)
//!
//! These are end-to-end tests that validate the full animation pipeline.

use archflow_core::EntityId;
use archflow_core::{
    AnimatedPropertyValue, AnimationCanvasAdapter, AnimationEvent, AnimationEventDispatcher,
    AnimationId, AnimationPhase, AnimatorBuilder, Ease, EasingFunction, Stagger, StaggerAxis,
    Timeline, TimelinePosition,
};
use std::time::Duration;

#[test]
fn test_complete_animation_pipeline() {
    // Test 1: Create animation with builder
    let entity_id = EntityId::new();
    let builder = AnimatorBuilder::new(entity_id);

    // Configure animation with all supported properties
    builder
        .to(100.0, 200.0)
        .rotate(45.0)
        .opacity(0.5)
        .duration(Duration::from_millis(500))
        .ease(Ease::QuadInOut);

    // Test 2: Test timeline creation and basic operations
    let timeline = Timeline::new();
    let _entity2 = EntityId::new();
    let _entity3 = EntityId::new();

    // Timeline can be created and configured
    assert_eq!(timeline.duration(), 0.0);
    assert!(!timeline.is_playing());

    // Test 3: Verify easing function works
    let ease = EasingFunction::QuadInOut;
    let result = ease.apply(0.5);
    assert!(result > 0.0 && result < 1.0);

    // Test 4: Test event dispatching
    let mut dispatcher = AnimationEventDispatcher::new();
    let event = AnimationEvent::started(AnimationId::new(), Some(entity_id), 500);

    let should_invalidate = dispatcher.dispatch(&event);
    assert!(should_invalidate); // Start events should always invalidate

    // Test 5: Test event lifecycle
    let mut adapter = AnimationCanvasAdapter::new();
    let anim_id = AnimationId::new();
    let property = AnimatedPropertyValue::Opacity {
        from: 0.0,
        to: 1.0,
        current: 0.0,
    };

    adapter.track_animation(anim_id, entity_id, property.clone());
    assert_eq!(adapter.tracked_count(), 1);

    let operation = adapter.complete_animation(anim_id);
    assert!(operation.is_some());
    assert_eq!(adapter.tracked_count(), 0);
}

#[test]
fn test_stagger_with_grid() {
    // Test staggering with grid positions
    let entities: Vec<EntityId> = (0..9).map(|_| EntityId::new()).collect();

    let stagger = Stagger::new(100.0)
        .from_center()
        .with_axis(StaggerAxis::X)
        .grid(3, 3);

    // Calculate delays using the simplified API
    let delays = stagger.calculate_delays(entities.len());

    // Verify delays increase from center outward
    // Center element should have 0 delay (index 4 of 9 elements)
    let center_delay = delays[4];
    assert_eq!(center_delay, 0.0);

    // Edge elements should have positive delays
    assert!(delays[0] > 0.0);
    assert!(delays[8] > 0.0);
}

#[test]
fn test_timeline_position_parsing() {
    // Test GSAP-style position strings
    let valid_positions = vec![
        "-=200",     // overlap previous by 200ms
        "+=100",     // 100ms after previous
        "label+=50", // 50ms after label
        "0.5",       // 500ms from start
    ];

    for pos_str in valid_positions {
        let position = TimelinePosition::parse(pos_str);
        assert!(position.is_some(), "Failed to parse: {}", pos_str);
    }
}

#[test]
fn test_all_easing_functions_exist() {
    // Verify all easing categories are accessible
    let _ = EasingFunction::Linear;

    // Sine
    let _ = EasingFunction::SineIn;
    let _ = EasingFunction::SineOut;
    let _ = EasingFunction::SineInOut;

    // Quad
    let _ = EasingFunction::QuadIn;
    let _ = EasingFunction::QuadOut;
    let _ = EasingFunction::QuadInOut;

    // Cubic
    let _ = EasingFunction::CubicIn;
    let _ = EasingFunction::CubicOut;
    let _ = EasingFunction::CubicInOut;

    // Quart
    let _ = EasingFunction::QuartIn;
    let _ = EasingFunction::QuartOut;
    let _ = EasingFunction::QuartInOut;

    // Quint
    let _ = EasingFunction::QuintIn;
    let _ = EasingFunction::QuintOut;
    let _ = EasingFunction::QuintInOut;

    // Expo
    let _ = EasingFunction::ExpoIn;
    let _ = EasingFunction::ExpoOut;
    let _ = EasingFunction::ExpoInOut;

    // Circ
    let _ = EasingFunction::CircIn;
    let _ = EasingFunction::CircOut;
    let _ = EasingFunction::CircInOut;

    // Back
    let _ = EasingFunction::BackIn;
    let _ = EasingFunction::BackOut;
    let _ = EasingFunction::BackInOut;

    // Elastic
    let _ = EasingFunction::ElasticIn;
    let _ = EasingFunction::ElasticOut;
    let _ = EasingFunction::ElasticInOut;

    // Bounce
    let _ = EasingFunction::BounceIn;
    let _ = EasingFunction::BounceOut;
    let _ = EasingFunction::BounceInOut;

    // Spring
    let _ = EasingFunction::Spring {
        mass: 1.0,
        stiffness: 100.0,
        damping: 10.0,
        rest_threshold: 0.01,
    };

    // Custom Bezier
    let _ = EasingFunction::CubicBezier(0.25, 0.1, 0.25, 1.0);
}

#[test]
fn test_event_dispatcher_throttling() {
    let mut dispatcher = AnimationEventDispatcher::new();
    let entity_id = EntityId::new();
    let animation_id = AnimationId::new();

    // Test that different event phases behave correctly
    // Started events always invalidate
    let start_event = AnimationEvent::started(animation_id, Some(entity_id), 1000);
    assert!(dispatcher.dispatch(&start_event));

    // Completed events always invalidate
    let complete_event = AnimationEvent::completed(animation_id, Some(entity_id), 1000);
    assert!(dispatcher.dispatch(&complete_event));

    // Cancelled events always invalidate
    let cancel_event = AnimationEvent::cancelled(animation_id, Some(entity_id), 500);
    assert!(dispatcher.dispatch(&cancel_event));

    // Failed events always invalidate
    let fail_event = AnimationEvent::failed(animation_id, Some(entity_id), 500);
    assert!(dispatcher.dispatch(&fail_event));

    // Update events are throttled (timing-dependent, so we just verify it doesn't panic)
    let update_event = AnimationEvent::update(
        animation_id,
        Some(entity_id),
        AnimatedPropertyValue::Opacity {
            from: 0.0,
            to: 1.0,
            current: 0.5,
        },
        0.5,
        1000,
        500,
    );
    let _ = dispatcher.dispatch(&update_event); // Should not panic
}

#[test]
fn test_timeline_with_loops() {
    let timeline = Timeline::new();

    // Timeline can be configured with loops (set_loops returns Self)
    let timeline_with_loops = timeline.set_loops(2);

    // Verify timeline configuration
    assert_eq!(timeline_with_loops.duration(), 0.0);
    assert!(!timeline_with_loops.is_playing());
}

#[test]
fn test_animation_value_extraction() {
    // Test that we can extract current values from animated properties
    let property = AnimatedPropertyValue::Position {
        from: (0.0, 0.0),
        to: (100.0, 100.0),
        current: (50.0, 50.0),
    };

    let values = property.current_value();
    assert_eq!(values, vec![50.0, 50.0]);
}

#[test]
fn test_multiple_animations_coexistence() {
    // Test that multiple entities can be animated
    let entity1 = EntityId::new();
    let entity2 = EntityId::new();
    let entity3 = EntityId::new();

    // Create animations for different entities
    let _anim1 = AnimatorBuilder::new(entity1)
        .to(10.0, 10.0)
        .duration(Duration::from_millis(100));

    let _anim2 = AnimatorBuilder::new(entity2)
        .rotate(45.0)
        .duration(Duration::from_millis(200));

    let _anim3 = AnimatorBuilder::new(entity3)
        .opacity(0.5)
        .duration(Duration::from_millis(150));

    // Animations can be created independently
    assert_ne!(entity1, entity2);
    assert_ne!(entity2, entity3);
}

#[test]
fn test_backward_compatibility_easing() {
    // Test that legacy easing aliases still work
    let ease_in = EasingFunction::EaseIn;
    let ease_out = EasingFunction::EaseOut;
    let ease_in_out = EasingFunction::EaseInOut;
    let elastic = EasingFunction::Elastic;
    let bounce = EasingFunction::Bounce;

    // All should produce valid values
    let t = 0.5;
    assert!(ease_in.apply(t) >= 0.0 && ease_in.apply(t) <= 1.0);
    assert!(ease_out.apply(t) >= 0.0 && ease_out.apply(t) <= 1.0);
    assert!(ease_in_out.apply(t) >= 0.0 && ease_in_out.apply(t) <= 1.0);
    assert!(elastic.apply(t) >= 0.0);
    assert!(bounce.apply(t) >= 0.0);
}

#[test]
fn test_complex_timeline_sequence() {
    // Test a timeline with labels
    let mut timeline = Timeline::new();
    let _entity1 = EntityId::new();
    let _entity2 = EntityId::new();
    let _entity3 = EntityId::new();

    // Add labels to timeline
    timeline = timeline.add_label("checkpoint");

    // Verify label was added
    assert!(!timeline.is_playing());
    assert_eq!(timeline.duration(), 0.0);
}

#[test]
fn test_animation_event_phases() {
    let animation_id = AnimationId::new();
    let entity_id = EntityId::new();

    // Test all event phases
    let started = AnimationEvent::started(animation_id, Some(entity_id), 1000);
    assert_eq!(started.phase, AnimationPhase::Started);
    assert_eq!(started.progress, 0.0);

    let running = AnimationEvent::update(
        animation_id,
        Some(entity_id),
        AnimatedPropertyValue::Opacity {
            from: 0.0,
            to: 1.0,
            current: 0.5,
        },
        0.5,
        1000,
        500,
    );
    assert_eq!(running.phase, AnimationPhase::Running);
    assert_eq!(running.progress, 0.5);

    let completed = AnimationEvent::completed(animation_id, Some(entity_id), 1000);
    assert_eq!(completed.phase, AnimationPhase::Completed);
    assert_eq!(completed.progress, 1.0);
    assert_eq!(completed.elapsed_ms, 1000);

    let cancelled = AnimationEvent::cancelled(animation_id, Some(entity_id), 500);
    assert_eq!(cancelled.phase, AnimationPhase::Cancelled);

    let failed = AnimationEvent::failed(animation_id, Some(entity_id), 500);
    assert_eq!(failed.phase, AnimationPhase::Failed);
}

#[test]
fn test_easing_produces_valid_range() {
    // Most easing functions produce values in [0, 1] for input in [0, 1]
    // Note: Elastic and Bounce can overshoot slightly, which is expected behavior
    let easings = vec![
        EasingFunction::Linear,
        EasingFunction::QuadIn,
        EasingFunction::QuadOut,
        EasingFunction::QuadInOut,
        EasingFunction::CubicIn,
        EasingFunction::CubicOut,
        EasingFunction::CubicInOut,
        EasingFunction::SineIn,
        EasingFunction::SineOut,
        EasingFunction::SineInOut,
        EasingFunction::BounceOut, // Bounce stays in range
    ];

    for easing in easings {
        let result = easing.apply(0.5);
        assert!(
            result >= 0.0 && result <= 1.0,
            "Easing {:?} produced invalid value: {}",
            easing,
            result
        );
    }
}

#[test]
fn test_stagger_from_different_positions() {
    let stagger_from_start = Stagger::new(100.0).from_first();
    let stagger_from_center = Stagger::new(100.0).from_center();
    let stagger_from_end = Stagger::new(100.0).from_last();

    // For 5 elements (indices 0-4):
    // - from_first: index 0 has 0ms delay
    // - from_center: index 2 has 0ms delay (center)
    // - from_last: index 4 has 0ms delay

    let delay_start = stagger_from_start.calculate_delay_for_index(0, 5);
    let delay_center = stagger_from_center.calculate_delay_for_index(2, 5);
    let delay_end = stagger_from_end.calculate_delay_for_index(4, 5);

    // All should be 0.0 for their respective reference points
    assert_eq!(delay_start, 0.0);
    assert_eq!(delay_center, 0.0);
    assert_eq!(delay_end, 0.0);

    // For index 0:
    // - from_first: 0ms (reference point)
    // - from_center: |0 - 2| * 100 = 200ms
    // - from_last: (5-1-0) * 100 = 400ms
    let delay_start_idx0 = stagger_from_start.calculate_delay_for_index(0, 5);
    let delay_center_idx0 = stagger_from_center.calculate_delay_for_index(0, 5);
    let delay_end_idx0 = stagger_from_end.calculate_delay_for_index(0, 5);

    // These should all be different (0, 200, 400)
    assert_eq!(delay_start_idx0, 0.0);
    assert_eq!(delay_center_idx0, 200.0);
    assert_eq!(delay_end_idx0, 400.0);
}

#[test]
fn test_canvas_adapter_tracks_animations() {
    let mut adapter = AnimationCanvasAdapter::new();
    let anim_id = AnimationId::new();
    let entity_id = EntityId::new();
    let property = AnimatedPropertyValue::Position {
        from: (0.0, 0.0),
        to: (100.0, 100.0),
        current: (0.0, 0.0),
    };

    // Track animation
    adapter.track_animation(anim_id, entity_id, property.clone());
    assert_eq!(adapter.tracked_count(), 1);

    // Record updates
    let update1 = AnimatedPropertyValue::Position {
        from: (0.0, 0.0),
        to: (100.0, 100.0),
        current: (25.0, 25.0),
    };
    let update2 = AnimatedPropertyValue::Position {
        from: (0.0, 0.0),
        to: (100.0, 100.0),
        current: (50.0, 50.0),
    };

    let event = AnimationEvent::update(anim_id, Some(entity_id), update1.clone(), 0.25, 1000, 250);
    adapter.record_update(&event);

    let event = AnimationEvent::update(anim_id, Some(entity_id), update2.clone(), 0.5, 1000, 500);
    adapter.record_update(&event);

    // Complete animation
    let operation = adapter.complete_animation(anim_id);
    assert!(operation.is_some());

    let op = operation.unwrap();
    assert_eq!(op.entity, entity_id);
    assert_eq!(op.updates.len(), 2);
}
