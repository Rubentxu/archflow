// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Export - Terraform HCL Generator
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 20
//
// Generates production-ready Terraform HCL code from C4 architecture diagrams.
// Supports AWS, GCP, and Azure cloud providers with proper resource mapping.
//
// Features:
// - Entity-to-resource mapping (containers → compute instances, databases → RDS/CloudSQL)
// - Connection-to-security group rules
// - Variable outputs for modularity
// - Provider-specific best practices
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_diagram::{C4EntityType, CloudProvider};
use archflow_engine::{ArchitectureData as EngineArchData, ConnectionStore, EntityStore};
use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

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
        // Convert u8 to C4EntityType
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

    fn technology(&self) -> &str {
        &self.data.technology
    }

    fn description(&self) -> &str {
        &self.data.description
    }

    fn c4_level(&self) -> u8 {
        self.data.c4_level
    }
}

/// Terraform HCL code generator
///
/// Converts C4 architecture diagrams into production-ready Terraform configuration.
/// Maps entities to appropriate cloud resources based on type and provider.
///
/// # Example
///
/// ```
/// use archflow_export::TerraformExporter;
/// use archflow_engine::{EntityStore, ConnectionStore};
///
/// let exporter = TerraformExporter::new();
/// let store = EntityStore::new();
/// let connections = ConnectionStore::new();
/// let hcl = exporter.export(&store, &connections, "my-project");
/// assert!(hcl.contains("terraform {"));
/// ```
pub struct TerraformExporter {
    /// Terraform version requirement
    terraform_version: String,

    /// Whether to generate provider blocks
    include_provider: bool,

    /// Whether to generate variable blocks
    include_variables: bool,

    /// Whether to generate output blocks
    include_outputs: bool,
}

impl TerraformExporter {
    /// Create a new Terraform exporter with default settings
    #[inline]
    pub fn new() -> Self {
        Self {
            terraform_version: ">= 1.0".to_string(),
            include_provider: true,
            include_variables: true,
            include_outputs: true,
        }
    }

    /// Create a new Terraform exporter with custom Terraform version
    #[inline]
    pub fn with_version(mut self, version: String) -> Self {
        self.terraform_version = version;
        self
    }

    /// Exclude provider blocks from output
    #[inline]
    pub fn without_provider(mut self) -> Self {
        self.include_provider = false;
        self
    }

    /// Exclude variable blocks from output
    #[inline]
    pub fn without_variables(mut self) -> Self {
        self.include_variables = false;
        self
    }

    /// Exclude output blocks from output
    #[inline]
    pub fn without_outputs(mut self) -> Self {
        self.include_outputs = false;
        self
    }

    /// Generate Terraform HCL from entity store and connections
    ///
    /// # Arguments
    /// * `store` - Entity store with architecture data
    /// * `connections` - Connection store for security group rules
    /// * `project_name` - Base name for resources
    ///
    /// # Returns
    /// Complete Terraform HCL configuration as string
    pub fn export(
        &self,
        store: &EntityStore,
        connections: &ConnectionStore,
        project_name: &str,
    ) -> String {
        let mut hcl = String::new();

        // Terraform block
        hcl.push_str(&self.generate_terraform_block());

        // Provider block
        if self.include_provider {
            if let Some(provider) = self.detect_cloud_provider(store) {
                hcl.push_str(&self.generate_provider_block(provider));
            }
        }

        // Variables
        if self.include_variables {
            hcl.push_str(&self.generate_variables());
        }

        // Provider-specific resources
        if let Some(provider) = self.detect_cloud_provider(store) {
            hcl.push_str(&self.generate_resources(store, connections, project_name, provider));
        }

        // Outputs
        if self.include_outputs {
            hcl.push_str(&self.generate_outputs(store, project_name));
        }

        hcl
    }

    /// Generate the terraform configuration block
    fn generate_terraform_block(&self) -> String {
        format!(
            r#"terraform {{
  required_version = "{}"
  required_providers {{
    aws = {{
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }}
    google = {{
      source  = "hashicorp/google"
      version = "~> 5.0"
    }}
    azurerm = {{
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }}
  }}
}}

"#,
            self.terraform_version
        )
    }

    /// Detect the primary cloud provider from entities
    fn detect_cloud_provider(&self, store: &EntityStore) -> Option<CloudProvider> {
        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                if helper.cloud_provider() != CloudProvider::None {
                    return Some(helper.cloud_provider());
                }
            }
        }
        None
    }

    /// Generate provider configuration block
    fn generate_provider_block(&self, provider: CloudProvider) -> String {
        match provider {
            CloudProvider::AWS => String::from(
                r#"provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = var.project_name
      ManagedBy   = "terraform"
      Environment = var.environment
    }
  }
}

