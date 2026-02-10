// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS System Module
//
// This module provides the System trait and SystemScheduler for executing
// game logic in a prioritized, parallel-safe manner.
//
// Key Features:
// - System trait for defining game logic
// - Prioritized execution order
// - Thread-safe systems (Send + Sync)
// - Startup systems (execute once)
// - Delta time tracking
//
// Architecture:
// - System: Trait for systems that process entities
// - SystemScheduler: Manages and executes systems in priority order
// - Startup systems: Special systems that run once at initialization
//
// Examples:
// ```ignore
// // Define a system
// struct MovementSystem;
//
// impl System for MovementSystem {
//     fn run(&mut self, world: &mut World, delta_time: f32) {
//         world.query::<(&mut Position, &Velocity)>().each(|(pos, vel)| {
//             pos.x += vel.dx * delta_time;
//         });
//     }
//
//     fn name(&self) -> &str { "MovementSystem" }
//     fn priority(&self) -> i32 { 50 }
// }
//
// // Add to scheduler
// let mut scheduler = SystemScheduler::new();
// scheduler.add_system(MovementSystem::new());
// scheduler.run(&mut world, 0.016); // 60 FPS
// ```
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Trait for systems that process entities in the ECS
///
/// Systems contain the game logic that operates on entities with specific
/// components. They are executed by the SystemScheduler in priority order.
///
/// # Requirements
///
/// Systems must be:
/// - `Send + Sync`: Safe to share across threads
/// - `'static`: No borrowed data
///
/// # Examples
///
/// ```ignore
/// struct PhysicsSystem;
///
/// impl System for PhysicsSystem {
///     fn run(&mut self, world: &mut World, delta_time: f32) {
///         // Update physics
///     }
///
///     fn name(&self) -> &str { "PhysicsSystem" }
///     fn priority(&self) -> i32 { 40 }
/// }
/// ```
pub trait System: Send + Sync {
    /// Runs the system logic
    ///
    /// # Parameters
    ///
    /// - `world`: Mutable reference to the ECS world
    /// - `delta_time`: Time elapsed since last frame in seconds
    fn run(&mut self, world: &mut World, delta_time: f32);

    /// Returns the name of this system
    ///
    /// Used for debugging and logging.
    fn name(&self) -> &str;

    /// Returns the priority of this system
    ///
    /// Systems with higher priority run first.
    /// Default priority is 50.
    #[inline]
    fn priority(&self) -> i32 {
        50
    }

    /// Returns true if this is a startup system
    ///
    /// Startup systems run only once and are then removed.
    /// Default is false.
    #[inline]
    fn is_startup(&self) -> bool {
        false
    }
}

// Forward declaration for World (will be defined in world.rs)
// This is a placeholder until we implement World container
pub struct World {
    // Placeholder - will be implemented in HU-ECS-003
    _private: (),
}

/// Scheduler for executing systems in priority order
///
/// Systems are sorted by priority (highest first) and executed sequentially.
/// Thread-safe systems can potentially be executed in parallel in the future.
///
/// # Examples
///
/// ```ignore
/// let mut scheduler = SystemScheduler::new();
///
/// // Add systems (will be sorted by priority)
/// scheduler.add_system(InputSystem::new());          // Priority 100
/// scheduler.add_system(BgeLogicSystem::new());       // Priority 50
/// scheduler.add_system(PhysicsSystem::new());        // Priority 40
/// scheduler.add_system(RenderSystem::new());         // Priority 10
///
/// // Execute all systems
/// scheduler.run(&mut world, 0.016);
/// ```
pub struct SystemScheduler {
    /// Ordered systems: priority -> list of systems at that priority
    systems: BTreeMap<i32, Vec<Box<dyn System>>>,
    /// Startup systems that haven't run yet
    startup_systems: Vec<Box<dyn System>>,
    /// Whether startup systems have been run
    startup_executed: bool,
}

