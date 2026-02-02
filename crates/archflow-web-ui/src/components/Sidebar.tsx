/**
 * Sidebar Component - Entity Library Panel
 */

import { useState } from "react";
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
import { useUIStore } from "../store/useUIStore";
import { useEntityStore } from "../hooks/useEntityStore";
import { useCamera } from "../hooks/useCamera";

type EntityCategory =
  | "compute"
  | "storage"
  | "database"
  | "network"
  | "security"
  | "integration";

interface EntityTemplate {
  type: string;
  name: string;
  icon: React.ComponentType<{ className?: string }>;
  category: EntityCategory;
  defaultSize: { width: number; height: number };
  description: string;
}

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

const categoryLabels: Record<EntityCategory, string> = {
  compute: "Compute",
  storage: "Storage",
  database: "Database",
  network: "Network",
  security: "Security",
  integration: "Integration",
};

interface SidebarProps {
  className?: string;
  isOpen?: boolean;
}

export default function Sidebar({ className, isOpen = true }: SidebarProps) {
  const [expandedCategories, setExpandedCategories] = useState<
    Set<EntityCategory>
  >(new Set(["compute", "storage", "database", "network"]));
  const [searchQuery, setSearchQuery] = useState("");
  const { setActiveTool } = useUIStore();
  const { spawnEntity } = useEntityStore();
  const { camera } = useCamera(null);

  const toggleCategory = (category: EntityCategory) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) next.delete(category);
      else next.add(category);
      return next;
    });
  };

  const filteredTemplates = searchQuery
    ? entityTemplates.filter((t) =>
        t.name.toLowerCase().includes(searchQuery.toLowerCase()),
      )
    : entityTemplates;

  const groupedTemplates = filteredTemplates.reduce(
    (acc, template) => {
      if (!acc[template.category]) acc[template.category] = [];
      acc[template.category].push(template);
      return acc;
    },
    {} as Record<EntityCategory, EntityTemplate[]>,
  );

  if (!isOpen) return null;

  return (
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
                      <button
                        key={template.type}
                        className="w-full flex items-center gap-2 px-2 py-2 rounded-lg text-sm text-gray-300 hover:bg-white/10 hover:text-white transition-colors cursor-grab"
                        title={template.description}
                        onClick={() => {
                          spawnEntity(
                            camera.center.x,
                            camera.center.y,
                            template.defaultSize.width,
                            template.defaultSize.height,
                          );
                          setActiveTool("select");
                        }}
                      >
                        <Icon className="w-4 h-4 text-primary/80" />
                        <span>{template.name}</span>
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </aside>
  );
}
