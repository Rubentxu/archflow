/**
 * Entity property schemas using Zod for validation
 *
 * Defines validation rules for each entity type's properties.
 * Used by PropertiesPanel for form validation.
 */

import { z } from "zod";

/**
 * Base property schema shared by all entities
 */
export const basePropertySchema = z.object({
  name: z.string().min(1, "Name is required").max(100, "Name too long"),
  description: z.string().max(500, "Description too long").optional(),
});

/**
 * AWS EC2 Instance properties schema
 */
export const ec2PropertiesSchema = basePropertySchema.extend({
  instanceType: z.enum([
    "t3.micro",
    "t3.small",
    "t3.medium",
    "t3.large",
    "t3.xlarge",
    "t3.2xlarge",
    "m6i.large",
    "m6i.xlarge",
    "m6i.2xlarge",
    "c6i.large",
    "c6i.xlarge",
  ]),
  region: z.string().min(1, "Region is required"),
  availabilityZone: z.string().optional(),
  tags: z.array(z.object({ key: z.string(), value: z.string() })).optional(),
});

/**
 * AWS Lambda Function properties schema
 */
export const lambdaPropertiesSchema = basePropertySchema.extend({
  runtime: z.enum([
    "nodejs18.x",
    "nodejs20.x",
    "python3.9",
    "python3.10",
    "python3.11",
    "python3.12",
    "java11",
    "java17",
    "dotnet6",
    "dotnet8",
    "go1.x",
    "ruby3.2",
  ]),
  timeout: z.number().min(1, "Timeout must be at least 1 second").max(900, "Timeout cannot exceed 15 minutes"),
  memory: z.number().min(128, "Memory must be at least 128 MB").max(10240, "Memory cannot exceed 10 GB"),
  handler: z.string().min(1, "Handler is required"),
  architecture: z.enum(["x86_64", "arm64"]).default("x86_64"),
});

/**
 * AWS RDS Database properties schema
 */
export const rdsPropertiesSchema = basePropertySchema.extend({
  engine: z.enum([
    "postgres",
    "mysql",
    "oracle-ee",
    "sqlserver-ee",
    "sqlserver-se",
    "sqlserver-ex",
    "sqlserver-web",
    "aurora-postgresql",
    "aurora-mysql",
  ]),
  instanceClass: z.string().min(1, "Instance class is required"),
  allocatedStorage: z.number().min(20, "Minimum 20 GB").max(65536, "Maximum 64 TB"),
  multiAZ: z.boolean().default(false),
  publiclyAccessible: z.boolean().default(false),
  backupRetention: z.number().min(1).max(35).default(7),
});

/**
 * AWS S3 Bucket properties schema
 */
export const s3PropertiesSchema = basePropertySchema.extend({
  bucketName: z.string().regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$/, "Invalid bucket name").min(3).max(63),
  versioning: z.boolean().default(false),
  encryption: z.enum(["SSE-S3", "SSE-KMS", "SSE-C", "NONE"]).default("SSE-S3"),
  blockPublicAccess: z.boolean().default(true),
  lifecycleRules: z.array(z.object({
    name: z.string(),
    prefix: z.string().optional(),
    expirationDays: z.number().optional(),
    transitionDays: z.number().optional(),
    storageClass: z.enum(["STANDARD", "INTELLIGENT_TIERING", "STANDARD_IA", "ONEZONE_IA", "GLACIER", "DEEP_ARCHIVE"]).optional(),
  })).optional(),
});

/**
 * AWS API Gateway properties schema
 */
export const apiGatewayPropertiesSchema = basePropertySchema.extend({
  apiType: z.enum(["REST", "HTTP", "WEBSOCKET"]),
  protocolType: z.enum(["HTTP", "WEBSOCKET"]),
  endpointType: z.enum(["EDGE", "REGIONAL", "PRIVATE"]).default("EDGE"),
  apiKeyRequired: z.boolean().default(false),
  throttlingRate: z.number().min(0).default(10000),
  throttlingBurst: z.number().min(0).default(5000),
});

/**
 * AWS ELB properties schema
 */
export const elbPropertiesSchema = basePropertySchema.extend({
  loadBalancerType: z.enum(["application", "network", "gateway"]),
  scheme: z.enum(["internet-facing", "internal"]).default("internet-facing"),
  crossZoneLoadBalancing: z.boolean().default(true),
  idleTimeout: z.number().min(1).max(4000).default(60),
});

/**
 * Generic container/actor properties
 */
export const containerPropertiesSchema = basePropertySchema.extend({
  label: z.string().max(100).optional(),
  color: z.string().regex(/^#[0-9A-Fa-f]{6}$/).optional(),
  borderColor: z.string().regex(/^#[0-9A-Fa-f]{6}$/).optional(),
});

/**
 * Type union for all entity property schemas
 */
export type EntityPropertySchema =
  | typeof ec2PropertiesSchema
  | typeof lambdaPropertiesSchema
  | typeof rdsPropertiesSchema
  | typeof s3PropertiesSchema
  | typeof apiGatewayPropertiesSchema
  | typeof elbPropertiesSchema
  | typeof containerPropertiesSchema;

/**
 * Map entity types to their property schemas
 */
export const entitySchemas: Record<string, EntityPropertySchema> = {
  "aws-ec2": ec2PropertiesSchema,
  "aws-lambda": lambdaPropertiesSchema,
  "aws-rds": rdsPropertiesSchema,
  "aws-s3": s3PropertiesSchema,
  "aws-api-gateway": apiGatewayPropertiesSchema,
  "aws-elb": elbPropertiesSchema,
  "container": containerPropertiesSchema,
  "actor": containerPropertiesSchema,
  "database": containerPropertiesSchema,
  "queue": containerPropertiesSchema,
  "service": containerPropertiesSchema,
};

/**
 * Get the schema for a specific entity type
 */
export function getSchemaForEntityType(entityType: string): EntityPropertySchema {
  return entitySchemas[entityType] || containerPropertiesSchema;
}

/**
 * Validate properties against the appropriate schema
 */
export function validateEntityProperties(
  entityType: string,
  properties: Record<string, unknown>
): { success: boolean; errors?: Record<string, string> } {
  const schema = getSchemaForEntityType(entityType);
  const result = schema.safeParse(properties);

  if (result.success) {
    return { success: true };
  }

  const errors: Record<string, string> = {};
  for (const issue of result.error.issues) {
    const path = issue.path.join(".");
    if (!errors[path]) {
      errors[path] = issue.message;
    }
  }

  return { success: false, errors };
}
