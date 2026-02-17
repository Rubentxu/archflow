// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - BehaviorBlockBuilder Module
//
// This module provides the BehaviorBlock and BehaviorBlockBuilder for defining
// Logic Bricks behaviors directly in the EntityBuilder fluent API.
//
// # Example
//
// ```
// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
//
// let entity = world.spawn()
//     .behavior("move")
//         .sensor(SensorType::KeyShortcut)
//         .controller(Controller::Direct)
//         .actuator(ActuatorType::Move)
//     .build();
// ```
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;

use crate::ecs::component::Component;
use crate::ecs::entity_builder::EntityBuilder;
use crate::mapping::{ActuatorType, Controller, SensorType};

/// A single behavior block containing sensors, controller, and actuators.
///
/// This represents a Logic Bricks "brick" that can be attached to an entity.
/// Each behavior has a name for identification and contains:
/// - Sensors: conditions that trigger the behavior
/// - Controller: logic that combines sensor signals
/// - Actuators: actions to perform when triggered
#[derive(Clone, Debug)]
pub struct BehaviorBlock {
    /// Name identifier for this behavior block
    pub name: String,
    /// List of sensors that trigger this behavior
    pub sensors: Vec<SensorType>,
    /// Optional controller that combines sensor signals
    pub controller: Option<Controller>,
    /// List of actuators that execute when behavior triggers
    pub actuators: Vec<ActuatorType>,
}

impl BehaviorBlock {
    /// Creates a new BehaviorBlock with the given name.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sensors: Vec::new(),
            controller: None,
            actuators: Vec::new(),
        }
    }

    /// Adds a sensor to this behavior block.
    #[inline]
    pub fn with_sensor(mut self, sensor: SensorType) -> Self {
        self.sensors.push(sensor);
        self
    }

    /// Sets the controller for this behavior block.
    #[inline]
    pub fn with_controller(mut self, controller: Controller) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Adds an actuator to this behavior block.
    #[inline]
    pub fn with_actuator(mut self, actuator: ActuatorType) -> Self {
        self.actuators.push(actuator);
        self
    }
}

/// Component that stores multiple behavior blocks for an entity.
///
/// This component allows entities to have multiple behaviors attached,
/// each with their own sensors, controller, and actuators.
#[derive(Clone, Debug, Default)]
pub struct BehaviorBlockComponent {
    /// List of behavior blocks attached to this entity
    pub blocks: Vec<BehaviorBlock>,
}

impl BehaviorBlockComponent {
    /// Creates a new empty BehaviorBlockComponent.
    #[inline]
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Creates a BehaviorBlockComponent with initial blocks.
    #[inline]
    pub fn with_blocks(blocks: Vec<BehaviorBlock>) -> Self {
        Self { blocks }
    }
}

impl Component for BehaviorBlockComponent {
    type Storage = crate::ecs::VecStorage<Self>;
}

/// A builder for creating BehaviorBlocks within the EntityBuilder fluent API.
///
/// This builder is returned by `.behavior("name")` and allows chaining
/// sensors, controllers, and actuators before returning to the EntityBuilder.
pub struct BehaviorBlockBuilder<'w> {
    /// Owned reference to the parent EntityBuilder
    parent: EntityBuilder<'w>,
    /// The current behavior block being built
    current_block: BehaviorBlock,
}

impl<'w> BehaviorBlockBuilder<'w> {
    /// Creates a new BehaviorBlockBuilder.
    #[inline]
    pub fn new(parent: EntityBuilder<'w>, name: impl Into<String>) -> Self {
        Self {
            parent,
            current_block: BehaviorBlock::new(name),
        }
    }

    /// Adds a sensor to the current behavior block.
    ///
    /// Multiple sensors can be added to create complex triggering conditions.
    #[inline]
    pub fn sensor(mut self, sensor: SensorType) -> Self {
        self.current_block.sensors.push(sensor);
        self
    }

    /// Adds multiple sensors to the current behavior block.
    #[inline]
    pub fn sensors(mut self, sensors: impl IntoIterator<Item = SensorType>) -> Self {
        self.current_block.sensors.extend(sensors);
        self
    }

    /// Sets the controller for the current behavior block.
    ///
    /// The controller defines how sensor signals are combined.
    /// Only one controller can be set per block.
    #[inline]
    pub fn controller(mut self, controller: Controller) -> Self {
        self.current_block.controller = Some(controller);
        self
    }

    /// Adds an actuator to the current behavior block.
    ///
    /// Multiple actuators can be added to perform multiple actions.
    #[inline]
    pub fn actuator(mut self, actuator: ActuatorType) -> Self {
        self.current_block.actuators.push(actuator);
        self
    }

    /// Adds multiple actuators to the current behavior block.
    #[inline]
    pub fn actuators(mut self, actuators: impl IntoIterator<Item = ActuatorType>) -> Self {
        self.current_block.actuators.extend(actuators);
        self
    }

    /// Finalizes the current behavior block and returns to the EntityBuilder.
    ///
    /// This method stores the current block in the parent EntityBuilder
    /// and returns the parent EntityBuilder for continued configuration.
    #[inline]
    pub fn end(self) -> EntityBuilder<'w> {
        let mut parent = self.parent;
        parent.behaviors.push(self.current_block);
        parent
    }
}

#[cfg(test)]
mod behavior_block_tests {
    use super::*;

    #[test]
    fn test_behavior_block_creation() {
        let block = BehaviorBlock::new("test_behavior");
        assert_eq!(block.name, "test_behavior");
        assert!(block.sensors.is_empty());
        assert!(block.controller.is_none());
        assert!(block.actuators.is_empty());
    }

    #[test]
    fn test_behavior_block_with_sensor() {
        let block = BehaviorBlock::new("test")
            .with_sensor(SensorType::KeyShortcut)
            .with_sensor(SensorType::MouseOver);

        assert_eq!(block.sensors.len(), 2);
        assert_eq!(block.sensors[0], SensorType::KeyShortcut);
        assert_eq!(block.sensors[1], SensorType::MouseOver);
    }

    #[test]
    fn test_behavior_block_with_controller() {
        let block = BehaviorBlock::new("test").with_controller(Controller::Direct);

        assert!(block.controller.is_some());
        if let Controller::Direct = block.controller.unwrap() {
            // Controller::Direct detected
        } else {
            panic!("Expected Controller::Direct");
        }
    }

    #[test]
    fn test_behavior_block_with_actuator() {
        let block = BehaviorBlock::new("test")
            .with_actuator(ActuatorType::Move)
            .with_actuator(ActuatorType::Highlight);

        assert_eq!(block.actuators.len(), 2);
        assert_eq!(block.actuators[0], ActuatorType::Move);
        assert_eq!(block.actuators[1], ActuatorType::Highlight);
    }

    #[test]
    fn test_behavior_block_component() {
        let mut blocks = Vec::new();
        blocks.push(BehaviorBlock::new("behavior1"));
        blocks.push(BehaviorBlock::new("behavior2"));
        let component = BehaviorBlockComponent::with_blocks(blocks);

        assert_eq!(component.blocks.len(), 2);
    }

    #[test]
    fn test_behavior_block_component_default() {
        let component = BehaviorBlockComponent::default();
        assert!(component.blocks.is_empty());
    }
}