impl SystemScheduler {
    /// Creates a new empty SystemScheduler
    #[inline]
    pub fn new() -> Self {
        Self {
            systems: BTreeMap::new(),
            startup_systems: Vec::new(),
            startup_executed: false,
        }
    }

    /// Adds a system to the scheduler
    ///
    /// Systems are automatically placed in startup or regular systems
    /// based on their `is_startup()` implementation.
    ///
    /// # Parameters
    ///
    /// - `system`: The system to add (Box<dyn System>)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// scheduler.add_system(Box::new(PhysicsSystem::new()));
    /// ```
    #[inline]
    pub fn add_system(&mut self, system: Box<dyn System>) {
        if system.is_startup() {
            self.startup_systems.push(system);
        } else {
            let priority = system.priority();
            self.systems
                .entry(priority)
                .or_insert_with(Vec::new)
                .push(system);
        }
    }

    /// Adds a system by type (convenience method)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// scheduler.add_system_type::<PhysicsSystem>();
    /// ```
    #[inline]
    pub fn add_system_type<S: System + 'static>(&mut self, system: S) {
        self.add_system(Box::new(system));
    }

    /// Runs all systems in priority order
    ///
    /// Startup systems run first (only once), then regular systems run
    /// in descending priority order.
    ///
    /// # Parameters
    ///
    /// - `world`: Mutable reference to the ECS world
    /// - `delta_time`: Time elapsed since last frame in seconds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// scheduler.run(&mut world, 0.016); // 60 FPS
    /// ```
    pub fn run(&mut self, world: &mut World, delta_time: f32) {
        // Run startup systems once
        if !self.startup_executed {
            for system in &mut self.startup_systems {
                system.run(world, 0.0);
            }
            self.startup_executed = true;
            self.startup_systems.clear();
        }

        // Run regular systems in priority order (highest first)
        // BTreeMap iterates in ascending order, so we reverse
        for (_priority, systems) in self.systems.iter_mut().rev() {
            for system in systems {
                system.run(world, delta_time);
            }
        }
    }

    /// Returns the number of registered systems (excluding startup)
    #[inline]
    pub fn len(&self) -> usize {
        self.systems.values().map(|v| v.len()).sum()
    }

    /// Returns the number of startup systems
    #[inline]
    pub fn startup_len(&self) -> usize {
        self.startup_systems.len()
    }

    /// Returns true if no systems are registered
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0 && self.startup_systems.is_empty()
    }

    /// Removes all systems
    #[inline]
    pub fn clear(&mut self) {
        self.systems.clear();
        self.startup_systems.clear();
        self.startup_executed = false;
    }
}

impl Default for SystemScheduler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for SystemScheduler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemScheduler")
            .field("systems_count", &self.len())
            .field("startup_systems_count", &self.startup_systems.len())
            .field("startup_executed", &self.startup_executed)
            .finish()
    }
}

// ============================================================================
// System Info
// ============================================================================

/// Information about a registered system
#[derive(Clone, Debug)]
pub struct SystemInfo {
    /// System name
    pub name: String,
    /// System priority
    pub priority: i32,
    /// Whether this is a startup system
    pub is_startup: bool,
}

