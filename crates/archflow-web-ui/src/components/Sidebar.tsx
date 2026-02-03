/**
 * Sidebar Component - Entity Library Panel
 *
 * Provides draggable entity templates that can be dragged onto the canvas.
 * Uses @dnd-kit for drag and drop functionality.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useState, useMemo } from "react";
import {
  Server,
  HardDrive,
  Database,
  Globe,
  Zap,
  Search,
  ChevronRight,
  Box,
  Lightbulb
} from "lucide-react";
import { cn } from "../utils/cn";
import { useDragAndDrop, type EntityTemplate } from "../hooks/useDragAndDrop";

/**
 * Entity categories for organization
 */
type EntityCategory =
  | "compute"
  | "storage"
  | "database"
  | "network"
  | "security"
  | "integration";

/**
 * Category labels for display
 */
const categoryLabels: Record<EntityCategory, string> = {
  compute: "Compute",
  storage: "Storage",
  database: "Database",
  network: "Networking",
  security: "Security",
  integration: "Integration",
};

/**
 * Entity templates available in the sidebar
 */
const entityTemplates: EntityTemplate[] = [
  {
    type: "aws-ec2",
    name: "EC2",
    icon: Server,
    category: "compute",
    defaultSize: { width: 120, height: 80 },
    description: "Virtual server",
  },
  {
    type: "aws-lambda",
    name: "Lambda",
    icon: Zap,
    category: "compute",
    defaultSize: { width: 100, height: 60 },
    description: "Serverless function",
  },
  {
    type: "aws-eks",
    name: "EKS",
    icon: Box,
    category: "compute",
    defaultSize: { width: 120, height: 80 },
    description: "Kubernetes Service",
  },
  {
    type: "aws-lightsail",
    name: "Lightsail",
    icon: Lightbulb,
    category: "compute",
    defaultSize: { width: 120, height: 80 },
    description: "Virtual Private Server",
  },
  {
    type: "aws-s3",
    name: "S3 Bucket",
    icon: HardDrive,
    category: "storage",
    defaultSize: { width: 100, height: 80 },
    description: "Object storage",
  },
  {
    type: "aws-rds",
    name: "RDS Database",
    icon: Database,
    category: "database",
    defaultSize: { width: 120, height: 80 },
    description: "Relational database",
  },
  {
    type: "aws-vpc",
    name: "VPC",
    icon: Globe,
    category: "network",
    defaultSize: { width: 150, height: 100 },
    description: "Isolated network",
  },
];

interface SidebarProps {
  className?: string;
  isOpen?: boolean;
}

/**
 * Sidebar component with draggable entity templates
 */
