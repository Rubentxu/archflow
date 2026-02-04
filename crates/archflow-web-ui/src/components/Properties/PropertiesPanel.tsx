/**
 * PropertiesPanel - Entity Properties Editor
 *
 * Right sidebar allowing property editing for selected entities.
 * Includes Identity header and Motion/Particles footer.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import React, { useCallback, useEffect } from "react";
import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Database,
  Server,
  Code2,
  Globe,
  HardDrive,
  MoreHorizontal,
  Activity,
  Box,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { cn } from "../../utils/cn";
import { useSelectionStore } from "../../store/useSelectionStore";
import { useEntityStore } from "../../hooks/useEntityStore";
import {
  ec2PropertiesSchema,
  lambdaPropertiesSchema,
  rdsPropertiesSchema,
  s3PropertiesSchema,
  apiGatewayPropertiesSchema,
  containerPropertiesSchema,
  validateEntityProperties,
  type EntityPropertySchema,
} from "../../types/entity-schemas";

import { useArchFlowWasm } from "../../hooks/useArchFlowWasm";
import { useUIStore } from "../../store/useUIStore";
import { LogicPanel } from "./LogicPanel";
import { HistoryPanel } from "./HistoryPanel";
import { ShapeHistory } from "./ShapeHistory";

/**
 * Visual Properties Form
 */
function VisualPropertiesForm({
  entityId,
  bridge,
  onUpdate
}: {
  entityId: number | null,
  bridge: any,
  onUpdate?: () => void
}) {
  // Local state for immediate feedback
  const [fillColor, setFillColor] = React.useState("#3b82f6");
  const [strokeColor, setStrokeColor] = React.useState("#000000");
  const [strokeWidth, setStrokeWidth] = React.useState(2.0);

  const handleFillChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setFillColor(val);
    if (!bridge) return;

    const r = parseInt(val.slice(1, 3), 16);
    const g = parseInt(val.slice(3, 5), 16);
    const b = parseInt(val.slice(5, 7), 16);

    if (entityId !== null) {
      bridge.set_color(entityId, r, g, b, 255);
    } else {
      bridge.set_active_color(r, g, b, 255);
    }
    onUpdate?.();
  };

  const handleStrokeColorChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setStrokeColor(val);
    if (!bridge) return;

    const r = parseInt(val.slice(1, 3), 16);
    const g = parseInt(val.slice(3, 5), 16);
    const b = parseInt(val.slice(5, 7), 16);

    if (entityId !== null) {
      bridge.set_stroke_color(entityId, r, g, b, 255);
    } else {
      bridge.set_active_stroke_color(r, g, b, 255);
    }
    onUpdate?.();
  };

  const handleStrokeWidthChange = (val: number) => {
    setStrokeWidth(val);
    if (!bridge) return;

    if (entityId !== null) {
      bridge.set_stroke_width(entityId, val);
    } else {
      bridge.set_active_stroke_width(val);
    }
    onUpdate?.();
  };

  return (
    <div className="space-y-4 mb-6 pt-2 pb-4 border-b border-border-light dark:border-border-dark/50">
      <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-3">Appearance</h4>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-1.5">
          <label className="text-xs text-gray-400 font-medium">Fill</label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={fillColor}
              onChange={handleFillChange}
              className="h-8 w-12 rounded bg-transparent cursor-pointer"
            />
            <span className="text-xs text-gray-300 font-mono">{fillColor}</span>
          </div>
        </div>

        <div className="space-y-1.5">
          <label className="text-xs text-gray-400 font-medium">Stroke</label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={strokeColor}
              onChange={handleStrokeColorChange}
              className="h-8 w-12 rounded bg-transparent cursor-pointer"
            />
          </div>
        </div>

        <div className="col-span-2 space-y-1.5">
          <label className="text-xs text-gray-400 font-medium">Stroke Width</label>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="0" max="20" step="0.5"
              value={strokeWidth}
              onChange={(e) => handleStrokeWidthChange(parseFloat(e.target.value))}
              className="flex-1 accent-primary h-1 bg-gray-700 rounded-lg appearance-none cursor-pointer"
            />
            <span className="text-xs w-8 text-right font-mono text-gray-300">{strokeWidth}px</span>
          </div>
        </div>
      </div>
    </div>
  );
}

interface PropertiesPanelProps {
  className?: string;
}

/** Tipo de entidad con su icono */
const entityTypeConfig: Record<
  string,
  {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    schema: EntityPropertySchema;
  }
