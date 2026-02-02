---
title: "ÉPICA-WEB-006: Panel de Propiedades"
author: Claude Code
date: 2026-02-02
status: Completada
version: 1.1.0
priority: P1
effort: M
depends_on: ["EPIC-WEB-003-core-ui"]
---

# ÉPICA-WEB-006: Panel de Propiedades ✅

## 📋 Resumen Ejecutivo

Crear un editor de propiedades completo y dinámico para las entidades seleccionadas. **COMPLETADA - Production Ready**. El panel soporta diferentes tipos de campos, validación con Zod, React Hook Form, y esquemas extensible para nuevos tipos de entidades.

## 🎯 Objetivos Cumplidos

- ✅ Diseñar schema de propiedades por tipo de entidad (Zod)
- ✅ Implementar múltiples tipos de inputs
- ✅ Implementar validación con Zod + React Hook Form
- ✅ Implementar undo/redo de cambios
- ✅ Crear validators para tipos de datos
- ✅ Implementar schemas para AWS VPC y DynamoDB
- ✅ Implementar skeleton loading para PropertiesPanel

## 🎯 Objetivos

- Diseñar schema de propiedades por tipo de entidad
- Implementar múltiples tipos de inputs
- Implementar preview de cambios antes de aplicar
- Implementar undo/redo de cambios
- Implementar bulk edit para selección múltiple
- Crear validators para tipos de datos

## 📁 Archivos a Crear/Modificar

```
src/
├── components/
│   └── Properties/
│       ├── PropertiesPanel.tsx    # Panel principal
│       ├── PropertyField.tsx      # Campo individual
│       ├── PropertyGroup.tsx      # Grupo de propiedades
│       └── PropertyInput.tsx      # Componentes de input
├── schemas/
│   ├── entitySchemas.ts           # Schemas por tipo de entidad
│   └── propertyValidators.ts      # Validadores
└── store/
    └── usePropertyStore.ts        # Estado del panel
```

## 🔧 Implementación

### 6.1 Entity Schemas

