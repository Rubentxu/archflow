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
  Shield,
  Cpu,
  Search,
  ChevronRight,
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
 * Category icons mapping
 */
const categoryIcons: Record<
  EntityCategory,
  React.ComponentType<{ className?: string }>
> = {
  compute: Cpu,
  storage: HardDrive,
  database: Database,
  network: Globe,
  security: Shield,
  integration: Zap,
};

/**
 * Category labels for display
 */
const categoryLabels: Record<EntityCategory, string> = {
  compute: "Compute",
  storage: "Storage",
  database: "Database",
  network: "Network",
  security: "Security",
  integration: "Integration",
};

/**
 * Entity templates available in the sidebar
 */
const entityTemplates: EntityTemplate[] = [
  {
    type: "aws-ec2",
    name: "EC2 Instance",
    icon: Server,
    category: "compute",
    defaultSize: { width: 120, height: 80 },
    description: "Virtual server",
  },
  {
    type: "aws-lambda",
    name: "Lambda Function",
    icon: Cpu,
    category: "compute",
    defaultSize: { width: 100, height: 60 },
    description: "Serverless function",
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
  {
    type: "aws-api-gateway",
    name: "API Gateway",
    icon: Zap,
    category: "network",
    defaultSize: { width: 100, height: 70 },
    description: "API management",
  },
  {
    type: "aws-iam",
    name: "IAM Role",
    icon: Shield,
    category: "security",
    defaultSize: { width: 100, height: 60 },
    description: "Identity management",
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
          "w-64 h-full flex flex-col bg-surface-dark/95 border-r border-white/5",
          className,
        )}
      >
        <div className="p-3 border-b border-white/5">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              placeholder="Search components..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-9 pr-3 py-2 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 placeholder-gray-500 focus:outline-none focus:border-primary/50"
            />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          {(searchQuery
            ? Object.keys(groupedTemplates)
            : Object.keys(categoryLabels)
          ).map((category) => {
            const cat = category as EntityCategory;
            const templates = groupedTemplates[cat] || [];
            const isExpanded = expandedCategories.has(cat);
            const CategoryIcon = categoryIcons[cat];

            if (searchQuery && templates.length === 0) return null;

            return (
              <div key={cat} className="mb-1">
                <button
                  className="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-xs font-medium text-gray-400 uppercase tracking-wider hover:bg-white/5 hover:text-white transition-colors"
                  onClick={() => !searchQuery && toggleCategory(cat)}
                >
                  {searchQuery ? null : isExpanded ? null : (
                    <ChevronRight className="w-3 h-3" />
                  )}
                  <CategoryIcon className="w-4 h-4" />
                  <span>
                    {searchQuery
                      ? `${templates.length} results`
                      : categoryLabels[cat]}
                  </span>
                </button>

                {(isExpanded || searchQuery) && (
                  <div className="mt-1 space-y-1 ml-4">
                    {templates.map((template) => {
                      const Icon = template.icon;
                      return (
                        <DraggableItem key={template.type} template={template}>
                          {({
                            isDragging,
                            listeners,
                            setNodeRef,
                            transform,
                          }) => (
                            <button
                              ref={setNodeRef}
                              {...listeners}
                              className={cn(
                                "w-full flex items-center gap-2 px-2 py-2 rounded-lg text-sm transition-colors cursor-grab",
                                isDragging
                                  ? "bg-primary/20 text-primary"
                                  : "text-gray-300 hover:bg-white/10 hover:text-white",
                              )}
                              title={template.description}
                              style={{
                                transform: transform
                                  ? `translate3d(${transform.x}px, ${transform.y}px, 0)`
                                  : undefined,
                              }}
                            >
                              <Icon className="w-4 h-4 text-primary/80" />
                              <span>{template.name}</span>
                            </button>
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
      </aside>
    </DndProvider>
  );
}