> = {
  "aws-ec2": {
    icon: Server,
    label: "EC2 Instance",
    schema: ec2PropertiesSchema,
  },
  "aws-lambda": {
    icon: Code2,
    label: "Lambda Function",
    schema: lambdaPropertiesSchema,
  },
  "aws-rds": {
    icon: Database,
    label: "RDS Database",
    schema: rdsPropertiesSchema,
  },
  "aws-s3": {
    icon: HardDrive,
    label: "S3 Bucket",
    schema: s3PropertiesSchema,
  },
  "aws-api-gateway": {
    icon: Globe,
    label: "API Gateway",
    schema: apiGatewayPropertiesSchema,
  },
  container: {
    icon: Box,
    label: "Container",
    schema: containerPropertiesSchema,
  },
};

/**
 * Input component with label and error display
 */
interface FormInputProps {
  label: string;
  description?: string;
  error?: string;
  required?: boolean;
  children: React.ReactNode;
  fullWidth?: boolean;
}

function FormInput({
  label,
  description,
  error,
  required,
  children,
  fullWidth = false,
}: FormInputProps) {
  return (
    <div className={cn("flex flex-col gap-1.5", fullWidth && "col-span-2")}>
      <label className="flex items-center gap-1.5 text-sm font-medium text-gray-300">
        {label}
        {required && <span className="text-red-400">*</span>}
      </label>
      {children}
      {description && (
        <span className="text-xs text-gray-500">{description}</span>
      )}
      {error && (
        <motion.span
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
          className="text-xs text-red-400 flex items-center gap-1"
        >
          <AlertCircle className="w-3 h-3" />
          {error}
        </motion.span>
      )}
    </div>
  );
}

/**
 * Text input component
 */
interface TextInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  error?: string;
}

function TextInput({ value, onChange, placeholder, error }: TextInputProps) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      className={cn(
        "px-3 py-2 bg-surface-dark border rounded-lg text-sm text-gray-200",
        "placeholder:text-gray-500",
        "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50",
        "transition-all duration-150",
        error && "border-red-500 focus:ring-red-500/50",
      )}
    />
  );
}

/**
 * Number input component with min/max
 */
interface NumberInputProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  placeholder?: string;
  error?: string;
}

function NumberInput({
  value,
  onChange,
  min,
  max,
  step = 1,
  placeholder,
  error,
}: NumberInputProps) {
  return (
    <div className="relative">
      <input
        type="number"
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        min={min}
        max={max}
        step={step}
        placeholder={placeholder}
        className={cn(
          "w-full px-3 py-2 bg-surface-dark border rounded-lg text-sm text-gray-200",
          "placeholder:text-gray-500",
          "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50",
          "transition-all duration-150",
          error && "border-red-500 focus:ring-red-500/50",
        )}
      />
      {error && (
        <span className="absolute -bottom-4 left-0 text-xs text-red-400">
          {error}
        </span>
      )}
    </div>
  );
}

/**
 * Select input component
 */
interface SelectInputProps {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  error?: string;
  placeholder?: string;
}

function SelectInput({
  value,
  onChange,
  options,
  error,
  placeholder,
}: SelectInputProps) {
  return (
    <div className="relative">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={cn(
          "w-full px-3 py-2 bg-surface-dark border rounded-lg text-sm text-gray-200",
          "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary/50",
          "transition-all duration-150 appearance-none cursor-pointer",
          "bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2216%22%20height%3D%2216%22%20fill%3D%22%23CBD5E1%22%20viewBox%3D%220%200%2016%2016%22%3E%3Cpath%20d%3D%22M4%206l4%204%204-4%22%2F%3E%3C%2Fsvg%3E')] bg-no-repeat bg-[right_0.75rem_center]",
          error && "border-red-500 focus:ring-red-500/50",
        )}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {error && (
        <span className="absolute -bottom-4 left-0 text-xs text-red-400">
          {error}
        </span>
      )}
    </div>
  );
}

/**
 * EC2 Instance form component
 */