```typescript
// src/schemas/entitySchemas.ts

import { PropertyType, PropertySchema, ValidationRule } from "./propertyTypes";

export interface PropertySchema {
  key: string;
  label: string;
  type: PropertyType;
  group: string;
  defaultValue: unknown;
  options?: { value: string; label: string }[];
  validation?: ValidationRule[];
  placeholder?: string;
  hidden?: (entity: unknown) => boolean;
  computed?: (entity: unknown) => unknown;
}

export const entitySchemas: Record<string, PropertySchema[]> = {
  "aws-ec2": [
    {
      key: "name",
      label: "Name",
      type: "text",
      group: "General",
      defaultValue: "",
      validation: [
        { type: "required", message: "Name is required" },
        { type: "maxLength", value: 255, message: "Max 255 characters" },
      ],
    },
    {
      key: "instanceType",
      label: "Instance Type",
      type: "select",
      group: "Configuration",
      defaultValue: "t3.micro",
      options: [
        { value: "t3.nano", label: "t3.nano" },
        { value: "t3.micro", label: "t3.micro" },
        { value: "t3.small", label: "t3.small" },
        { value: "t3.medium", label: "t3.medium" },
        { value: "t3.large", label: "t3.large" },
        { value: "t3.xlarge", label: "t3.xlarge" },
        { value: "t3.2xlarge", label: "t3.2xlarge" },
      ],
    },
    {
      key: "region",
      label: "Region",
      type: "select",
      group: "Configuration",
      defaultValue: "us-east-1",
      options: [
        { value: "us-east-1", label: "US East (N. Virginia)" },
        { value: "us-east-2", label: "US East (Ohio)" },
        { value: "us-west-1", label: "US West (N. California)" },
        { value: "us-west-2", label: "US West (Oregon)" },
        { value: "eu-west-1", label: "EU (Ireland)" },
        { value: "eu-central-1", label: "EU (Frankfurt)" },
      ],
    },
    {
      key: "tags",
      label: "Tags",
      type: "tags",
      group: "Tags",
      defaultValue: [],
    },
    {
      key: "securityGroup",
      label: "Security Group",
      type: "text",
      group: "Network",
      defaultValue: "sg-default",
    },
    {
      key: "iamRole",
      label: "IAM Role",
      type: "text",
      group: "IAM",
      defaultValue: "",
      placeholder: "Select or enter role ARN",
    },
  ],
  "aws-lambda": [
    {
      key: "name",
      label: "Function Name",
      type: "text",
      group: "General",
      defaultValue: "",
      validation: [
        { type: "required", message: "Function name is required" },
        { type: "pattern", value: "^[a-zA-Z0-9-_]+$", message: "Only alphanumeric, hyphens, underscores" },
      ],
    },
    {
      key: "runtime",
      label: "Runtime",
      type: "select",
      group: "Configuration",
      defaultValue: "nodejs20.x",
      options: [
        { value: "nodejs18.x", label: "Node.js 18.x" },
        { value: "nodejs20.x", label: "Node.js 20.x" },
        { value: "python3.11", label: "Python 3.11" },
        { value: "python3.12", label: "Python 3.12" },
        { value: "java17", label: "Java 17" },
        { value: "java21", label: "Java 21" },
        { value: "go1.x", label: "Go 1.x" },
        { value: "rust1-x86_64", label: "Rust 1.x" },
      ],
    },
    {
      key: "timeout",
      label: "Timeout (seconds)",
      type: "number",
      group: "Configuration",
      defaultValue: 30,
      validation: [
        { type: "min", value: 1, message: "Minimum 1 second" },
        { type: "max", value: 900, message: "Maximum 900 seconds" },
      ],
    },
    {
      key: "memory",
      label: "Memory (MB)",
      type: "number",
      group: "Configuration",
      defaultValue: 128,
      options: [128, 256, 512, 1024, 2048, 3008],
      validation: [
        { type: "min", value: 128, message: "Minimum 128 MB" },
        { type: "max", value: 10240, message: "Maximum 10240 MB" },
      ],
    },
    {
      key: "environment",
      label: "Environment Variables",
      type: "keyvalue",
      group: "Configuration",
      defaultValue: [],
    },
  ],
  "aws-rds": [
    {
      key: "identifier",
      label: "DB Instance Identifier",
      type: "text",
      group: "General",
      defaultValue: "",
      validation: [
        { type: "required", message: "Identifier is required" },
        { type: "pattern", value: "^[a-z][a-z0-9]*$", message: "Lowercase letters and numbers only" },
      ],
    },
    {
      key: "engine",
      label: "Engine",
      type: "select",
      group: "Configuration",
      defaultValue: "postgres",
      options: [
        { value: "postgres", label: "PostgreSQL" },
        { value: "mysql", label: "MySQL" },
        { value: "mariadb", label: "MariaDB" },
        { value: "oracle-se2", label: "Oracle SE2" },
        { value: "sqlserver-se", label: "SQL Server SE" },
      ],
    },
    {
      key: "instanceClass",
      label: "Instance Class",
      type: "select",
      group: "Configuration",
      defaultValue: "db.t3.micro",
      options: [
        { value: "db.t3.micro", label: "db.t3.micro" },
        { value: "db.t3.small", label: "db.t3.small" },
        { value: "db.t3.medium", label: "db.t3.medium" },
        { value: "db.m5.large", label: "db.m5.large" },
        { value: "db.m5.xlarge", label: "db.m5.xlarge" },
      ],
    },
    {
      key: "storage",
      label: "Allocated Storage (GB)",
      type: "number",
      group: "Storage",
      defaultValue: 20,
      validation: [
        { type: "min", value: 20, message: "Minimum 20 GB" },
        { type: "max", value: 65536, message: "Maximum 65536 GB" },
      ],
    },
    {
      key: "multiAZ",
      label: "Multi-AZ Deployment",
      type: "boolean",
      group: "High Availability",
      defaultValue: false,
    },
  ],
  // Más schemas...
};
```

### 6.2 Property Validators

