// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Export - Mermaid Diagram Generator
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 20
//
// Generates Mermaid diagram syntax from C4 architecture diagrams.
// Supports multiple diagram types: flowchart, sequence, state, and ER diagrams.
//
// Features:
// - Flowchart generation with proper entity styling
// - C4 model level visualization
// - Connection direction and labels
// - Entity grouping by hierarchy
// - Cloud provider color coding
// ═══════════════════════════════════════════════════════════════════════════════

use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

use archflow_diagram::{C4EntityType, C4Level, CloudProvider};
use archflow_engine::{
    ArchitectureData as EngineArchData, ConnectionStore, EntityStore, LineStyle,
};

/// Helper to convert engine ArchitectureData to diagram types
struct ArchDataHelper<'a> {
    data: &'a EngineArchData,
}

impl<'a> ArchDataHelper<'a> {
    fn from_engine(data: &'a EngineArchData) -> Self {
        Self { data }
    }

    fn name(&self) -> &str {
        &self.data.name
    }

    fn entity_type(&self) -> C4EntityType {
        match self.data.entity_type {
            0 => C4EntityType::Person,
            1 => C4EntityType::SoftwareSystem,
            2 => C4EntityType::Container,
            3 => C4EntityType::Component,
            4 => C4EntityType::Database,
            5 => C4EntityType::MessageQueue,
            6 => C4EntityType::ExternalService,
            _ => C4EntityType::Generic,
        }
    }

    fn cloud_provider(&self) -> CloudProvider {
        match self.data.cloud_provider {
            1 => CloudProvider::AWS,
            2 => CloudProvider::GCP,
            3 => CloudProvider::Azure,
            _ => CloudProvider::None,
        }
    }

    fn c4_level(&self) -> C4Level {
        match self.data.c4_level {
            0 => C4Level::System,
            1 => C4Level::Container,
            2 => C4Level::Component,
            3 => C4Level::Code,
            _ => C4Level::System,
        }
    }

    fn technology(&self) -> &str {
        &self.data.technology
    }

    fn description(&self) -> &str {
        &self.data.description
    }
}

/// Mermaid diagram types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MermaidDiagramType {
    /// Flowchart (default) - best for C4 architecture
    #[default]
    FlowChart = 0,

    /// Sequence diagram - for interaction flows
    Sequence = 1,

    /// State diagram - for state machines
    State = 2,

    /// Entity Relationship - for database schemas
    ER = 3,

    /// Class diagram - for code structure
    Class = 4,
}

impl MermaidDiagramType {
    /// Get the Mermaid keyword for this diagram type
    #[inline]
    pub fn keyword(self) -> &'static str {
        match self {
            MermaidDiagramType::FlowChart => "flowchart TD",
            MermaidDiagramType::Sequence => "sequenceDiagram",
            MermaidDiagramType::State => "stateDiagram-v2",
            MermaidDiagramType::ER => "erDiagram",
            MermaidDiagramType::Class => "classDiagram",
        }
    }
}

/// Direction for flowchart layout
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlowchartDirection {
    /// Top to Bottom (default for C4)
    #[default]
    TD = 0,
    /// Left to Right
    LR = 2,
}

impl FlowchartDirection {
    /// Get the Mermaid direction syntax
    #[inline]
    pub fn syntax(self) -> &'static str {
        match self {
            FlowchartDirection::TD => "TD",
            FlowchartDirection::LR => "LR",
        }
    }
}

/// Styling options for Mermaid diagrams
#[derive(Clone, Debug, PartialEq, Default)]
pub struct MermaidStyle {
    /// Whether to include entity colors
    pub use_colors: bool,

    /// Whether to include cloud provider badges
    pub show_cloud_badges: bool,

    /// Whether to group entities by C4 level
    pub group_by_level: bool,

    /// Whether to include entity descriptions
    pub show_descriptions: bool,
}

impl MermaidStyle {
    /// Create default styling
    #[inline]
    pub fn new() -> Self {
        Self {
            use_colors: true,
            show_cloud_badges: true,
            group_by_level: false,
            show_descriptions: false,
        }
    }

    /// Minimal styling (no colors, no icons)
    #[inline]
    pub fn minimal() -> Self {
        Self {
            use_colors: false,
            show_cloud_badges: false,
            group_by_level: false,
            show_descriptions: false,
        }
    }

    /// Full styling with all features
    #[inline]
    pub fn full() -> Self {
        Self {
            use_colors: true,
            show_cloud_badges: true,
            group_by_level: true,
            show_descriptions: true,
        }
    }
}