function EC2PropertiesForm({ form }: { form: ReturnType<typeof useForm> }) {
  const {
    control,
    formState: { errors },
  } = form;

  const instanceTypes = [
    { value: "t3.micro", label: "t3.micro (2 vCPU, 1 GiB)" },
    { value: "t3.small", label: "t3.small (2 vCPU, 2 GiB)" },
    { value: "t3.medium", label: "t3.medium (2 vCPU, 4 GiB)" },
    { value: "t3.large", label: "t3.large (2 vCPU, 8 GiB)" },
    { value: "m6i.large", label: "m6i.large (2 vCPU, 8 GiB)" },
    { value: "c6i.large", label: "c6i.large (2 vCPU, 4 GiB)" },
  ];

  const regions = [
    { value: "us-east-1", label: "US East (N. Virginia)" },
    { value: "us-west-2", label: "US West (Oregon)" },
    { value: "eu-west-1", label: "EU (Ireland)" },
    { value: "eu-central-1", label: "EU (Frankfurt)" },
    { value: "ap-southeast-1", label: "Asia Pacific (Singapore)" },
    { value: "ap-northeast-1", label: "Asia Pacific (Tokyo)" },
  ];

  return (
    <div className="grid grid-cols-2 gap-4">
      <FormInput
        label="Instance Name"
        required
        error={errors.name?.message as string}
      >
        <Controller
          name="name"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="my-instance"
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Instance Type"
        required
        error={errors.instanceType?.message as string}
      >
        <Controller
          name="instanceType"
          control={control}
          render={({ field }) => (
            <SelectInput
              value={field.value || ""}
              onChange={field.onChange}
              options={instanceTypes}
              placeholder="Select type..."
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Region"
        required
        error={errors.region?.message as string}
      >
        <Controller
          name="region"
          control={control}
          render={({ field }) => (
            <SelectInput
              value={field.value || ""}
              onChange={field.onChange}
              options={regions}
              placeholder="Select region..."
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Availability Zone"
        error={errors.availabilityZone?.message as string}
      >
        <Controller
          name="availabilityZone"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="us-east-1a"
            />
          )}
        />
      </FormInput>

      <div className="col-span-2">
        <Controller
          name="description"
          control={control}
          render={({ field }) => (
            <FormInput
              label="Description"
              error={errors.description?.message as string}
              fullWidth
            >
              <textarea
                {...field}
                value={field.value || ""}
                onChange={(e) => field.onChange(e.target.value)}
                placeholder="Optional description..."
                rows={2}
                className="w-full px-3 py-2 bg-surface-dark border rounded-lg text-sm text-gray-200 resize-none placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
              />
            </FormInput>
          )}
        />
      </div>
    </div>
  );
}

/**
 * Lambda Function form component
 */
function LambdaPropertiesForm({ form }: { form: ReturnType<typeof useForm> }) {
  const {
    control,
    formState: { errors },
  } = form;

  const runtimes = [
    { value: "nodejs20.x", label: "Node.js 20.x" },
    { value: "nodejs18.x", label: "Node.js 18.x" },
    { value: "python3.12", label: "Python 3.12" },
    { value: "python3.11", label: "Python 3.11" },
    { value: "java17", label: "Java 17" },
    { value: "dotnet8", label: ".NET 8" },
  ];

  const architectures = [
    { value: "x86_64", label: "x86_64" },
    { value: "arm64", label: "ARM64 (Graviton)" },
  ];

  return (
    <div className="grid grid-cols-2 gap-4">
      <FormInput
        label="Function Name"
        required
        error={errors.name?.message as string}
      >
        <Controller
          name="name"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="my-function"
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Runtime"
        required
        error={errors.runtime?.message as string}
      >
        <Controller
          name="runtime"
          control={control}
          render={({ field }) => (
            <SelectInput
              value={field.value || ""}
              onChange={field.onChange}
              options={runtimes}
              placeholder="Select runtime..."
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Handler"
        required
        error={errors.handler?.message as string}
      >
        <Controller
          name="handler"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="index.handler"
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Architecture"
        error={errors.architecture?.message as string}
      >
        <Controller
          name="architecture"
          control={control}
          render={({ field }) => (
            <SelectInput
              value={field.value || "x86_64"}
              onChange={field.onChange}
              options={architectures}
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Timeout (seconds)"
        error={errors.timeout?.message as string}
      >
        <Controller
          name="timeout"
          control={control}
          render={({ field }) => (
            <NumberInput
              value={field.value || 3}
              onChange={field.onChange}
              min={1}
              max={900}
            />
          )}
        />
      </FormInput>

      <FormInput label="Memory (MB)" error={errors.memory?.message as string}>
        <Controller
          name="memory"
          control={control}
          render={({ field }) => (
            <NumberInput
              value={field.value || 128}
              onChange={field.onChange}
              min={128}
              max={10240}
              step={128}
            />
          )}
        />
      </FormInput>
    </div>
  );
}

/**
 * Container/Generic form component
 */
function ContainerPropertiesForm({
  form,
}: {
  form: ReturnType<typeof useForm>;
}) {
  const {
    control,
    formState: { errors },
  } = form;

  return (
    <div className="grid grid-cols-2 gap-4">
      <FormInput
        label="Name"
        required
        error={errors.name?.message as string}
        fullWidth
      >
        <Controller
          name="name"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="Entity name"
            />
          )}
        />
      </FormInput>

      <FormInput
        label="Label"
        error={errors.label?.message as string}
        fullWidth
      >
        <Controller
          name="label"
          control={control}
          render={({ field }) => (
            <TextInput
              value={field.value || ""}
              onChange={field.onChange}
              placeholder="Display label"
            />
          )}
        />
      </FormInput>

      <div className="col-span-2">
        <Controller
          name="description"
          control={control}
          render={({ field }) => (
            <FormInput
              label="Description"
              error={errors.description?.message as string}
              fullWidth
            >
              <textarea
                {...field}
                value={field.value || ""}
                onChange={(e) => field.onChange(e.target.value)}
                placeholder="Optional description..."
                rows={3}
                className="w-full px-3 py-2 bg-surface-dark border rounded-lg text-sm text-gray-200 resize-none placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
              />
            </FormInput>
          )}
        />
      </div>
    </div>
  );
}

/**
 * Main PropertiesPanel component
 */
export function PropertiesPanel({ className }: PropertiesPanelProps) {
  const { bridge } = useArchFlowWasm();
  const { activeTool } = useUIStore();
  const { selectedIds } = useSelectionStore();
  const { updateProperty, getEntity } = useEntityStore();
  const [isSaving, setIsSaving] = React.useState(false);
  const [activeTab, setActiveTab] = React.useState<"properties" | "logic" | "history">("properties");

  // Motion & Particles state
  const [throughput, setThroughput] = React.useState(500);
  const [packetSize, setPacketSize] = React.useState(24);

  // Get selected entity
  const selectedId = selectedIds[0] ?? null;
  const entity = selectedId ? getEntity(selectedId) : null;
  const entityType = (entity?.type as string) || "container";

  // Get config for entity type
  const config = entityTypeConfig[entityType] || entityTypeConfig.container;
  const Icon = config.icon;
  const schema = config.schema;

  // Initialize form
  const form = useForm({
    resolver: zodResolver(schema),
    defaultValues: entity?.properties || {},
    mode: "onChange",
  });

  // Reset form when entity changes
  useEffect(() => {
    if (entity?.properties) {
      form.reset(entity.properties);
    }
  }, [entity, form, selectedId]);

  // Handle form submission
  const onSubmit = useCallback(
    async (data: Record<string, unknown>) => {
      if (!selectedId) return;

      setIsSaving(true);
      try {
        // Validate before saving
        const validation = validateEntityProperties(entityType, data);
        if (!validation.success) {
          // Set form errors
          if (validation.errors) {
            Object.entries(validation.errors).forEach(([path, message]) => {
              form.setError(path as any, { message: message as string });
            });
          }
          return;
        }

        // Update each property
        for (const [key, value] of Object.entries(data)) {
          updateProperty(selectedId, key, value);
        }

        // Show success feedback (could integrate with toast)
      } finally {
        setIsSaving(false);
      }
    },
    [selectedId, entityType, updateProperty, form],
  );

  // Reset form to original values
  const handleReset = useCallback(() => {
    if (entity?.properties) {
      form.reset(entity.properties);
    }
  }, [entity, form]);

  // Render appropriate form based on entity type
  const renderForm = () => {
    switch (entityType) {
      case "aws-ec2":
        return <EC2PropertiesForm form={form} />;
      case "aws-lambda":
        return <LambdaPropertiesForm form={form} />;
      default:
        return <ContainerPropertiesForm form={form} />;
    }
  };

  // Empty state when no entity is selected
  if (!entity || !selectedId) {
    if (activeTool !== 'select') {
      return (
        <aside className={cn(
          "w-80 bg-surface-light dark:bg-surface-dark border-l border-border-light dark:border-border-dark flex flex-col items-stretch transition-all duration-300",
          className
        )}>
          <div className="flex-1 w-full space-y-4 overflow-y-auto p-6">
            <div className="flex items-center gap-3 pb-4 border-b border-border-light dark:border-border-dark">
              <div className="size-10 rounded bg-primary/10 flex items-center justify-center text-primary shrink-0">
                <Box className="w-5 h-5" />
              </div>
              <div className="overflow-hidden">
                <h4 className="font-bold text-sm text-slate-900 dark:text-white truncate">
                  {activeTool.charAt(0).toUpperCase() + activeTool.slice(1)} Tool
                </h4>
                <p className="text-xs text-slate-500">
                  Default Properties
                </p>
              </div>
            </div>

            <VisualPropertiesForm
              entityId={null}
              bridge={bridge}
            />
          </div>
          <ShapeHistory />
        </aside>
      );
    }

    return (
      <aside className={cn(
        "w-80 bg-surface-light dark:bg-surface-dark border-l border-border-light dark:border-border-dark flex flex-col transition-all duration-300",
        className
      )}>
        <div className="flex-1 flex flex-col items-center justify-center p-6 text-center text-gray-500 dark:text-gray-400">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-slate-100 dark:bg-white/5 flex items-center justify-center">
            <AlertCircle className="w-8 h-8" />
          </div>
          <p className="text-sm font-medium text-slate-900 dark:text-white mb-2">
            No Entity Selected
          </p>
          <p className="text-xs text-gray-600 dark:text-gray-500">
            Select an entity to view its properties
          </p>
        </div>
        <ShapeHistory />
      </aside>
    );
  }

  return (
    <aside
      className={cn(
        "w-80 bg-surface-light dark:bg-surface-dark border-l border-border-light dark:border-border-dark flex flex-col shrink-0 z-20 shadow-sm transition-all duration-300",
        className,
      )}
    >
      {/* Header with Tabs */}
      <div className="flex items-center border-b border-border-light dark:border-border-dark bg-slate-50 dark:bg-black/20">
        <button
          onClick={() => setActiveTab("properties")}
          className={cn(
            "flex-1 py-3 text-xs font-bold uppercase tracking-wider transition-colors border-b-2",
            activeTab === "properties"
              ? "border-primary text-primary bg-white dark:bg-transparent"
              : "border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
          )}
        >
          Properties
        </button>
        <button
          onClick={() => setActiveTab("logic")}
          className={cn(
            "flex-1 py-3 text-xs font-bold uppercase tracking-wider transition-colors border-b-2",
            activeTab === "logic"
              ? "border-primary text-primary bg-white dark:bg-transparent"
              : "border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
          )}
        >
          Logic
        </button>
        <button
          onClick={() => setActiveTab("history")}
          className={cn(
            "flex-1 py-3 text-xs font-bold uppercase tracking-wider transition-colors border-b-2",
            activeTab === "history"
              ? "border-primary text-primary bg-white dark:bg-transparent"
              : "border-transparent text-slate-500 hover:text-slate-700 dark:hover:text-slate-300"
          )}
        >
          History
        </button>
      </div>

      {/* Main Content Area */}
      <div className="flex-1 overflow-y-auto relative flex flex-col">
        {/* PROPERTIES TAB */}
        {activeTab === "properties" && (
          <div className="flex flex-col h-full">
            <div className="p-4 border-b border-border-light dark:border-border-dark bg-slate-50/50 dark:bg-slate-900/20 flex justify-between items-center">
              <h3 className="font-bold text-sm text-slate-800 dark:text-white uppercase tracking-wider">
                PROPERTIES
              </h3>
              <button className="text-slate-400 hover:text-primary transition-colors">
                <MoreHorizontal className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-6">
              <VisualPropertiesForm
                entityId={selectedId}
                bridge={bridge}
              />

              {/* Identity Section */}
              <div className="space-y-3">
                <div className="flex items-center gap-3">
                  <div className="size-10 rounded bg-orange-100 dark:bg-orange-900/30 flex items-center justify-center text-orange-600 shrink-0">
                    <Icon className="w-5 h-5" />
                  </div>
                  <div className="overflow-hidden">
                    <h4 className="font-bold text-sm text-slate-900 dark:text-white truncate">
                      {entity.label || config.label}
                    </h4>
                    <p className="text-xs text-slate-500 font-mono">
                      {selectedId.toString().substring(0, 12)}
                    </p>
                  </div>
                </div>
              </div>

              {/* Form */}
              <form
                onSubmit={form.handleSubmit(onSubmit)}
                className="space-y-4"
              >
                {renderForm()}
              </form>

              {/* Apply Changes Button - Inside scrollable area */}
              <button
                onClick={form.handleSubmit(onSubmit)}
                disabled={isSaving || !form.formState.isDirty}
                className={cn(
                  "w-full flex items-center justify-center gap-2 px-4 py-2 mt-4 rounded-lg text-sm font-medium transition-all",
                  "focus:outline-none focus:ring-2 focus:ring-primary/50",
                  form.formState.isDirty
                    ? "bg-primary text-white hover:bg-primary/90 shadow-md shadow-primary/20"
                    : "bg-slate-100 dark:bg-white/5 text-gray-400 dark:text-gray-500 cursor-not-allowed",
                )}
              >
                {isSaving ? (
                  <>
                    <motion.div
                      animate={{ rotate: 360 }}
                      transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
                      className="w-4 h-4 border-2 border-current border-t-transparent rounded-full"
                    />
                    <span>Saving...</span>
                  </>
                ) : (
                  <>
                    <CheckCircle2 className="w-4 h-4" />
                    <span>Apply Changes</span>
                  </>
                )}
              </button>
            </div>

            {/* Motion & Particles Footer */}
            <div className="bg-background-light dark:bg-[#152329] flex flex-col border-t-4 border-double border-border-light dark:border-border-dark shrink-0">
              <div className="p-3 border-b border-border-light dark:border-border-dark flex justify-between items-center bg-surface-light dark:bg-surface-dark">
                <div className="flex items-center gap-2">
                  <Activity className="w-5 h-5 text-primary" />
                  <h3 className="font-bold text-sm text-slate-800 dark:text-white">
                    Motion & Particles
                  </h3>
                </div>
                <div className="flex items-center gap-1">
                  <span className="size-2 bg-primary rounded-full animate-pulse"></span>
                  <span className="text-[10px] font-bold text-primary tracking-wider">LIVE</span>
                </div>
              </div>

              <div className="p-4 space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between text-xs">
                    <span className="text-gray-500">Throughput Speed</span>
                    <span className="text-primary font-mono">{throughput} req/s</span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="1000"
                    value={throughput}
                    onChange={(e) => setThroughput(Number(e.target.value))}
                    className="w-full h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3.5 [&::-webkit-slider-thumb]:h-3.5 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary"
                  />
                </div>

                <div className="space-y-2">
                  <div className="flex justify-between text-xs">
                    <span className="text-gray-500">Packet Size</span>
                    <span className="text-gray-200 font-mono">{packetSize}kb</span>
                  </div>
                  <input
                    type="range"
                    min="1"
                    max="100"
                    value={packetSize}
                    onChange={(e) => setPacketSize(Number(e.target.value))}
                    className="w-full h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3.5 [&::-webkit-slider-thumb]:h-3.5 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-slate-400"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1.5">
                    <span className="text-[10px] font-bold text-slate-400 uppercase">Flow Color</span>
                    <div className="flex gap-1.5">
                      <button className="w-5 h-5 rounded-full bg-primary ring-2 ring-offset-1 ring-offset-[#0d1117] ring-primary"></button>
                      <button className="w-5 h-5 rounded-full bg-orange-500 hover:ring-2 ring-orange-500/50"></button>
                      <button className="w-5 h-5 rounded-full bg-green-500 hover:ring-2 ring-green-500/50"></button>
                      <button className="w-5 h-5 rounded-full bg-purple-500 hover:ring-2 ring-purple-500/50"></button>
                    </div>
                  </div>
                  <div className="space-y-1.5">
                    <span className="text-[10px] font-bold text-slate-400 uppercase">Effect Style</span>
                    <select className="w-full text-xs bg-surface-dark border border-white/10 rounded px-2 py-1 text-gray-300 focus:outline-none">
                      <option>Pulse</option>
                      <option>Stream</option>
                      <option>Particles</option>
                    </select>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* LOGIC TAB */}
        {activeTab === "logic" && <LogicPanel entityId={selectedId} />}

        {/* HISTORY TAB */}
        {activeTab === "history" && <HistoryPanel entityId={selectedId} />}
      </div>
    </aside>
  );
}

/**
 * Collapsible toggle for the panel
 */
export function PropertiesPanelToggle({
  onClick,
  isOpen,
}: {
  onClick: () => void;
  isOpen: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "p-2 rounded-lg transition-all",
        isOpen
          ? "bg-primary/10 text-primary"
          : "text-gray-400 hover:text-white hover:bg-white/5",
      )}
      title="Toggle Properties Panel"
    >
      <Database className="w-5 h-5" />
    </button>
  );
}