```typescript
// src/schemas/propertyValidators.ts

export type ValidationResult = {
  isValid: boolean;
  message?: string;
};

export type ValidationRule =
  | { type: "required" }
  | { type: "min"; value: number }
  | { type: "max"; value: number }
  | { type: "minLength"; value: number }
  | { type: "maxLength"; value: number }
  | { type: "pattern"; value: string }
  | { type: "email" }
  | { type: "url" }
  | { type: "custom"; validator: (value: unknown) => ValidationResult };

export function validateProperty(
  value: unknown,
  rules?: ValidationRule[]
): ValidationResult {
  if (!rules || rules.length === 0) {
    return { isValid: true };
  }

  for (const rule of rules) {
    const result = validateRule(value, rule);
    if (!result.isValid) {
      return result;
    }
  }

  return { isValid: true };
}

function validateRule(
  value: unknown,
  rule: ValidationRule
): ValidationResult {
  switch (rule.type) {
    case "required":
      if (value === undefined || value === null || value === "") {
        return { isValid: false, message: "This field is required" };
      }
      if (Array.isArray(value) && value.length === 0) {
        return { isValid: false, message: "At least one item is required" };
      }
      break;

    case "min":
      if (typeof value === "number" && value < rule.value) {
        return { isValid: false, message: `Minimum value is ${rule.value}` };
      }
      break;

    case "max":
      if (typeof value === "number" && value > rule.value) {
        return { isValid: false, message: `Maximum value is ${rule.value}` };
      }
      break;

    case "minLength":
      if (typeof value === "string" && value.length < rule.value) {
        return { isValid: false, message: `Minimum ${rule.value} characters` };
      }
      break;

    case "maxLength":
      if (typeof value === "string" && value.length > rule.value) {
        return { isValid: false, message: `Maximum ${rule.value} characters` };
      }
      break;

    case "pattern":
      if (typeof value === "string" && !new RegExp(rule.value).test(value)) {
        return { isValid: false, message: "Invalid format" };
      }
      break;

    case "email":
      if (typeof value === "string" && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
        return { isValid: false, message: "Invalid email address" };
      }
      break;

    case "url":
      if (typeof value === "string") {
        try {
          new URL(value);
        } catch {
          return { isValid: false, message: "Invalid URL" };
        }
      }
      break;

    case "custom":
      return rule.validator(value);
  }

  return { isValid: true };
}

// Bulk validation for multiple entities
export function validateBulkUpdate(
  entities: Array<{ id: string; properties: Record<string, unknown> }>,
  schema: PropertySchema[]
): Map<string, Map<string, ValidationResult>> {
  const errors = new Map<string, Map<string, ValidationResult>>();

  for (const entity of entities) {
    const entityErrors = new Map<string, ValidationResult>();

    for (const field of schema) {
      const value = entity.properties[field.key];
      const result = validateProperty(value, field.validation);
      if (!result.isValid) {
        entityErrors.set(field.key, result);
      }
    }

    if (entityErrors.size > 0) {
      errors.set(entity.id, entityErrors);
    }
  }

  return errors;
}
```

### 6.3 Property Components