/// Mermaid diagram generator
///
/// Converts C4 architecture diagrams into Mermaid diagram syntax.
/// Supports flowchart, sequence, state, ER, and class diagrams.
///
/// # Example
///
/// ```
/// use archflow_export::MermaidExporter;
/// use archflow_engine::{EntityStore, ConnectionStore};
///
/// let exporter = MermaidExporter::new();
/// let store = EntityStore::new();
/// let connections = ConnectionStore::new();
/// let mermaid = exporter.export(&store, &connections);
/// assert!(mermaid.contains("flowchart TD"));
/// ```
pub struct MermaidExporter {
    /// Diagram type to generate
    diagram_type: MermaidDiagramType,

    /// Flowchart direction (for flowchart type)
    direction: FlowchartDirection,

    /// Styling options
    style: MermaidStyle,

    /// Diagram title
    title: Option<String>,
}

impl MermaidExporter {
    /// Create a new Mermaid exporter with default settings
    #[inline]
    pub fn new() -> Self {
        Self {
            diagram_type: MermaidDiagramType::FlowChart,
            direction: FlowchartDirection::TD,
            style: MermaidStyle::new(),
            title: None,
        }
    }

    /// Set the diagram type
    #[inline]
    pub fn with_diagram_type(mut self, diagram_type: MermaidDiagramType) -> Self {
        self.diagram_type = diagram_type;
        self
    }

    /// Set the flowchart direction
    #[inline]
    pub fn with_direction(mut self, direction: FlowchartDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the styling options
    #[inline]
    pub fn with_style(mut self, style: MermaidStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the diagram title
    #[inline]
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Generate Mermaid diagram from entity store and connections
    ///
    /// # Arguments
    /// * `store` - Entity store with architecture data
    /// * `connections` - Connection store for relationships
    ///
    /// # Returns
    /// Complete Mermaid diagram syntax as string
    pub fn export(&self, store: &EntityStore, connections: &ConnectionStore) -> String {
        match self.diagram_type {
            MermaidDiagramType::FlowChart => self.generate_flowchart(store, connections),
            MermaidDiagramType::Sequence => self.generate_sequence_diagram(store, connections),
            MermaidDiagramType::State => self.generate_state_diagram(store),
            MermaidDiagramType::ER => self.generate_er_diagram(store, connections),
            MermaidDiagramType::Class => self.generate_class_diagram(store),
        }
    }

    /// Generate a flowchart diagram (primary C4 diagram type)
    fn generate_flowchart(&self, store: &EntityStore, connections: &ConnectionStore) -> String {
        let mut mermaid = String::new();

        // Header with title
        if let Some(title) = &self.title {
            mermaid.push_str("---\n");
            mermaid.push_str("title: ");
            mermaid.push_str(title);
            mermaid.push_str("\n---\n");
        }

        // Flowchart declaration
        mermaid.push_str("flowchart ");
        mermaid.push_str(self.direction.syntax());
        mermaid.push('\n');

        // Collect entities
        let mut entities: Vec<(usize, ArchDataHelper)> = Vec::new();
        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                entities.push((i, ArchDataHelper::from_engine(arch_data)));
            }
        }

        // Generate entity definitions
        for (idx, helper) in &entities {
            let entity_id = self.entity_id(*idx, helper);
            let label = self.entity_label(helper);

            // Add entity with styling
            if self.style.use_colors {
                let color = self.entity_class(helper);
                mermaid.push_str(&format!("    {}[{}]:::{}\n", entity_id, label, color));
            } else {
                mermaid.push_str(&format!("    {}[{}]\n", entity_id, label));
            }

            // Add description as note if enabled
            if self.style.show_descriptions && !helper.description().is_empty() {
                mermaid.push_str(&format!(
                    "    note_{}[note: {}]\n",
                    entity_id,
                    helper.description()
                ));
                mermaid.push_str(&format!("    {} -.-> note_{}\n", entity_id, entity_id));
            }

            // Add cloud provider badge if enabled
            if self.style.show_cloud_badges && helper.cloud_provider() != CloudProvider::None {
                let badge = self.cloud_badge(helper.cloud_provider());
                mermaid.push_str(&format!("    {}:::{}\n", entity_id, badge));
            }
        }

        // Add subgraphs for C4 levels if grouping
        if self.style.group_by_level {
            mermaid.push('\n');
            let levels = [C4Level::System, C4Level::Container, C4Level::Component];

            for level in levels {
                let level_entities: Vec<&(usize, ArchDataHelper)> = entities
                    .iter()
                    .filter(|(_, h)| h.c4_level() == level)
                    .collect();

                if !level_entities.is_empty() {
                    let level_name = self.level_name(level);
                    mermaid.push_str(&format!("    subgraph {}[{}]\n", level_name, level_name));
                    for (idx, helper) in level_entities {
                        let entity_id = self.entity_id(*idx, helper);
                        mermaid.push_str(&format!("        {}\n", entity_id));
                    }
                    mermaid.push_str("    end\n");
                }
            }
        }

        // Generate connections
        mermaid.push('\n');
        for i in 0..connections.len() {
            let source = connections.sources[i];
            let target = connections.targets[i];

            if !store.is_alive(source) || !store.is_alive(target) {
                continue;
            }

            let src_idx = source.index().0 as usize;
            let tgt_idx = target.index().0 as usize;

            if let (Some(src_arch), Some(tgt_arch)) =
                (&store.arch_data[src_idx], &store.arch_data[tgt_idx])
            {
                let src_helper = ArchDataHelper::from_engine(src_arch);
                let tgt_helper = ArchDataHelper::from_engine(tgt_arch);
                let src_id = self.entity_id(src_idx, &src_helper);
                let tgt_id = self.entity_id(tgt_idx, &tgt_helper);

                // Get arrow style based on line style
                let arrow = self.arrow_style(connections.line_styles[i]);

                mermaid.push_str(&format!("    {} {} {}\n", src_id, arrow, tgt_id));
            }
        }

        // Add styling classes if colors are enabled
        if self.style.use_colors {
            mermaid.push('\n');
            mermaid.push_str(
                "    classDef aws fill:#FF9900,stroke:#146EB4,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef gcp fill:#4285F4,stroke:#34A853,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef azure fill:#0078D4,stroke:#005A9E,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef person fill:#999999,stroke:#333,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef database fill:#ECF0F1,stroke:#95A5A6,stroke-width:2px,color:#333\n",
            );
            mermaid.push_str(
                "    classDef system fill:#08427A,stroke:#053663,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef container fill:#2492D6,stroke:#1A6BA8,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef component fill:#6DB3F9,stroke:#4A90E2,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef queue fill:#FFA726,stroke:#F57C00,stroke-width:2px,color:#fff\n",
            );
            mermaid.push_str(
                "    classDef external fill:#E91E63,stroke:#C2185B,stroke-width:2px,color:#fff\n",
            );
        }

        mermaid
    }