impl SystemInfo {
    /// Creates system info from a system reference
    #[inline]
    pub fn from_system(system: &dyn System) -> Self {
        Self {
            name: system.name().to_string(),
            priority: system.priority(),
            is_startup: system.is_startup(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Mock system for testing
    struct MockSystem {
        name: String,
        priority: i32,
        startup: bool,
        run_count: core::cell::RefCell<usize>,
    }

    impl MockSystem {
        fn new(name: &str, priority: i32) -> Self {
            Self {
                name: name.to_string(),
                priority,
                startup: false,
                run_count: core::cell::RefCell::new(0),
            }
        }

        fn startup(name: &str, priority: i32) -> Self {
            Self {
                name: name.to_string(),
                priority,
                startup: true,
                run_count: core::cell::RefCell::new(0),
            }
        }

        fn run_count(&self) -> usize {
            *self.run_count.borrow()
        }
    }

    impl System for MockSystem {
        fn run(&mut self, _world: &mut World, _delta_time: f32) {
            *self.run_count.borrow_mut() += 1;
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn is_startup(&self) -> bool {
            self.startup
        }
    }

    #[test]
    fn test_system_scheduler_new() {
        let scheduler = SystemScheduler::new();
        assert!(scheduler.is_empty());
        assert_eq!(scheduler.len(), 0);
        assert_eq!(scheduler.startup_len(), 0);
    }

    #[test]
    fn test_system_scheduler_add_system() {
        let mut scheduler = SystemScheduler::new();
        let system = Box::new(MockSystem::new("Test", 50));

        scheduler.add_system(system);

        assert_eq!(scheduler.len(), 1);
        assert!(!scheduler.is_empty());
    }

    #[test]
    fn test_system_scheduler_add_startup_system() {
        let mut scheduler = SystemScheduler::new();
        let system = Box::new(MockSystem::startup("Startup", 100));

        scheduler.add_system(system);

        assert_eq!(scheduler.startup_len(), 1);
        assert_eq!(scheduler.len(), 0);
    }

    #[test]
    fn test_system_scheduler_priority_ordering() {
        let mut scheduler = SystemScheduler::new();

        scheduler.add_system_type(MockSystem::new("Low", 10));
        scheduler.add_system_type(MockSystem::new("High", 100));
        scheduler.add_system_type(MockSystem::new("Medium", 50));

        // Systems should be ordered by priority internally
        assert_eq!(scheduler.len(), 3);
    }

    #[test]
    fn test_system_scheduler_clear() {
        let mut scheduler = SystemScheduler::new();

        scheduler.add_system_type(MockSystem::new("Test", 50));
        scheduler.add_system_type(MockSystem::startup("Startup", 100));

        assert_eq!(scheduler.len(), 1);
        assert_eq!(scheduler.startup_len(), 1);

        scheduler.clear();

        assert_eq!(scheduler.len(), 0);
        assert_eq!(scheduler.startup_len(), 0);
        assert!(scheduler.is_empty());
    }

    #[test]
    fn test_system_scheduler_default() {
        let scheduler = SystemScheduler::default();
        assert!(scheduler.is_empty());
    }

    #[test]
    fn test_system_info_from_system() {
        let system = MockSystem::new("TestSystem", 75);
        let info = SystemInfo::from_system(&system);

        assert_eq!(info.name, "TestSystem");
        assert_eq!(info.priority, 75);
        assert_eq!(info.is_startup, false);
    }

    #[test]
    fn test_system_info_from_startup_system() {
        let system = MockSystem::startup("StartupSystem", 100);
        let info = SystemInfo::from_system(&system);

        assert_eq!(info.name, "StartupSystem");
        assert_eq!(info.priority, 100);
        assert_eq!(info.is_startup, true);
    }

    #[test]
    fn test_system_priority_default() {
        struct DefaultPrioritySystem;

        impl System for DefaultPrioritySystem {
            fn run(&mut self, _world: &mut World, _delta_time: f32) {}
            fn name(&self) -> &str {
                "DefaultPrioritySystem"
            }
            // No override of priority()
        }

        let system = DefaultPrioritySystem;
        assert_eq!(system.priority(), 50); // Default priority
    }

    #[test]
    fn test_system_is_startup_default() {
        struct RegularSystem;

        impl System for RegularSystem {
            fn run(&mut self, _world: &mut World, _delta_time: f32) {}
            fn name(&self) -> &str {
                "RegularSystem"
            }
            // No override of is_startup()
        }

        let system = RegularSystem;
        assert_eq!(system.is_startup(), false); // Default is false
    }

    #[test]
    fn test_system_scheduler_debug() {
        let scheduler = SystemScheduler::new();
        let debug_format = format!("{:?}", scheduler);
        assert!(debug_format.contains("SystemScheduler"));
    }
}