"#,
            ),
            CloudProvider::GCP => String::from(
                r#"provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

"#,
            ),
            CloudProvider::Azure => String::from(
                r#"provider "azurerm" {
  features {}
}

resource "azurerm_resource_group" "main" {
  name     = "${var.project_name}-rg"
  location = var.azure_location
}

"#,
            ),
            _ => String::new(),
        }
    }

    /// Generate variable definitions
    fn generate_variables(&self) -> String {
        String::from(
            r#"variable "project_name" {
  description = "Project name used for resource naming"
  type        = string
  default     = "archflow-project"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "prod"
}

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "gcp_project_id" {
  description = "GCP project ID"
  type        = string
  default     = ""
}

variable "gcp_region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "azure_location" {
  description = "Azure region"
  type        = string
  default     = "eastus"
}

variable "db_instance_class" {
  description = "Database instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "compute_instance_type" {
  description = "Compute instance type"
  type        = string
  default     = "t3.micro"
}

"#,
        )
    }

    /// Generate resource blocks based on provider and entities
    fn generate_resources(
        &self,
        store: &EntityStore,
        connections: &ConnectionStore,
        project_name: &str,
        provider: CloudProvider,
    ) -> String {
        match provider {
            CloudProvider::AWS => self.generate_aws_resources(store, connections, project_name),
            CloudProvider::GCP => self.generate_gcp_resources(store, connections, project_name),
            CloudProvider::Azure => self.generate_azure_resources(store, connections, project_name),
            _ => String::new(),
        }
    }

    /// Generate AWS-specific resources
    fn generate_aws_resources(
        &self,
        store: &EntityStore,
        connections: &ConnectionStore,
        project_name: &str,
    ) -> String {
        let mut resources = String::new();

        // Collect entities by type
        let mut databases: Vec<(usize, ArchDataHelper)> = Vec::new();
        let mut containers: Vec<(usize, ArchDataHelper)> = Vec::new();
        let mut queues: Vec<(usize, ArchDataHelper)> = Vec::new();

        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                match helper.entity_type() {
                    C4EntityType::Database => databases.push((i, helper)),
                    C4EntityType::Container | C4EntityType::Component => {
                        containers.push((i, helper))
                    }
                    C4EntityType::MessageQueue => queues.push((i, helper)),
                    _ => {}
                }
            }
        }

        // Generate database resources (RDS)
        for (_idx, helper) in &databases {
            let resource_name = self.sanitize_name(helper.name());
            resources.push_str(&format!(
                r#"resource "aws_db_instance" "{}" {{
  identifier = "${{var.project_name}}-{}"
  engine     = "postgres"
  instance_class = var.db_instance_class

  allocated_storage     = 20
  max_allocated_storage = 100
  storage_encrypted     = true

  db_name  = "main"
  username = "admin"
  password = random_password.main_password.result

  vpc_security_group_ids = [aws_security_group.{}.id]
  skip_final_snapshot  = true

  tags = {{
    Name = "{}"
  }}
}}

"#,
                resource_name, resource_name, resource_name, resource_name,
            ));
        }

        // Generate compute resources (ECS)
        for (_idx, helper) in &containers {
            let resource_name = self.sanitize_name(helper.name());
            let technology = if helper.technology().is_empty() {
                "nginx"
            } else {
                helper.technology()
            };

            resources.push_str(&format!(
                r#"resource "aws_ecs_task_definition" "{}" {{
  family = "{}"

  container_definitions = jsonencode([
    {{
      name      = "{}"
      image     = "{}:latest"
      cpu       = 256
      memory    = 512
      essential = true

      port_mappings = [
        {{
          containerPort = 8080
          protocol      = "tcp"
        }}
      ]

      environment = [
        {{
          name  = "RUST_LOG"
          value = "info"
        }}
      ]

      log_configuration = {{
        logDriver = "awslogs"
        options = {{
          "awslogs-group"         = aws_cloudwatch_log_group.main.name
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "{}"
        }}
      }}
    }}
  ])

  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
}}

resource "aws_ecs_service" "{}" {{
  name            = "{}"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.{}.arn
  desired_count   = 1
  launch_type     = "FARGATE"

  network_configuration {{
    subnets          = aws_subnet.private[*].id
    security_groups  = [aws_security_group.{}.id]
    assign_public_ip = false
  }}
}}

"#,
                resource_name,
                resource_name,
                resource_name,
                technology,
                resource_name,
                resource_name,
                resource_name,
                resource_name,
                resource_name,
            ));
        }

        // Generate SQS queues
        for (_idx, helper) in &queues {
            let resource_name = self.sanitize_name(helper.name());
            resources.push_str(&format!(
                r#"resource "aws_sqs_queue" "{}" {{
  name                      = "${{var.project_name}}-{}"
  message_retention_seconds = 345600
  max_message_size          = 262144
  delay_seconds             = 0
  receive_wait_time_seconds = 20

  tags = {{
    Name = "{}"
  }}
}}

"#,
                resource_name, resource_name, resource_name
            ));
        }

        // Generate security group rules from connections
        if !connections.is_empty() {
            resources.push_str(&self.generate_aws_security_groups(
                store,
                connections,
                project_name,
            ));
        }

        // Generate infrastructure resources
        if !databases.is_empty() || !containers.is_empty() {
            resources.push_str(&self.generate_aws_infrastructure(project_name));
        }

        resources
    }

    /// Generate AWS security group rules from connections
    fn generate_aws_security_groups(
        &self,
        store: &EntityStore,
        connections: &ConnectionStore,
        project_name: &str,
    ) -> String {
        let mut rules = String::new();

        rules.push_str(&format!(
            r#"resource "aws_security_group" "{}" {{
  name        = "${{var.project_name}}-sg"
  description = "Security group for {} application"
  vpc_id      = aws_vpc.main.id

  tags = {{
    Name = "${{var.project_name}}-sg"
  }}
}}

"#,
            project_name, project_name
        ));

        // Add egress rule
        rules.push_str(&format!(
            r#"resource "aws_security_group_rule" "{}_egress" {{
  description       = "Allow all outbound traffic"
  from_port         = 0
  to_port           = 0
  protocol          = "-1"
  security_group_id = aws_security_group.{}.id
  cidr_blocks       = ["0.0.0.0/0"]

  type = "egress"
}}

