/**
 * ArchFlow Tools Bridge - Tool Management Facade
 *
 * Provides organized access to tool-related operations in the WASM bridge.
 * Wraps the flat WasmBridge API with a domain-specific interface.
 *
 * Architecture Reference: ARCHITECTURE-CLEAN-BRIDGE.md
 */

import type { WasmBridge } from "../wasm/archflow_web.d";

/**
 * Available tool types in ArchFlow
 */
export type ToolType =
  | "select"
  | "pan"
  | "rectangle"
  | "circle"
  | "ellipse"
  | "path"
  | "text"
  | "connector"
  | "image"
  | "hand";

/**
 * Tool metadata for UI rendering
 */
export interface ToolInfo {
  /** Tool identifier */
  id: ToolType;
  /** Display name */
  name: string;
  /** Keyboard shortcut */
  shortcut?: string;
  /** Icon identifier */
  icon: string;
  /** Category for tool palette */
  category: "navigation" | "shapes" | "drawing" | "special";
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOOL REGISTRY & DEFINITIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Registry of available tools with metadata
 */
export const TOOL_REGISTRY: ToolInfo[] = [
  {
    id: "select",
    name: "Select",
    shortcut: "V",
    icon: "cursor",
    category: "navigation",
  },
  {
    id: "pan",
    name: "Pan",
    shortcut: "H",
    icon: "hand",
    category: "navigation",
  },
  {
    id: "hand",
    name: "Hand",
    shortcut: "Space",
    icon: "hand",
    category: "navigation",
  },
  {
    id: "rectangle",
    name: "Rectangle",
    shortcut: "R",
    icon: "square",
    category: "shapes",
  },
  {
    id: "circle",
    name: "Circle",
    shortcut: "C",
    icon: "circle",
    category: "shapes",
  },
  {
    id: "ellipse",
    name: "Ellipse",
    shortcut: "O",
    icon: "ellipse",
    category: "shapes",
  },
  {
    id: "path",
    name: "Path",
    shortcut: "P",
    icon: "pen",
    category: "drawing",
  },
  {
    id: "text",
    name: "Text",
    shortcut: "T",
    icon: "text",
    category: "special",
  },
  {
    id: "connector",
    name: "Connector",
    shortcut: "E",
    icon: "line",
    category: "special",
  },
  {
    id: "image",
    name: "Image",
    shortcut: "I",
    icon: "image",
    category: "special",
  },
];

/**
 * Default tool when initializing
 */
export const DEFAULT_TOOL: ToolType = "select";

/**
 * Tool category mapping for UI organization
 */
export const TOOLS_BY_CATEGORY: Record<ToolInfo["category"], ToolInfo[]> = {
  navigation: TOOL_REGISTRY.filter((t) => t.category === "navigation"),
  shapes: TOOL_REGISTRY.filter((t) => t.category === "shapes"),
  drawing: TOOL_REGISTRY.filter((t) => t.category === "drawing"),
  special: TOOL_REGISTRY.filter((t) => t.category === "special"),
};

type ToolCategory = { category: ToolInfo["category"] };

// ═══════════════════════════════════════════════════════════════════════════════
// TOOLS BRIDGE
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * ToolsBridge - Facade for tool management operations
 *
 * Provides a clean API for:
 * - Setting and getting the current tool
 * - Tool state management
 * - Tool shortcuts and metadata
 *
 * This bridge wraps the flat WasmBridge API:
 * - `set_tool(tool: string)` → `setTool("select")`
 * - `get_tool()` → `"select"`
 *
 * @example
 * ```typescript
 * const tools = new ToolsBridge(bridge);
 *
 * // Set current tool
 * tools.setTool("rectangle");
 *
 * // Get current tool
 * const current = tools.getTool();
 * console.log(`Active tool: ${current}`);
 *
 * // Check if specific tool is active
 * if (tools.isToolActive("select")) {
 *   // Handle selection mode
 * }
 * ```
 */
export class ToolsBridge {
  /**
   * Reference to the underlying WASM bridge
   */
  private bridge: WasmBridge;

  /**
   * Currently active tool cache (for state management)
   */
  private cachedTool: ToolType = DEFAULT_TOOL;

  /**
   * Create a new ToolsBridge
   *
   * @param bridge - The WASM bridge instance
   */
  constructor(bridge: WasmBridge) {
    this.bridge = bridge;
    this.initializeDefaultTool();
  }

  /**
   * Initialize with default tool
   */
  private initializeDefaultTool(): void {
    try {
      const wasmTool = this.bridge.get_tool();
      if (wasmTool && this.isValidTool(wasmTool)) {
        this.cachedTool = wasmTool as ToolType;
      } else {
        this.setTool(DEFAULT_TOOL);
      }
    } catch {
      // Fallback: set default tool
      this.setTool(DEFAULT_TOOL);
    }
  }

