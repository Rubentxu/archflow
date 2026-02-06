/**
 * Context Menu Store
 *
 * Manages the state for context menus across the application.
 * Supports multiple menu types and positioning.
 */

import { create } from "zustand";

/**
 * Entity ID type
 */
export type EntityId = number;

/**
 * Context menu types
 */
export type ContextMenuType =
  | "canvas"
  | "entity"
  | "connection"
  | "selection"
  | "layer"
  | null;

/**
 * Context menu position
 */
export interface ContextMenuPosition {
  x: number;
  y: number;
}

/**
 * Context menu item action types
 */
export type MenuAction =
  | "copy"
  | "paste"
  | "delete"
  | "duplicate"
  | "cut"
  | "undo"
  | "redo"
  | "selectAll"
  | "deselectAll"
  | "group"
  | "ungroup"
  | "bringForward"
  | "sendBackward"
  | "bringToFront"
  | "sendToBack"
  | "lock"
  | "unlock"
  | "rename"
  | "copyAsPng"
  | "copyAsSvg"
  | "export"
  | "viewSource"
  | "addComment"
  | "createChild"
  | "expand"
  | "collapse";

/**
 * Context menu item
 */
export interface ContextMenuItem {
  id: MenuAction;
  label: string;
  icon?: React.ReactNode;
  shortcut?: string;
  disabled?: boolean;
  destructive?: boolean;
  divider?: boolean;
  submenu?: ContextMenuItem[];
}

/**
 * Context menu state
 */
interface ContextMenuState {
  // Menu visibility and type
  isOpen: boolean;
  menuType: ContextMenuType;
  position: ContextMenuPosition;

  // Target information
  targetEntityId: EntityId | null;
  targetEntityIds: EntityId[];

  // Menu items
  items: ContextMenuItem[];

  // Actions
  open: (
    type: ContextMenuType,
    position: ContextMenuPosition,
    targetInfo?: {
      entityId?: EntityId;
      entityIds?: EntityId[];
    },
  ) => void;
  close: () => void;
  updatePosition: (position: ContextMenuPosition) => void;
  setItems: (items: ContextMenuItem[]) => void;
}

/**
 * Create a divider item
 */
function createDivider(): ContextMenuItem {
  return { id: "copy", label: "", divider: true };
}

/**
 * Default menu items for different contexts
 */
const CANVAS_MENU_ITEMS: ContextMenuItem[] = [
  { id: "paste", label: "Paste", shortcut: "Ctrl+V" },
  { id: "selectAll", label: "Select All", shortcut: "Ctrl+A" },
  createDivider(),
  { id: "viewSource", label: "View Source" },
  { id: "export", label: "Export", shortcut: "Ctrl+E" },
];

const ENTITY_MENU_ITEMS: ContextMenuItem[] = [
  { id: "copy", label: "Copy", shortcut: "Ctrl+C" },
  { id: "cut", label: "Cut", shortcut: "Ctrl+X" },
  { id: "duplicate", label: "Duplicate", shortcut: "Ctrl+D" },
  createDivider(),
  { id: "delete", label: "Delete", shortcut: "Del", destructive: true },
  createDivider(),
  { id: "group", label: "Group", shortcut: "Ctrl+G" },
  { id: "ungroup", label: "Ungroup", shortcut: "Ctrl+Shift+G" },
  createDivider(),
  {
    id: "bringForward",
    label: "Bring Forward",
    submenu: [
      { id: "bringForward", label: "Bring Forward", shortcut: "Ctrl+]" },
      { id: "bringToFront", label: "Bring to Front", shortcut: "Ctrl+Shift+]" },
    ],
  },
  {
    id: "sendBackward",
    label: "Send Backward",
    submenu: [
      { id: "sendBackward", label: "Send Backward", shortcut: "Ctrl+[" },
      { id: "sendToBack", label: "Send to Back", shortcut: "Ctrl+Shift+[" },
    ],
  },
  createDivider(),
  { id: "lock", label: "Lock" },
  { id: "unlock", label: "Unlock" },
];

const SELECTION_MENU_ITEMS: ContextMenuItem[] = [
  { id: "copy", label: "Copy", shortcut: "Ctrl+C" },
  { id: "cut", label: "Cut", shortcut: "Ctrl+X" },
  { id: "duplicate", label: "Duplicate", shortcut: "Ctrl+D" },
  createDivider(),
  { id: "delete", label: "Delete", shortcut: "Del", destructive: true },
  createDivider(),
  { id: "group", label: "Group", shortcut: "Ctrl+G" },
  { id: "ungroup", label: "Ungroup", shortcut: "Ctrl+Shift+G" },
  createDivider(),
  {
    id: "bringForward",
    label: "Arrange",
    submenu: [
      { id: "bringForward", label: "Bring Forward" },
      { id: "bringToFront", label: "Bring to Front" },
      { id: "sendBackward", label: "Send Backward" },
      { id: "sendToBack", label: "Send to Back" },
    ],
  },
];

/**
 * Context menu store
 */
export const useContextMenuStore = create<ContextMenuState>((set) => ({
  isOpen: false,
  menuType: null,
  position: { x: 0, y: 0 },
  targetEntityId: null,
  targetEntityIds: [],
  items: [],

  open: (type, position, targetInfo) => {
    const items = getItemsForType(type);
    set({
      isOpen: true,
      menuType: type,
      position,
      targetEntityId: targetInfo?.entityId ?? null,
      targetEntityIds: targetInfo?.entityIds ?? [],
      items,
    });
  },

  close: () => {
    set({
      isOpen: false,
      menuType: null,
      targetEntityId: null,
      targetEntityIds: [],
    });
  },

  updatePosition: (position) => {
    set({ position });
  },

  setItems: (items) => {
    set({ items });
  },
}));

/**
 * Get default items for a menu type
 */
function getItemsForType(type: ContextMenuType): ContextMenuItem[] {
  switch (type) {
    case "canvas":
      return CANVAS_MENU_ITEMS;
    case "entity":
      return ENTITY_MENU_ITEMS;
    case "selection":
      return SELECTION_MENU_ITEMS;
    case "connection":
      return [
        { id: "copy", label: "Copy Connection" },
        { id: "delete", label: "Delete Connection", destructive: true },
      ];
    case "layer":
      return [
        { id: "rename", label: "Rename Layer" },
        { id: "duplicate", label: "Duplicate Layer" },
        createDivider(),
        { id: "lock", label: "Lock Layer" },
        { id: "unlock", label: "Unlock Layer" },
        createDivider(),
        { id: "delete", label: "Delete Layer", destructive: true },
      ];
    default:
      return [];
  }
}
