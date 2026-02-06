/**
 * Export Modal Component
 *
 * Modal for exporting the canvas to various formats:
 * - PNG (raster image)
 * - SVG (vector graphics)
 * - PDF (document format)
 * - JSON (data backup)
 *
 * Architecture Reference: docs/epics/EPIC-UI_COMPONENTS_SPECIFICATION.md - ExportModal
 */

import { useState, useCallback, memo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  X,
  Download,
  Image,
  FileType,
  FileText,
  FileJson,
  Loader2,
} from "lucide-react";
import { cn } from "../utils/cn";
import { useToastStore } from "../store/useToastStore";

/**
 * Default canvas dimensions
 */
const DEFAULT_CANVAS_WIDTH = 1920;
const DEFAULT_CANVAS_HEIGHT = 1080;

/**
 * Export format options
 */
export type ExportFormat = "png" | "svg" | "pdf" | "json";

/**
 * Export configuration
 */
interface ExportConfig {
  format: ExportFormat;
  scale: number;
  quality: number;
  background: "transparent" | "white" | "custom";
  customBackground?: string;
  selectionOnly: boolean;
  includeAnnotations: boolean;
}

/**
 * ExportModal props
 */
interface ExportModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback when modal should close */
  onClose: () => void;
  /** Callback when export is triggered */
  onExport?: (config: ExportConfig) => Promise<void>;
}

/**
 * Format descriptions and presets
 */
const FORMAT_PRESETS: Record<
  ExportFormat,
  { name: string; description: string; icon: typeof Image }
> = {
  png: {
    name: "PNG",
    description: "Raster image with transparency",
    icon: Image,
  },
  svg: {
    name: "SVG",
    description: "Vector graphics for scalable output",
    icon: FileType,
  },
  pdf: {
    name: "PDF",
    description: "Document format for printing",
    icon: FileText,
  },
  json: {
    name: "JSON",
    description: "Data backup for restoring later",
    icon: FileJson,
  },
};

/**
 * Scale options
 */
const SCALE_OPTIONS = [
  { value: 0.5, label: "0.5x" },
  { value: 1, label: "1x (Original)" },
  { value: 2, label: "2x (High DPI)" },
  { value: 3, label: "3x (Ultra HD)" },
  { value: 4, label: "4x (Print Quality)" },
];

/**
 * Quality options (for raster formats)
 */
const QUALITY_OPTIONS = [
  { value: 72, label: "72 DPI (Web)" },
  { value: 150, label: "150 DPI (Screen)" },
  { value: 300, label: "300 DPI (Print)" },
  { value: 600, label: "600 DPI (High Quality)" },
];

/**
 * Export Modal Component
 */