export default function Sidebar({ className, isOpen = true }: SidebarProps) {
  const [expandedCategories, setExpandedCategories] = useState<
    Set<EntityCategory>
  >(new Set(["compute", "storage", "database", "network"]));
  const [searchQuery, setSearchQuery] = useState("");

  const { DndProvider, DraggableItem } = useDragAndDrop();

  /**
   * Toggle category expansion
   */
  const toggleCategory = (category: EntityCategory) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) next.delete(category);
      else next.add(category);
      return next;
    });
  };

  /**
   * Filter templates based on search query
   */
  const filteredTemplates = useMemo(
    () =>
      searchQuery
        ? entityTemplates.filter((t) =>
          t.name.toLowerCase().includes(searchQuery.toLowerCase()),
        )
        : entityTemplates,
    [searchQuery],
  );

  /**
   * Group templates by category
   */
  const groupedTemplates = useMemo(
    () =>
      filteredTemplates.reduce(
        (acc, template) => {
          const cat = template.category as EntityCategory;
          if (!acc[cat]) acc[cat] = [];
          acc[cat].push(template);
          return acc;
        },
        {} as Record<EntityCategory, EntityTemplate[]>,
      ),
    [filteredTemplates],
  );

  if (!isOpen) return null;

  return (
    <DndProvider>
      <aside
        className={cn(
          "w-64 bg-surface-light dark:bg-surface-dark border-r border-border-light dark:border-border-dark flex flex-col shrink-0 z-20 shadow-sm",
          className,
        )}
      >
        {/* Search */}
        <div className="p-3 border-b border-border-light dark:border-border-dark">
          <div className="relative group">
            <span className="absolute inset-y-0 left-0 flex items-center pl-3 text-slate-400 group-focus-within:text-primary transition-colors">
              <Search className="w-5 h-5" />
            </span>
            <input
              className="w-full bg-background-light dark:bg-background-dark border-transparent focus:border-primary/50 focus:ring-0 rounded-md py-2 pl-9 pr-3 text-sm placeholder-slate-400 dark:placeholder-slate-500 text-slate-900 dark:text-white transition-all outline-none"
              placeholder="Search resources..."
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <div className="absolute inset-y-0 right-0 flex items-center pr-2">
              <kbd className="hidden sm:inline-block border border-slate-200 dark:border-slate-700 rounded px-1 text-[10px] font-mono font-medium text-slate-400">
                ⌘K
              </kbd>
            </div>
          </div>
        </div>

        {/* Library Content */}
        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {(searchQuery
            ? Object.keys(groupedTemplates)
            : Object.keys(groupedTemplates) // Iterate only over categories with items or all if desired
          ).map((category) => {
            const cat = category as EntityCategory;
            const templates = groupedTemplates[cat] || [];
            if (!searchQuery && templates.length === 0) return null;

            const isExpanded = expandedCategories.has(cat);
            // const CategoryIcon = categoryIcons[cat]; // Unused in reference design for header

            return (
              <div key={cat} className="group">
                <button
                  className="flex items-center justify-between w-full p-2 text-left text-sm font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800 rounded-md transition-colors"
                  onClick={() => !searchQuery && toggleCategory(cat)}
                >
                  <div className="flex items-center gap-2">
                    <span className={cn(
                      "material-symbols-outlined text-[18px] text-slate-400 transition-transform duration-200",
                      isExpanded ? "rotate-90" : ""
                    )}>
                      <ChevronRight className="w-4 h-4" />
                    </span>
                    <span>{categoryLabels[cat]}</span>
                  </div>
                  <span className="text-xs text-slate-400 font-mono">{templates.length}</span>
                </button>

                {(isExpanded || searchQuery) && (
                  <div className={cn(
                    "pl-4 pr-1 pb-2 pt-1 gap-2",
                    cat === 'compute' ? "grid grid-cols-2" : "flex flex-col space-y-1"
                  )}>
                    {templates.map((template) => {
                      const Icon = template.icon;

                      // Render as Grid Item (Card) for Compute
                      if (cat === 'compute') {
                        return (
                          <DraggableItem key={template.type} template={template}>
                            {({ setNodeRef, listeners, attributes }) => (
                              <div
                                ref={setNodeRef}
                                {...listeners}
                                {...attributes}
                                className="flex flex-col items-center justify-center p-2 rounded border border-transparent hover:border-primary/30 hover:bg-primary/5 cursor-grab active:cursor-grabbing group/item transition-all"
                              >
                                <div className="size-8 mb-1 flex items-center justify-center text-primary bg-primary/10 rounded">
                                  <Icon className="w-5 h-5" />
                                </div>
                                <span className="text-[10px] text-center font-medium text-slate-600 dark:text-slate-400 group-hover/item:text-primary">
                                  {template.name}
                                </span>
                              </div>
                            )}
                          </DraggableItem>
                        );
                      }

                      // Render as List Item for others
                      return (
                        <DraggableItem key={template.type} template={template}>
                          {({ setNodeRef, listeners, attributes }) => (
                            <div
                              ref={setNodeRef}
                              {...listeners}
                              {...attributes}
                              className="flex items-center gap-3 p-2 rounded hover:bg-slate-50 dark:hover:bg-slate-800 cursor-grab active:cursor-grabbing group/item transition-all"
                            >
                              <div className="size-6 flex items-center justify-center text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 rounded group-hover/item:text-primary group-hover/item:bg-primary/10">
                                <Icon className="w-4 h-4" />
                              </div>
                              <span className="text-xs font-medium text-slate-600 dark:text-slate-300 group-hover/item:text-primary">
                                {template.name}
                              </span>
                            </div>
                          )}
                        </DraggableItem>
                      );
                    })}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* User Status Footer */}
        <div className="p-3 border-t border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark/50">
          <div className="flex items-center justify-between text-xs text-slate-500">
            <span className="flex items-center gap-1.5">
              <span className="size-2 rounded-full bg-green-500"></span>
              AWS Connected
            </span>
            <span className="font-mono">v2.4.0</span>
          </div>
        </div>
      </aside>
    </DndProvider>
  );
}