    /// Generate sequence diagram
    fn generate_sequence_diagram(
        &self,
        store: &EntityStore,
        connections: &ConnectionStore,
    ) -> String {
        let mut mermaid = String::new();

        if let Some(title) = &self.title {
            mermaid.push_str("---\n");
            mermaid.push_str("title: ");
            mermaid.push_str(title);
            mermaid.push_str("\n---\n");
        }

        mermaid.push_str("sequenceDiagram\n");

        // Collect unique entities
        let mut entities: Vec<(usize, ArchDataHelper)> = Vec::new();
        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                entities.push((i, ArchDataHelper::from_engine(arch_data)));
            }
        }

        // Define participants
        for (idx, helper) in &entities {
            let entity_id = self.entity_id(*idx, helper);
            let label = self.entity_label(helper);

            let actor_type = if helper.entity_type() == C4EntityType::Person {
                "actor"
            } else {
                "participant"
            };

            mermaid.push_str(&format!("    {} {} as {}\n", actor_type, entity_id, label));
        }

        // Generate interactions from connections
        for i in 0..connections.len() {
            let source = connections.sources[i];
            let target = connections.targets[i];

            if !store.is_alive(source) || !store.is_alive(target) {
                continue;
            }

            let src_idx = source.index().0 as usize;
            let tgt_idx = target.index().0 as usize;

            if let (Some(src_arch), Some(tgt_arch)) =
                (&store.arch_data[src_idx], &store.arch_data[tgt_idx])
            {
                let src_helper = ArchDataHelper::from_engine(src_arch);
                let tgt_helper = ArchDataHelper::from_engine(tgt_arch);
                let src_id = self.entity_id(src_idx, &src_helper);
                let tgt_id = self.entity_id(tgt_idx, &tgt_helper);

                mermaid.push_str(&format!("    {}->>{}: Request\n", src_id, tgt_id));

                if tgt_helper.entity_type() != C4EntityType::ExternalService {
                    mermaid.push_str(&format!("    {}-->>{}: Response\n\n", tgt_id, src_id));
                }
            }
        }

        mermaid
    }

    /// Generate state diagram
    fn generate_state_diagram(&self, store: &EntityStore) -> String {
        let mut mermaid = String::new();

        if let Some(title) = &self.title {
            mermaid.push_str("---\n");
            mermaid.push_str("title: ");
            mermaid.push_str(title);
            mermaid.push_str("\n---\n");
        }

        mermaid.push_str("stateDiagram-v2\n");
        mermaid.push_str("    [*] --> Idle\n");

        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                let state_name = self.sanitize_name(helper.name());
                mermaid.push_str(&format!("    state {} {{\n", state_name));

                if helper.c4_level() == C4Level::Container {
                    mermaid.push_str("        [*] --> Running\n");
                    mermaid.push_str("        Running --> Stopped\n");
                    mermaid.push_str("        Stopped --> [*]\n");
                }

                mermaid.push_str("    }\n");
            }
        }

        mermaid
    }

    /// Generate Entity Relationship diagram
    fn generate_er_diagram(&self, store: &EntityStore, connections: &ConnectionStore) -> String {
        let mut mermaid = String::new();

        if let Some(title) = &self.title {
            mermaid.push_str("---\n");
            mermaid.push_str("title: ");
            mermaid.push_str(title);
            mermaid.push_str("\n---\n");
        }

        mermaid.push_str("erDiagram\n");

        // Generate entities
        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                let entity_name = self.sanitize_name(helper.name());

                if helper.entity_type() == C4EntityType::Database {
                    mermaid.push_str(&format!("    {} {{\n", entity_name));
                    mermaid.push_str("        uuid UUID\n");
                    mermaid.push_str("        created_at TIMESTAMP\n");
                    mermaid.push_str("    }\n");
                } else if helper.entity_type() == C4EntityType::Container {
                    mermaid.push_str(&format!("    {} {{\n", entity_name));
                    mermaid.push_str("        id UUID PK\n");
                    mermaid.push_str("        name VARCHAR\n");
                    mermaid.push_str("    }\n");
                }
            }
        }

        // Generate relationships
        for i in 0..connections.len() {
            let source = connections.sources[i];
            let target = connections.targets[i];

            if !store.is_alive(source) || !store.is_alive(target) {
                continue;
            }

            let src_idx = source.index().0 as usize;
            let tgt_idx = target.index().0 as usize;

            if let (Some(src_arch), Some(tgt_arch)) =
                (&store.arch_data[src_idx], &store.arch_data[tgt_idx])
            {
                let src_helper = ArchDataHelper::from_engine(src_arch);
                let tgt_helper = ArchDataHelper::from_engine(tgt_arch);
                let src_entity = self.sanitize_name(src_helper.name());
                let tgt_entity = self.sanitize_name(tgt_helper.name());

                let relationship = if tgt_helper.entity_type() == C4EntityType::Database {
                    "have"
                } else {
                    "use"
                };
                mermaid.push_str(&format!(
                    "    {} {} ||--o{{ {}\n",
                    src_entity, relationship, tgt_entity
                ));
            }
        }

        mermaid
    }

    /// Generate class diagram
    fn generate_class_diagram(&self, store: &EntityStore) -> String {
        let mut mermaid = String::new();

        if let Some(title) = &self.title {
            mermaid.push_str("---\n");
            mermaid.push_str("title: ");
            mermaid.push_str(title);
            mermaid.push_str("\n---\n");
        }

        mermaid.push_str("classDiagram\n");

        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                let class_name = self.sanitize_name(helper.name());

                if helper.entity_type() == C4EntityType::Container {
                    mermaid.push_str(&format!("    class {} {{\n", class_name));
                    mermaid.push_str("        +id: UUID\n");
                    mermaid.push_str("        +initialize()\n");
                    mermaid.push_str("        +process()\n");
                    mermaid.push_str("    }\n");
                } else if helper.entity_type() == C4EntityType::Component {
                    mermaid.push_str(&format!("    class {} {{\n", class_name));
                    mermaid.push_str("        +execute()\n");
                    mermaid.push_str("    }\n");
                }
            }
        }

        mermaid
    }

    /// Get entity ID for Mermaid syntax
    fn entity_id(&self, idx: usize, helper: &ArchDataHelper) -> String {
        format!("{}_{}", self.sanitize_name(helper.name()), idx)
    }

    /// Get entity label
    fn entity_label(&self, helper: &ArchDataHelper) -> String {
        let mut label = helper.name().to_string();

        // Add technology if present
        if !helper.technology().is_empty() {
            label.push_str("<br/>");
            label.push_str(&format!("[{}]", helper.technology()));
        }

        label
    }

    /// Get CSS class for entity
    fn entity_class(&self, helper: &ArchDataHelper) -> String {
        match helper.cloud_provider() {
            CloudProvider::AWS => "aws".to_string(),
            CloudProvider::GCP => "gcp".to_string(),
            CloudProvider::Azure => "azure".to_string(),
            _ => match helper.entity_type() {
                C4EntityType::Person => "person".to_string(),
                C4EntityType::Database => "database".to_string(),
                C4EntityType::SoftwareSystem => "system".to_string(),
                C4EntityType::Container => "container".to_string(),
                C4EntityType::Component => "component".to_string(),
                C4EntityType::MessageQueue => "queue".to_string(),
                C4EntityType::ExternalService => "external".to_string(),
                _ => "default".to_string(),
            },
        }
    }

    /// Get cloud badge style
    fn cloud_badge(&self, provider: CloudProvider) -> String {
        match provider {
            CloudProvider::AWS => "aws".to_string(),
            CloudProvider::GCP => "gcp".to_string(),
            CloudProvider::Azure => "azure".to_string(),
            _ => String::new(),
        }
    }

    /// Get arrow style based on line style
    fn arrow_style(&self, line_style: LineStyle) -> &'static str {
        match line_style {
            LineStyle::Direct => "-->",
            LineStyle::Orthogonal => "-.->",
            LineStyle::Step => "==>",
            LineStyle::Bezier => "~>~",
        }
    }

    /// Get C4 level name
    fn level_name(&self, level: C4Level) -> &'static str {
        match level {
            C4Level::System => "SystemContext",
            C4Level::Container => "Containers",
            C4Level::Component => "Components",
            C4Level::Code => "Code",
        }
    }

    /// Sanitize name for Mermaid syntax
    fn sanitize_name(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_end_matches('_')
            .to_string()
    }
}

