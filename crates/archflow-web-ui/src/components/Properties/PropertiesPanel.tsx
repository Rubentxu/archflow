/**
 * PropertiesPanel - Editor de propiedades para entidades
 *
 * Panel lateral derecho que permite editar las propiedades de las entidades
 * seleccionadas usando React Hook Form + Zod para validación.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import React, { useCallback, useEffect } from "react";
import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  X,
  RotateCcw,
  Database,
  Server,
  FunctionSquare,
  Globe,
  HardDrive,
  Zap,
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

/** Props para el panel de propiedades */
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
    icon: FunctionSquare,
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
    icon: Zap,
    label: "Container",
    schema: containerPropertiesSchema,
  },
  actor: {
    icon: Zap,
    label: "Actor",
    schema: containerPropertiesSchema,
  },
  database: {
    icon: Database,
    label: "Database",
    schema: rdsPropertiesSchema,
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
  const { selectedIds } = useSelectionStore();
  const { updateProperty, getEntity } = useEntityStore();
  const [isOpen, setIsOpen] = React.useState(true);
  const [isSaving, setIsSaving] = React.useState(false);

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
  }, [entity, form]);

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
  if (!entity) {
    return (
      <div
        className={cn(
          "w-80 h-full bg-surface-dark border-l border-white/5 flex flex-col items-center justify-center p-6",
          className,
        )}
      >
        <div className="text-center text-gray-500">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-white/5 flex items-center justify-center">
            <AlertCircle className="w-8 h-8" />
          </div>
          <p className="text-sm font-medium">No Entity Selected</p>
          <p className="text-xs mt-1 text-gray-600">
            Select an entity to view its properties
          </p>
        </div>
      </div>
    );
  }

  return (
    <motion.aside
      initial={{ width: 0, opacity: 0 }}
      animate={{ width: isOpen ? 320 : 0, opacity: isOpen ? 1 : 0 }}
      exit={{ width: 0, opacity: 0 }}
      className={cn(
        "h-full bg-surface-dark border-l border-white/5 flex flex-col overflow-hidden",
        className,
      )}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-white/5">
        <div className="flex items-center gap-2">
          <Icon className="w-5 h-5 text-primary" />
          <span className="text-sm font-medium text-gray-200">
            {config.label}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={handleReset}
            className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/5 transition-colors"
            title="Reset changes"
          >
            <RotateCcw className="w-4 h-4" />
          </button>
          <button
            onClick={() => setIsOpen(false)}
            className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-white/5 transition-colors"
            title="Close panel"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Form */}
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="flex-1 overflow-y-auto p-4"
      >
        <AnimatePresence mode="wait">
          <motion.div
            key={entityType}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            className="space-y-4"
          >
            {renderForm()}
          </motion.div>
        </AnimatePresence>
      </form>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-white/5">
        <button
          onClick={form.handleSubmit(onSubmit)}
          disabled={isSaving || !form.formState.isDirty}
          className={cn(
            "w-full flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all",
            "focus:outline-none focus:ring-2 focus:ring-primary/50",
            form.formState.isDirty
              ? "bg-primary text-white hover:bg-primary/90"
              : "bg-white/5 text-gray-500 cursor-not-allowed",
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
    </motion.aside>
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