```typescript
// src/components/Properties/PropertyInput.tsx

import React, { useState, useCallback } from "react";
import { cn } from "@utils/cn";
import { X, Plus, Check } from "lucide-react";

interface PropertyInputProps {
  type: "text" | "number" | "select" | "boolean" | "tags" | "color" | "keyvalue";
  value: unknown;
  options?: { value: string; label: string }[];
  placeholder?: string;
  onChange: (value: unknown) => void;
  onBlur?: () => void;
  error?: string;
  disabled?: boolean;
}

// Text Input
export function TextInput({ value, placeholder, onChange, onBlur, error, disabled }: PropertyInputProps) {
  return (
    <input
      type="text"
      value={String(value ?? "")}
      placeholder={placeholder}
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
      disabled={disabled}
      className={cn(
        "w-full px-2 py-1.5 rounded bg-surface-light/5",
        "border border-border-dark focus:border-primary",
        "text-sm text-gray-200 placeholder-gray-500",
        "transition-colors",
        error && "border-red-500 focus:border-red-500"
      )}
    />
  );
}

// Number Input
export function NumberInput({ value, placeholder, onChange, onBlur, error, disabled }: PropertyInputProps) {
  return (
    <input
      type="number"
      value={Number(value) ?? ""}
      placeholder={placeholder}
      onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
      onBlur={onBlur}
      disabled={disabled}
      className={cn(
        "w-full px-2 py-1.5 rounded bg-surface-light/5",
        "border border-border-dark focus:border-primary",
        "text-sm text-gray-200 placeholder-gray-500",
        "transition-colors",
        error && "border-red-500 focus:border-red-500"
      )}
    />
  );
}

// Select Input
export function SelectInput({ value, options, onChange, error, disabled }: PropertyInputProps) {
  return (
    <select
      value={String(value ?? "")}
      onChange={(e) => onChange(e.target.value)}
      disabled={disabled}
      className={cn(
        "w-full px-2 py-1.5 rounded bg-surface-light/5",
        "border border-border-dark focus:border-primary",
        "text-sm text-gray-200",
        "transition-colors",
        error && "border-red-500 focus:border-red-500"
      )}
    >
      {options?.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}

// Boolean Toggle
export function BooleanInput({ value, onChange, disabled }: PropertyInputProps) {
  return (
    <button
      type="button"
      onClick={() => onChange(!value)}
      disabled={disabled}
      className={cn(
        "w-10 h-5 rounded-full transition-colors relative",
        value ? "bg-primary" : "bg-surface-light/20",
        "disabled:opacity-50"
      )}
    >
      <span
        className={cn(
          "absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform",
          value ? "left-5 translate-x-0" : "left-0.5"
        )}
      />
    </button>
  );
}

// Tags Input
export function TagsInput({ value, onChange, error, disabled }: PropertyInputProps) {
  const tags = Array.isArray(value) ? (value as string[]) : [];
  const [inputValue, setInputValue] = useState("");

  const addTag = useCallback(() => {
    if (inputValue.trim() && !tags.includes(inputValue.trim())) {
      onChange([...tags, inputValue.trim()]);
      setInputValue("");
    }
  }, [inputValue, tags, onChange]);

  const removeTag = useCallback((tag: string) => {
    onChange(tags.filter((t) => t !== tag));
  }, [tags, onChange]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      addTag();
    }
  };

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-1">
        {tags.map((tag) => (
          <span
            key={tag}
            className={cn(
              "px-2 py-0.5 rounded bg-primary/20 text-primary text-xs",
              "flex items-center gap-1"
            )}
          >
            {tag}
            {!disabled && (
              <button
                type="button"
                onClick={() => removeTag(tag)}
                className="hover:text-white"
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </span>
        ))}
      </div>
      {!disabled && (
        <div className="flex gap-2">
          <input
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add tag..."
            className={cn(
              "flex-1 px-2 py-1 rounded bg-surface-light/5",
              "border border-border-dark focus:border-primary",
              "text-sm text-gray-200 placeholder-gray-500"
            )}
          />
          <button
            type="button"
            onClick={addTag}
            className="px-2 py-1 rounded bg-primary/20 text-primary hover:bg-primary/30"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
      )}
      {error && <p className="text-xs text-red-500">{error}</p>}
    </div>
  );
}

// Key-Value Input
export function KeyValueInput({ value, onChange, error, disabled }: PropertyInputProps) {
  const pairs = Array.isArray(value) 
    ? (value as Array<{ key: string; value: string }>) 
    : [];

  const addPair = useCallback(() => {
    onChange([...pairs, { key: "", value: "" }]);
  }, [pairs, onChange]);

  const updatePair = useCallback((index: number, field: "key" | "value", val: string) => {
    const newPairs = [...pairs];
    newPairs[index] = { ...newPairs[index], [field]: val };
    onChange(newPairs);
  }, [pairs, onChange]);

  const removePair = useCallback((index: number) => {
    onChange(pairs.filter((_, i) => i !== index));
  }, [pairs, onChange]);

  return (
    <div className="space-y-2">
      {pairs.map((pair, index) => (
        <div key={index} className="flex gap-2">
          <input
            type="text"
            value={pair.key}
            onChange={(e) => updatePair(index, "key", e.target.value)}
            placeholder="Key"
            disabled={disabled}
            className={cn(
              "flex-1 px-2 py-1 rounded bg-surface-light/5",
              "border border-border-dark focus:border-primary",
              "text-sm text-gray-200"
            )}
          />
          <input
            type="text"
            value={pair.value}
            onChange={(e) => updatePair(index, "value", e.target.value)}
            placeholder="Value"
            disabled={disabled}
            className={cn(
              "flex-1 px-2 py-1 rounded bg-surface-light/5",
              "border border-border-dark focus:border-primary",
              "text-sm text-gray-200"
            )}
          />
          {!disabled && (
            <button
              type="button"
              onClick={() => removePair(index)}
              className="p-1 text-red-400 hover:text-red-300"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      ))}
      {!disabled && (
        <button
          type="button"
          onClick={addPair}
          className="text-xs text-primary hover:underline"
        >
          + Add environment variable
        </button>
      )}
      {error && <p className="text-xs text-red-500">{error}</p>}
    </div>
  );
}
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| Propiedades | Tiempo real | <16ms latency |
| Undo/redo | Eventos | 100% soportado |
| Bulk edit | Entidades | Sin límite |
| Validators | Cobertura | 100% campos |

## 📊 Estimación

| Fase | Esfuerzo | Estimación |
|------|----------|------------|
| Schemas | M | 4 horas |
| Validators | S | 2 horas |
| Property Components | M | 6 horas |
| Bulk Edit | M | 4 horas |
| Testing | S | 3 horas |
| **Total** | **M** | **~19 horas** |

## 📝 Notas

1. **Performance**: Usar `React.memo` para PropertyField para evitar re-renders
2. **Preview**: Considerar preview visual en canvas para cambios de color/size
3. **Schema Extensibility**: Permitir plugins para nuevos tipos de entidades

---

**Documento creado**: `docs/epics/EPIC-WEB-006-properties.md`
**Estado**: Listo para implementación
**Dependencia**: EPIC-WEB-003