impl Default for MermaidExporter {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mermaid_exporter_creation() {
        let exporter = MermaidExporter::new();
        assert_eq!(exporter.diagram_type, MermaidDiagramType::FlowChart);
        assert_eq!(exporter.direction, FlowchartDirection::TD);
    }

    #[test]
    fn test_with_diagram_type() {
        let exporter = MermaidExporter::new().with_diagram_type(MermaidDiagramType::Sequence);
        assert_eq!(exporter.diagram_type, MermaidDiagramType::Sequence);
    }

    #[test]
    fn test_diagram_type_keywords() {
        assert_eq!(MermaidDiagramType::FlowChart.keyword(), "flowchart TD");
        assert_eq!(MermaidDiagramType::Sequence.keyword(), "sequenceDiagram");
        assert_eq!(MermaidDiagramType::State.keyword(), "stateDiagram-v2");
        assert_eq!(MermaidDiagramType::ER.keyword(), "erDiagram");
        assert_eq!(MermaidDiagramType::Class.keyword(), "classDiagram");
    }

    #[test]
    fn test_flowchart_direction_syntax() {
        assert_eq!(FlowchartDirection::TD.syntax(), "TD");
        assert_eq!(FlowchartDirection::LR.syntax(), "LR");
    }

    #[test]
    fn test_sanitize_name() {
        let exporter = MermaidExporter::new();
        assert_eq!(exporter.sanitize_name("My Service"), "My_Service");
        assert_eq!(exporter.sanitize_name("API-Gateway"), "API_Gateway");
        assert_eq!(exporter.sanitize_name("test"), "test");
    }

    #[test]
    fn test_export_empty_store() {
        let exporter = MermaidExporter::new();
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let result = exporter.export(&store, &connections);
        assert!(result.contains("flowchart TD"));
    }

    #[test]
    fn test_export_with_title() {
        let exporter = MermaidExporter::new().with_title("Test Diagram".to_string());
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let result = exporter.export(&store, &connections);
        assert!(result.contains("title: Test Diagram"));
    }

    #[test]
    fn test_default() {
        let exporter = MermaidExporter::default();
        assert_eq!(exporter.diagram_type, MermaidDiagramType::FlowChart);
    }
}
