// ═══════════════════════════════════════════════════════════════════════════════
// C4 Model - Domain Types for Architecture Diagrams
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 20
//
// The C4 Model is a simple way to model software architecture:
// - System Context: The big picture of what the system does
// - Containers: Applications, data stores, microservices
// - Components: Libraries, modules, or other components
// - Code: Classes, functions, or other code elements
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::Color;

// ═══════════════════════════════════════════════════════════════════════════════
// C4 LEVELS
// ═══════════════════════════════════════════════════════════════════════════════

/// C4 Model hierarchy levels
///
/// Each level provides more detail than the previous one:
/// - Level 0 (System): Person or Software System
/// - Level 1 (Container): Web application, mobile app, database, etc.
/// - Level 2 (Component): Component within a container
/// - Level 3 (Code): Classes, functions within a component
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum C4Level {
    /// Level 0: System Context
    /// Represents a person or software system in the broader context
    #[default]
    System = 0,

    /// Level 1: Container
    /// A container is a runnable unit (web app, mobile app, database, microservice, etc.)
    Container = 1,

    /// Level 2: Component
    /// A component is a grouping of related functionality enclosed behind a nice interface
    Component = 2,

    /// Level 3: Code
    /// Code elements (classes, functions, etc.) - optional and rarely used
    Code = 3,
}

impl C4Level {
    /// Get the level as a u8
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the level as a usize
    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    /// Get the next level of detail
    #[inline(always)]
    pub const fn next_level(self) -> Option<C4Level> {
        match self {
            C4Level::System => Some(C4Level::Container),
            C4Level::Container => Some(C4Level::Component),
            C4Level::Component => Some(C4Level::Code),
            C4Level::Code => None,
        }
    }

