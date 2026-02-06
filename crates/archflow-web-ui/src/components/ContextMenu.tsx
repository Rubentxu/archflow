/**
 * Context Menu Component
 *
 * A professional context menu component with keyboard navigation,
 * submenu support, and animations.
 */

import { useEffect, useRef, useState, useCallback, memo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  useContextMenuStore,
  type ContextMenuItem,
  type MenuAction,
} from "../store/useContextMenuStore";
import { useSelectionCommands } from "../hooks/useCommandHistory";
import { useSelectionStore } from "../store/useSelectionStore";
import { cn } from "../utils/cn";
import {
  Copy,
  Scissors,
  ClipboardPaste,
  Trash2,
  Undo2,
  Redo2,
  Lock,
  Unlock,
  Group,
  Ungroup,
  ArrowUp,
  ArrowDown,
  BringToFront,
  SendToBack,
  FileImage,
  FileType,
  Download,
  ChevronRight,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

/**
 * Get icon component for menu action
 */
function getIconForAction(action: MenuAction): LucideIcon | undefined {
  const iconMap: Partial<Record<MenuAction, LucideIcon>> = {
    copy: Copy,
    cut: Scissors,
    paste: ClipboardPaste,
    delete: Trash2,
    duplicate: Copy,
    undo: Undo2,
    redo: Redo2,
    lock: Lock,
    unlock: Unlock,
    group: Group,
    ungroup: Ungroup,
    bringForward: ArrowUp,
    bringToFront: BringToFront,
    sendBackward: ArrowDown,
    sendToBack: SendToBack,
    copyAsPng: FileImage,
    copyAsSvg: FileType,
    export: Download,
  };

  return iconMap[action];
}

/**
 * Context Menu Item Component
 */
const ContextMenuItemComponent = memo(function ContextMenuItemComponent({
  item,
  depth = 0,
  onAction,
}: {
  item: ContextMenuItem;
  depth?: number;
  onAction: (action: MenuAction) => void;
}) {
  const [isHovered, setIsHovered] = useState(false);
  const [showSubmenu, setShowSubmenu] = useState(false);
  const itemRef = useRef<HTMLButtonElement>(null);
  const submenuRef = useRef<HTMLDivElement>(null);
  const submenuTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );

  const hasSubmenu = (item.submenu?.length ?? 0) > 0;

  // Handle mouse enter/leave for submenu
  const handleMouseEnter = useCallback(() => {
    if (hasSubmenu) {
      if (submenuTimeoutRef.current) {
        clearTimeout(submenuTimeoutRef.current);
      }
      setShowSubmenu(true);
    }
    setIsHovered(true);
  }, [hasSubmenu]);

  const handleMouseLeave = useCallback(() => {
    setIsHovered(false);
    if (hasSubmenu) {
      submenuTimeoutRef.current = setTimeout(() => {
        setShowSubmenu(false);
      }, 150);
    }
  }, [hasSubmenu]);

  // Handle click
  const handleClick = useCallback(() => {
    if (hasSubmenu) {
      setShowSubmenu(true);
    } else if (!item.disabled) {
      onAction(item.id);
    }
  }, [hasSubmenu, item.disabled, item.id, onAction]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isHovered) return;

      if (e.key === "Enter" && !item.disabled) {
        onAction(item.id);
      } else if (e.key === "ArrowRight" && hasSubmenu) {
        setShowSubmenu(true);
      } else if (e.key === "ArrowLeft" && depth > 0) {
        setShowSubmenu(false);
      }
    };

    const element = itemRef.current;
    if (element) {
      element.addEventListener("keydown", handleKeyDown);
    }

    return () => {
      if (element) {
        element.removeEventListener("keydown", handleKeyDown);
      }
    };
  }, [isHovered, item.disabled, item.id, onAction, hasSubmenu, depth]);

  if (item.divider) {
    return (
      <div
        className={cn(
          "my-1 h-px bg-gray-200 dark:bg-gray-700",
          depth > 0 && "mx-2",
        )}
        role="separator"
      />
    );
  }

  const Icon = item.icon
    ? (item.icon as React.ReactNode)
    : getIconForAction(item.id);

  return (
    <div className="relative">
      <button
        ref={itemRef}
        className={cn(
          "flex w-full items-center gap-2 px-3 py-1.5 text-sm transition-colors",
          "outline-none focus:bg-blue-100 dark:focus:bg-blue-900",
          item.disabled
            ? "cursor-not-allowed text-gray-400 dark:text-gray-500"
            : "cursor-pointer text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800",
          item.destructive &&
            "hover:bg-red-50 dark:hover:bg-red-900/20 text-red-600 dark:text-red-400",
          depth > 0 && "pl-6",
        )}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onClick={handleClick}
        disabled={item.disabled}
        role="menuitem"
      >
        {Icon && typeof Icon !== "string" && "size" in Icon ? (
          <Icon
            size={16}
            className={cn(item.destructive && "text-red-500 dark:text-red-400")}
          />
        ) : null}
        <span className="flex-1 text-left">{item.label}</span>
        {item.shortcut && (
          <span className="text-xs text-gray-400 dark:text-gray-500">
            {item.shortcut}
          </span>
        )}
        {hasSubmenu && (
          <ChevronRight
            size={14}
            className="text-gray-400 dark:text-gray-500"
          />
        )}
      </button>

      {/* Submenu */}
      <AnimatePresence>
        {hasSubmenu && showSubmenu && item.submenu && (
          <motion.div
            ref={submenuRef}
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.1 }}
            className="absolute left-full top-0 ml-0.5 min-w-[160px] rounded-lg border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-700 dark:bg-gray-900"
            style={{
              zIndex: 1000 + depth,
            }}
            onMouseEnter={() => {
              if (submenuTimeoutRef.current) {
                clearTimeout(submenuTimeoutRef.current);
              }
            }}
            onMouseLeave={() => {
              submenuTimeoutRef.current = setTimeout(() => {
                setShowSubmenu(false);
              }, 150);
            }}
          >
            {item.submenu.map((subItem, index) => (
              <ContextMenuItemComponent
                key={subItem.id ?? index}
                item={subItem}
                depth={depth + 1}
                onAction={onAction}
              />
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
});

/**
 * Context Menu Component
 */
export const ContextMenu = memo(function ContextMenu() {
  const { isOpen, position, items, close, updatePosition } =
    useContextMenuStore();

  const { duplicate, deleteSelected } = useSelectionCommands();
  const menuRef = useRef<HTMLDivElement>(null);

  // Handle window resize to prevent menu going off-screen
  useEffect(() => {
    if (!isOpen) return;

    const handleResize = () => {
      if (!menuRef.current) return;

      const rect = menuRef.current.getBoundingClientRect();
      const maxX = window.innerWidth - rect.width - 10;
      const maxY = window.innerHeight - rect.height - 10;

      let newX = position.x;
      let newY = position.y;

      if (newX > maxX) newX = Math.max(10, maxX);
      if (newY > maxY) newY = Math.max(10, maxY);

      if (newX !== position.x || newY !== position.y) {
        updatePosition({ x: newX, y: newY });
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [isOpen, position.x, position.y, updatePosition]);

  // Close on click outside
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        close();
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        close();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleEscape);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, close]);

  // Handle menu actions
  const handleAction = useCallback(
    (action: MenuAction) => {
      console.log("[ContextMenu] Action triggered:", action);

      switch (action) {
        case "delete":
          deleteSelected();
          break;
        case "duplicate":
          // Get first selected entity and duplicate
          const { selectedIds } = useSelectionStore.getState();
          if (selectedIds.length > 0) {
            duplicate(selectedIds[0]);
          }
          break;
        // Add more actions as needed
      }

      close();
    },
    [close, duplicate, deleteSelected],
  );

  if (!isOpen || items.length === 0) return null;

  return (
    <AnimatePresence>
      <motion.div
        ref={menuRef}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        transition={{ duration: 0.1 }}
        className="fixed z-50 min-w-[200px] rounded-lg border border-gray-200 bg-white py-1 shadow-xl dark:border-gray-700 dark:bg-gray-900"
        style={{
          left: position.x,
          top: position.y,
        }}
        role="menu"
        aria-orientation="vertical"
      >
        {items.map((item, index) => (
          <ContextMenuItemComponent
            key={item.id ?? index}
            item={item}
            onAction={handleAction}
          />
        ))}
      </motion.div>
    </AnimatePresence>
  );
});