export const ExportModal = memo(function ExportModal({
  isOpen,
  onClose,
  onExport,
}: ExportModalProps) {
  // Export configuration state
  const [config, setConfig] = useState<ExportConfig>({
    format: "png",
    scale: 2,
    quality: 150,
    background: "transparent",
    customBackground: "#ffffff",
    selectionOnly: false,
    includeAnnotations: true,
  });

  // Export state
  const [isExporting, setIsExporting] = useState(false);
  const [exportProgress, setExportProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // Toast store for notifications
  const addToast = useToastStore((state) => state.addToast);

  // Handle format change
  const handleFormatChange = useCallback((format: ExportFormat) => {
    setConfig((prev) => ({ ...prev, format }));
  }, []);

  // Handle scale change
  const handleScaleChange = useCallback((scale: number) => {
    setConfig((prev) => ({ ...prev, scale }));
  }, []);

  // Handle quality change
  const handleQualityChange = useCallback((quality: number) => {
    setConfig((prev) => ({ ...prev, quality }));
  }, []);

  // Handle background change
  const handleBackgroundChange = useCallback(
    (background: "transparent" | "white" | "custom") => {
      setConfig((prev) => ({ ...prev, background }));
    },
    [],
  );

  // Handle toggle
  const handleToggle = useCallback((key: keyof ExportConfig) => {
    setConfig((prev) => ({
      ...prev,
      [key]: !prev[key],
    }));
  }, []);

  // Handle export
  const handleExport = useCallback(async () => {
    if (!onExport || isExporting) return;

    setIsExporting(true);
    setExportProgress(0);
    setError(null);

    try {
      // Simulate progress (actual implementation would use real progress)
      setExportProgress(20);
      await onExport(config);
      setExportProgress(100);

      addToast({
        type: "success",
        message: `Successfully exported as ${FORMAT_PRESETS[config.format].name}`,
      });

      onClose();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Export failed";
      setError(errorMessage);
      addToast({
        type: "error",
        message: errorMessage,
      });
    } finally {
      setIsExporting(false);
      setExportProgress(0);
    }
  }, [config, onClose]);

  // Handle keyboard escape
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape" && !isExporting) {
        onClose();
      }
    },
    [isExporting, onClose],
  );

  if (!isOpen) return null;

  return (
    <AnimatePresence>
      {/* Backdrop */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ duration: 0.2 }}
        className="fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white p-6 shadow-2xl dark:bg-gray-900"
        onKeyDown={handleKeyDown}
      >
        {/* Header */}
        <div className="mb-6 flex items-center justify-between">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Export Diagram
          </h2>
          <button
            onClick={onClose}
            disabled={isExporting}
            className="rounded-lg p-2 text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="space-y-6">
          {/* Format Selection */}
          <div>
            <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Format
            </label>
            <div className="grid grid-cols-4 gap-2">
              {(Object.keys(FORMAT_PRESETS) as ExportFormat[]).map((format) => {
                const preset = FORMAT_PRESETS[format];
                const Icon = preset.icon;
                const isSelected = config.format === format;

                return (
                  <button
                    key={format}
                    onClick={() => handleFormatChange(format)}
                    disabled={isExporting}
                    className={cn(
                      "flex flex-col items-center gap-2 rounded-lg border-2 p-3 transition-colors",
                      isSelected
                        ? "border-blue-500 bg-blue-50 dark:bg-blue-900/30"
                        : "border-gray-200 hover:border-gray-300 dark:border-gray-700",
                      isExporting && "cursor-not-allowed opacity-50",
                    )}
                  >
                    <Icon
                      size={24}
                      className={cn(
                        isSelected ? "text-blue-500" : "text-gray-500",
                      )}
                    />
                    <span className="text-sm font-medium">{preset.name}</span>
                  </button>
                );
              })}
            </div>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              {FORMAT_PRESETS[config.format].description}
            </p>
          </div>

          {/* Scale (for raster formats) */}
          {config.format === "png" && (
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                Scale
              </label>
              <div className="flex gap-2">
                {SCALE_OPTIONS.map((option) => (
                  <button
                    key={option.value}
                    onClick={() => handleScaleChange(option.value)}
                    disabled={isExporting}
                    className={cn(
                      "rounded-lg border px-3 py-2 text-sm transition-colors",
                      config.scale === option.value
                        ? "border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-900/30"
                        : "border-gray-200 hover:border-gray-300 dark:border-gray-700",
                    )}
                  >
                    {option.label}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Quality (for PNG) */}
          {config.format === "png" && (
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                Quality
              </label>
              <select
                value={config.quality}
                onChange={(e) => handleQualityChange(Number(e.target.value))}
                disabled={isExporting}
                className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm dark:border-gray-700"
              >
                {QUALITY_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          )}

          {/* Background */}
          <div>
            <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Background
            </label>
            <div className="flex gap-2">
              {[
                { value: "transparent", label: "Transparent" },
                { value: "white", label: "White" },
              ].map((option) => (
                <button
                  key={option.value}
                  onClick={() =>
                    handleBackgroundChange(
                      option.value as "transparent" | "white",
                    )
                  }
                  disabled={isExporting}
                  className={cn(
                    "flex-1 rounded-lg border px-3 py-2 text-sm transition-colors",
                    config.background === option.value
                      ? "border-blue-500 bg-blue-50 text-blue-600 dark:bg-blue-900/30"
                      : "border-gray-200 hover:border-gray-300 dark:border-gray-700",
                  )}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          {/* Options */}
          <div className="space-y-3">
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={config.selectionOnly}
                onChange={() => handleToggle("selectionOnly")}
                disabled={isExporting}
                className="h-4 w-4 rounded border-gray-300 text-blue-600"
              />
              <span className="text-sm text-gray-700 dark:text-gray-300">
                Export selection only
              </span>
            </label>
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={config.includeAnnotations}
                onChange={() => handleToggle("includeAnnotations")}
                disabled={isExporting}
                className="h-4 w-4 rounded border-gray-300 text-blue-600"
              />
              <span className="text-sm text-gray-700 dark:text-gray-300">
                Include annotations
              </span>
            </label>
          </div>

          {/* Preview Info */}
          <div className="rounded-lg bg-gray-50 p-3 dark:bg-gray-800">
            <div className="text-sm text-gray-600 dark:text-gray-400">
              <p>
                Output size: {(DEFAULT_CANVAS_WIDTH * config.scale).toFixed(0)}{" "}
                × {(DEFAULT_CANVAS_HEIGHT * config.scale).toFixed(0)}px
              </p>
            </div>
          </div>

          {/* Error */}
          {error && (
            <div className="rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20">
              {error}
            </div>
          )}

          {/* Progress */}
          {isExporting && (
            <div className="space-y-2">
              <div className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                <Loader2 size={16} className="animate-spin" />
                <span>Exporting...</span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
                <motion.div
                  className="h-full bg-blue-500"
                  initial={{ width: 0 }}
                  animate={{ width: `${exportProgress}%` }}
                  transition={{ duration: 0.3 }}
                />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="mt-6 flex justify-end gap-3">
          <button
            onClick={onClose}
            disabled={isExporting}
            className="rounded-lg px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800"
          >
            Cancel
          </button>
          <button
            onClick={handleExport}
            disabled={isExporting}
            className={cn(
              "flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors",
              isExporting
                ? "cursor-not-allowed bg-gray-400"
                : "bg-blue-600 hover:bg-blue-700",
            )}
          >
            {isExporting ? (
              <>
                <Loader2 size={16} className="animate-spin" />
                Exporting...
              </>
            ) : (
              <>
                <Download size={16} />
                Export
              </>
            )}
          </button>
        </div>
      </motion.div>
    </AnimatePresence>
  );
});