    /// Get the previous level (less detail)
    #[inline(always)]
    pub const fn prev_level(self) -> Option<C4Level> {
        match self {
            C4Level::System => None,
            C4Level::Container => Some(C4Level::System),
            C4Level::Component => Some(C4Level::Container),
            C4Level::Code => Some(C4Level::Component),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// C4 ENTITY TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Type of entity in a C4 diagram
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum C4EntityType {
    /// A person using the system
    #[default]
    Person = 0,

    /// A software system (e.g., "Banking System", "Email System")
    SoftwareSystem = 1,

    /// A container (e.g., "Web Application", "Database", "Mobile App")
    Container = 2,

    /// A component within a container
    Component = 3,

    /// A database container
    Database = 4,

    /// A message queue or bus
    MessageQueue = 5,

    /// An external service or API
    ExternalService = 6,

    /// A generic node (for non-C4 entities)
    Generic = 255,
}

impl C4EntityType {
    /// Get the entity type as a u8
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the default color for this entity type
    #[inline(always)]
    pub const fn default_color(self) -> Color {
        match self {
            C4EntityType::Person => Color::rgb(0x99, 0x99, 0x99), // Gray
            C4EntityType::SoftwareSystem => Color::rgb(0x08, 0x42, 0x7A), // Blue (C4 standard)
            C4EntityType::Container => Color::rgb(0x24, 0x92, 0xD6), // Light blue
            C4EntityType::Component => Color::rgb(0x6D, 0xB3, 0xF9), // Even lighter blue
            C4EntityType::Database => Color::rgb(0xEC, 0xF0, 0xF1), // Light gray with cylinder shape
            C4EntityType::MessageQueue => Color::rgb(0xFF, 0xA7, 0x26), // Orange
            C4EntityType::ExternalService => Color::rgb(0xE9, 0x1E, 0x63), // Pink
            C4EntityType::Generic => Color::rgb(0x9E, 0x9E, 0x9E),  // Medium gray
        }
    }

    /// Get the shape type for rendering this entity
    #[inline(always)]
    pub const fn shape_type(self) -> u8 {
        match self {
            C4EntityType::Person => 5,       // Person icon
            C4EntityType::Database => 2,     // Cylinder (represented as rounded rect)
            C4EntityType::MessageQueue => 3, // Queue representation
            _ => 0,                          // Rectangle (default)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLOUD PROVIDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Cloud provider for infrastructure entities
///
/// Used for Infrastructure as Code (IaC) export
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum CloudProvider {
    /// No cloud provider (on-premise or generic)
    #[default]
    None = 0,

    /// Amazon Web Services
    AWS = 1,

    /// Google Cloud Platform
    GCP = 2,

    /// Microsoft Azure
    Azure = 3,

    /// Other cloud provider
    Other = 255,
}

impl CloudProvider {
    /// Get the provider as a u8
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the provider name for IaC export
    #[inline(always)]
    pub fn name(self) -> &'static str {
        match self {
            CloudProvider::None => "none",
            CloudProvider::AWS => "aws",
            CloudProvider::GCP => "gcp",
            CloudProvider::Azure => "azure",
            CloudProvider::Other => "other",
        }
    }

    /// Create from string
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" | "" => Some(CloudProvider::None),
            "aws" | "amazon" => Some(CloudProvider::AWS),
            "gcp" | "google" => Some(CloudProvider::GCP),
            "azure" | "microsoft" => Some(CloudProvider::Azure),
            _ => Some(CloudProvider::Other),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ARCHITECTURE DATA
// ═══════════════════════════════════════════════════════════════════════════════

/// Architecture-specific data for C4 entities
///
/// This contains metadata that's specific to architecture diagrams
/// and doesn't apply to generic diagram entities.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct ArchitectureData {
    /// Name of the entity (e.g., "Web Application", "API Gateway")
    pub name: alloc::string::String,

    /// Description of what this entity does
    pub description: alloc::string::String,

    /// C4 hierarchy level
    pub c4_level: C4Level,

    /// Entity type
    pub entity_type: C4EntityType,

    /// Cloud provider (for IaC export)
    pub cloud_provider: CloudProvider,

    /// Technology stack (e.g., "Rust", "PostgreSQL", "React")
    pub technology: alloc::string::String,

    /// Tags for categorization
    pub tags: alloc::vec::Vec<alloc::string::String>,
}

impl ArchitectureData {
    /// Create new architecture data
    pub fn new(name: alloc::string::String, c4_level: C4Level, entity_type: C4EntityType) -> Self {
        Self {
            name,
            description: alloc::string::String::new(),
            c4_level,
            entity_type,
            cloud_provider: CloudProvider::None,
            technology: alloc::string::String::new(),
            tags: alloc::vec::Vec::new(),
        }
    }

    /// Create with description
    pub fn with_description(
        name: alloc::string::String,
        description: alloc::string::String,
        c4_level: C4Level,
        entity_type: C4EntityType,
    ) -> Self {
        Self {
            name,
            description,
            c4_level,
            entity_type,
            cloud_provider: CloudProvider::None,
            technology: alloc::string::String::new(),
            tags: alloc::vec::Vec::new(),
        }
    }

    /// Set the cloud provider
    pub fn with_cloud_provider(mut self, provider: CloudProvider) -> Self {
        self.cloud_provider = provider;
        self
    }

    /// Set the technology
    pub fn with_technology(mut self, technology: alloc::string::String) -> Self {
        self.technology = technology;
        self
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: alloc::string::String) {
        self.tags.push(tag);
    }

    /// Get the recommended color based on entity type
    pub fn recommended_color(&self) -> Color {
        self.entity_type.default_color()
    }
}

impl Default for ArchitectureData {
    fn default() -> Self {
        Self {
            name: alloc::string::String::from("Unnamed"),
            description: alloc::string::String::new(),
            c4_level: C4Level::System,
            entity_type: C4EntityType::Generic,
            cloud_provider: CloudProvider::None,
            technology: alloc::string::String::new(),
            tags: alloc::vec::Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c4_level_progression() {
        assert_eq!(C4Level::System.next_level(), Some(C4Level::Container));
        assert_eq!(C4Level::Container.next_level(), Some(C4Level::Component));
        assert_eq!(C4Level::Component.next_level(), Some(C4Level::Code));
        assert_eq!(C4Level::Code.next_level(), None);

        assert_eq!(C4Level::Container.prev_level(), Some(C4Level::System));
        assert_eq!(C4Level::System.prev_level(), None);
    }

    #[test]
    fn test_entity_type_colors() {
        assert_eq!(
            C4EntityType::SoftwareSystem.default_color(),
            Color::rgb(0x08, 0x42, 0x7A)
        );
        assert_eq!(
            C4EntityType::Database.default_color(),
            Color::rgb(0xEC, 0xF0, 0xF1)
        );
    }

    #[test]
    fn test_cloud_provider_from_str() {
        assert_eq!(
            CloudProvider::from_str_name("aws"),
            Some(CloudProvider::AWS)
        );
        assert_eq!(
            CloudProvider::from_str_name("AWS"),
            Some(CloudProvider::AWS)
        );
        assert_eq!(
            CloudProvider::from_str_name("gcp"),
            Some(CloudProvider::GCP)
        );
        assert_eq!(
            CloudProvider::from_str_name("azure"),
            Some(CloudProvider::Azure)
        );
        assert_eq!(
            CloudProvider::from_str_name("none"),
            Some(CloudProvider::None)
        );
    }

    #[test]
    fn test_architecture_data_builder() {
        let data = ArchitectureData::new(
            alloc::string::String::from("Web App"),
            C4Level::Container,
            C4EntityType::Container,
        )
        .with_technology(alloc::string::String::from("Rust"))
        .with_cloud_provider(CloudProvider::AWS);

        assert_eq!(data.name, "Web App");
        assert_eq!(data.technology, "Rust");
        assert_eq!(data.cloud_provider, CloudProvider::AWS);
    }
}