  /**
   * Check if a tool string is a valid ToolType
   *
   * @param tool - Tool string to validate
   * @returns True if valid tool
   */
  private isValidTool(tool: string): tool is ToolType {
    return (TOOL_REGISTRY.map((t) => t.id) as unknown as string[]).includes(
      tool,
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TOOL STATE OPERATIONS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Set the current tool
   *
   * Changes the active tool for the canvas. This affects how mouse
   * and keyboard events are interpreted.
   *
   * @param tool - The tool to activate
   *
   * @example
   * ```typescript
   * // Switch to rectangle tool
   * tools.setTool("rectangle");
   *
   * // Switch to pan tool
   * tools.setTool("pan");
   * ```
   */
  setTool(tool: ToolType): void {
    this.bridge.set_tool(tool);
    this.cachedTool = tool;
  }

  /**
   * Get the current tool
   *
   * @returns The currently active tool identifier
   *
   * @example
   * ```typescript
   * const current = tools.getTool();
   * if (current === "select") {
   *   // Selection mode is active
   * }
   * ```
   */
  getTool(): ToolType {
    return this.cachedTool;
  }

  /**
   * Check if a specific tool is currently active
   *
   * @param tool - Tool to check
   * @returns True if the tool is active
   *
   * @example
   * ```typescript
   * if (tools.isToolActive("select")) {
   *   // Enable selection handles on shapes
   * }
   * ```
   */
  isToolActive(tool: ToolType): boolean {
    return this.cachedTool === tool;
  }

  /**
   * Check if current tool is a shape creation tool
   *
   * @returns True if creating shapes (not select/pan)
   *
   * @example
   * ```typescript
   * if (tools.isCreatingShapes()) {
   *   // Show shape preview while dragging
   * }
   * ```
   */
  isCreatingShapes(): boolean {
    return (
      this.cachedTool === "rectangle" ||
      this.cachedTool === "circle" ||
      this.cachedTool === "ellipse" ||
      this.cachedTool === "path" ||
      this.cachedTool === "text" ||
      this.cachedTool === "connector" ||
      this.cachedTool === "image"
    );
  }

  /**
   * Check if current tool allows selection
   *
   * @returns True if selection is allowed
   *
   * @example
   * ```typescript
   * if (tools.canSelect()) {
   *   // Render selection handles
   * }
   * ```
   */
  canSelect(): boolean {
    return this.cachedTool === "select";
  }

  /**
   * Check if current tool allows panning
   *
   * @returns True if panning is allowed
   *
   * @example
   * ```typescript
   * if (tools.canPan()) {
   *   // Update cursor for panning
   * }
   * ```
   */
  canPan(): boolean {
    return this.cachedTool === "pan" || this.cachedTool === "hand";
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // TOOL REGISTRY ACCESS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Get all available tools
   *
   * @returns Array of tool definitions
   *
   * @example
   * ```typescript
   * const allTools = tools.getAvailableTools();
   * // Render tool palette
   * ```
   */
  getAvailableTools(): ToolInfo[] {
    return [...TOOL_REGISTRY];
  }

  /**
   * Get tools by category
   *
   * @param category - Tool category to filter by
   * @returns Array of tools in that category
   *
   * @example
   * ```typescript
   * const shapeTools = tools.getToolsByCategory("shapes");
   * ```
   */
  getToolsByCategory(category: ToolInfo["category"]): ToolInfo[] {
    return TOOL_REGISTRY.filter((t) => t.category === category);
  }

  /**
   * Get tool info by ID
   *
   * @param id - Tool identifier
   * @returns Tool definition or undefined
   *
   * @example
   * ```typescript
   * const rectInfo = tools.getToolInfo("rectangle");
   * console.log(rectInfo?.shortcut); // "R"
   * ```
   */
  getToolInfo(id: ToolType): ToolInfo | undefined {
    return TOOL_REGISTRY.find((t) => t.id === id);
  }

  /**
   * Find tool by keyboard shortcut
   *
   * @param key - Key pressed (e.g., "V", "R", " ")
   * @returns Tool with that shortcut or undefined
   *
   * @example
   * ```typescript
   * // Handle keyboard shortcut
   * const tool = tools.findByShortcut("R");
   * if (tool) tools.setTool(tool.id);
   * ```
   */
  findByShortcut(key: string): ToolInfo | undefined {
    return TOOL_REGISTRY.find((t) => t.shortcut === key);
  }

  /**
   * Get all tool shortcuts
   *
   * @returns Map of key → tool ID
   *
   * @example
   * ```typescript
   * const shortcuts = tools.getShortcuts();
   * // Bind keyboard events
   * ```
   */
  getShortcuts(): Map<string, ToolType> {
    const map = new Map<string, ToolType>();
    TOOL_REGISTRY.forEach((tool) => {
      if (tool.shortcut) {
        map.set(tool.shortcut, tool.id);
      }
    });
    return map;
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // CONVENIENCE METHODS
  // ═══════════════════════════════════════════════════════════════════════════

  /**
   * Switch to selection tool
   *
   * @example
   * ```typescript
   * tools.selectSelectionTool();
   * ```
   */
  selectSelectionTool(): void {
    this.setTool("select");
  }

  /**
   * Switch to pan tool
   *
   * @example
   * ```typescript
   * tools.selectPanTool();
   * ```
   */
  selectPanTool(): void {
    this.setTool("pan");
  }

  /**
   * Get the shape type for the current tool
   *
   * @returns Shape type or null if not a shape tool
   *
   * @example
   * ```typescript
   * const shapeType = tools.getShapeTypeForTool();
   * if (shapeType) {
   *   // Creating a shape
   * }
   * ```
   */
  getShapeTypeForTool(): string | null {
    const shapeTools: Record<ToolType, string> = {
      rectangle: "rectangle",
      circle: "circle",
      ellipse: "ellipse",
      path: "path",
      text: "text",
      connector: "connector",
      image: "image",
      select: "select",
      pan: "pan",
      hand: "hand",
    };
    return shapeTools[this.cachedTool] || null;
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Default ToolsBridge factory function
 *
 * Creates and initializes a ToolsBridge with the provided bridge.
 *
 * @param bridge - The WASM bridge instance
 * @returns Configured ToolsBridge instance
 *
 * @example
 * ```typescript
 * import { createToolsBridge } from './ToolsBridge';
 *
 * const tools = createToolsBridge(bridge);
 * tools.setTool("rectangle");
 * ```
 */
export function createToolsBridge(bridge: any): ToolsBridge {
  return new ToolsBridge(bridge);
}

export default ToolsBridge;