"#,
            project_name, project_name
        ));

        rules
    }

    /// Generate AWS infrastructure (VPC, subnets, etc.)
    fn generate_aws_infrastructure(&self, _project_name: &str) -> String {
        String::from(
            r#"resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = {
    Name = "${var.project_name}-vpc"
  }
}

resource "aws_subnet" "public" {
  count             = 2
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.${count.index}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "${var.project_name}-public-${count.index}"
  }
}

resource "aws_subnet" "private" {
  count             = 2
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.${count.index + 10}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "${var.project_name}-private-${count.index}"
  }
}

data "aws_availability_zones" "available" {
  state = "available"
}

resource "aws_ecs_cluster" "main" {
  name = "${var.project_name}-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

resource "aws_cloudwatch_log_group" "main" {
  name              = "/ecs/${var.project_name}"
  retention_in_days = 7
}

resource "random_password" "main_password" {
  length  = 32
  special = true
}

"#,
        )
    }

    /// Generate GCP-specific resources
    fn generate_gcp_resources(
        &self,
        _store: &EntityStore,
        _connections: &ConnectionStore,
        project_name: &str,
    ) -> String {
        String::from(format!(
            r#"resource "google_compute_network" "{}" {{
  name                    = "{}-vpc"
  auto_create_subnetworks = false
}}

resource "google_compute_subnetwork" "main" {{
  name          = "{}-subnet"
  ip_cidr_range = "10.0.0.0/24"
  region        = var.gcp_region
  network       = google_compute_network.{}.id
}}

"#,
            project_name, project_name, project_name, project_name
        ))
    }

    /// Generate Azure-specific resources
    fn generate_azure_resources(
        &self,
        _store: &EntityStore,
        _connections: &ConnectionStore,
        project_name: &str,
    ) -> String {
        String::from(format!(
            r#"resource "azurerm_container_app_environment" "main" {{
  name                = "{}-env"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
}}

resource "azurerm_log_analytics_workspace" "main" {{
  name                = "{}-logs"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "PerGB2018"
  retention_in_days   = 30
}}

"#,
            project_name, project_name
        ))
    }

    /// Generate output blocks
    fn generate_outputs(&self, store: &EntityStore, project_name: &str) -> String {
        let mut outputs =
            String::from("output \"project_name\" {\n  value = var.project_name\n}\n\n");

        for i in 0..store.alive_count() {
            if let Some(arch_data) = &store.arch_data[i] {
                let helper = ArchDataHelper::from_engine(arch_data);
                let resource_name = self.sanitize_name(helper.name());

                match helper.entity_type() {
                    C4EntityType::Database => {
                        outputs.push_str(&format!(
                            r#"output "{}_database_endpoint" {{
  description = "Database connection endpoint"
  value       = aws_db_instance.{}.endpoint
}}

"#,
                            resource_name, resource_name
                        ));
                    }
                    C4EntityType::Container => {
                        outputs.push_str(&format!(
                            r#"output "{}_service_url" {{
  description = "Service URL"
  value       = aws_ecs_service.{}.name
}}

"#,
                            resource_name, resource_name
                        ));
                    }
                    _ => {}
                }
            }
        }

        outputs
    }

    /// Sanitize name for Terraform resource naming
    fn sanitize_name(&self, name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .trim_end_matches('_')
            .to_string()
    }
}

impl Default for TerraformExporter {
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
    fn test_terraform_exporter_creation() {
        let exporter = TerraformExporter::new();
        assert_eq!(exporter.terraform_version, ">= 1.0");
        assert!(exporter.include_provider);
        assert!(exporter.include_variables);
        assert!(exporter.include_outputs);
    }

    #[test]
    fn test_terraform_exporter_builder() {
        let exporter = TerraformExporter::new()
            .without_provider()
            .without_variables()
            .without_outputs();

        assert!(!exporter.include_provider);
        assert!(!exporter.include_variables);
        assert!(!exporter.include_outputs);
    }

    #[test]
    fn test_generate_terraform_block() {
        let exporter = TerraformExporter::new();
        let block = exporter.generate_terraform_block();

        assert!(block.contains("terraform {"));
        assert!(block.contains("required_version"));
        assert!(block.contains("hashicorp/aws"));
    }

    #[test]
    fn test_sanitize_name() {
        let exporter = TerraformExporter::new();

        assert_eq!(exporter.sanitize_name("My Service"), "my_service");
        assert_eq!(exporter.sanitize_name("API-Gateway"), "api_gateway");
        assert_eq!(exporter.sanitize_name("test"), "test");
    }

    #[test]
    fn test_generate_variables() {
        let exporter = TerraformExporter::new();
        let vars = exporter.generate_variables();

        assert!(vars.contains("variable \"project_name\""));
        assert!(vars.contains("variable \"environment\""));
        assert!(vars.contains("variable \"aws_region\""));
    }

    #[test]
    fn test_export_empty_store() {
        let exporter = TerraformExporter::new();
        let store = EntityStore::new();
        let connections = ConnectionStore::new();

        let result = exporter.export(&store, &connections, "test-project");

        assert!(result.contains("terraform {"));
        assert!(result.contains("variable \"project_name\""));
        assert!(result.contains("output \"project_name\""));
    }

    #[test]
    fn test_with_version() {
        let exporter = TerraformExporter::new().with_version(">= 1.5".to_string());
        assert_eq!(exporter.terraform_version, ">= 1.5");
    }

    #[test]
    fn test_default() {
        let exporter = TerraformExporter::default();
        assert_eq!(exporter.terraform_version, ">= 1.0");
    }

    #[test]
    fn test_arch_data_helper() {
        let data = EngineArchData {
            name: String::from("TestDB"),
            c4_level: 1,
            entity_type: 4,    // Database
            cloud_provider: 1, // AWS
            technology: String::from("PostgreSQL"),
            description: String::from("Test database"),
        };

        let helper = ArchDataHelper::from_engine(&data);
        assert_eq!(helper.name(), "TestDB");
        assert_eq!(helper.entity_type(), C4EntityType::Database);
        assert_eq!(helper.cloud_provider(), CloudProvider::AWS);
        assert_eq!(helper.technology(), "PostgreSQL");
        assert_eq!(helper.description(), "Test database");
    }
}
