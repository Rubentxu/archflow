/**
 * Architecture Demo - C4 Diagram Example
 *
 * Demonstrates the whiteboard capabilities with a complete
 * AWS architecture diagram showing common cloud patterns.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import { useCallback, useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Cloud,
  Server,
  Database,
  HardDrive,
  Globe,
  FunctionSquare,
  Users,
  Activity,
  ZoomIn,
  ZoomOut,
  Play,
  Pause,
  RefreshCw,
} from "lucide-react";
import { cn } from "../utils/cn";
import { useCamera } from "../hooks/useCamera";
import { useSelectionStore } from "../store/useSelectionStore";
import { entityVariants, buttonVariants } from "../utils/animations";

/**
 * Demo entity types
 */
interface DemoEntity {
  id: number;
  type: string;
  label: string;
  position: { x: number; y: number };
  size: { w: number; h: number };
  color: string;
  properties: Record<string, string>;
}

/**
 * Demo connection types
 */
interface DemoConnection {
  id: string;
  sourceId: number;
  targetId: number;
  label: string;
}

/**
 * Props for DemoArchitecture component
 */
interface DemoArchitectureProps {
  /** Auto-start animation */
  autoStart?: boolean;
  /** Additional CSS class */
  className?: string;
}

/**
 * Sample AWS architecture data
 */
const demoEntities: DemoEntity[] = [
  // User Layer
  {
    id: 1,
    type: "actor",
    label: "Users",
    position: { x: 100, y: 100 },
    size: { w: 120, h: 60 },
    color: "#1a2c32",
    properties: { count: "10K+", region: "Global" },
  },
  // CDN Layer
  {
    id: 2,
    type: "aws-cloudfront",
    label: "CloudFront",
    position: { x: 300, y: 90 },
    size: { w: 140, h: 80 },
    color: "#1a3a52",
    properties: { edgeLocations: "200+", ssl: "true" },
  },
  // API Layer
  {
    id: 3,
    type: "aws-api-gateway",
    label: "API Gateway",
    position: { x: 520, y: 90 },
    size: { w: 160, h: 80 },
    color: "#1a4a52",
    properties: { rateLimit: "10K RPS", auth: "Cognito" },
  },
  // Compute Layer
  {
    id: 4,
    type: "aws-lambda",
    label: "Auth Function",
    position: { x: 520, y: 230 },
    size: { w: 140, h: 70 },
    color: "#1a5a32",
    properties: { runtime: "Node.js 20", memory: "256MB", timeout: "30s" },
  },
  {
    id: 5,
    type: "aws-lambda",
    label: "API Handler",
    position: { x: 520, y: 350 },
    size: { w: 140, h: 70 },
    color: "#1a5a32",
    properties: { runtime: "Python 3.12", memory: "512MB", timeout: "60s" },
  },
  {
    id: 6,
    type: "aws-lambda",
    label: "Processor",
    position: { x: 520, y: 470 },
    size: { w: 140, h: 70 },
    color: "#1a5a32",
    properties: { runtime: "Python 3.12", memory: "1024MB", timeout: "300s" },
  },
  // Database Layer
  {
    id: 7,
    type: "aws-rds",
    label: "PostgreSQL",
    position: { x: 750, y: 250 },
    size: { w: 140, h: 80 },
    color: "#2a1a52",
    properties: {
      instance: "db.m6g.large",
      storage: "100GB",
      engine: "PostgreSQL 15",
    },
  },
  // Storage Layer
  {
    id: 8,
    type: "aws-s3",
    label: "S3 Bucket",
    position: { x: 750, y: 400 },
    size: { w: 140, h: 70 },
    color: "#2a4a32",
    properties: {
      storageClass: "Standard",
      versioning: "true",
      encryption: "AES256",
    },
  },
  // Cache Layer
  {
    id: 9,
    type: "aws-elasticache",
    label: "ElastiCache",
    position: { x: 750, y: 520 },
    size: { w: 140, h: 70 },
    color: "#3a2a52",
    properties: { engine: "Redis 7", nodeType: "cache.t3.micro" },
  },
  // Compute EC2
  {
    id: 10,
    type: "aws-ec2",
    label: "Worker EC2",
    position: { x: 300, y: 400 },
    size: { w: 140, h: 90 },
    color: "#1a2a62",
    properties: {
      instanceType: "t3.medium",
      ami: "Amazon Linux 2023",
      keyPair: "prod-key",
    },
  },
  // Queue
  {
    id: 11,
    type: "aws-sqs",
    label: "SQS Queue",
    position: { x: 300, y: 270 },
    size: { w: 140, h: 60 },
    color: "#2a3a42",
    properties: { visibilityTimeout: "300s", maxMessageSize: "256KB" },
  },
];

const demoConnections: DemoConnection[] = [
  { id: "c1", sourceId: 1, targetId: 2, label: "HTTPS Requests" },
  { id: "c2", sourceId: 2, targetId: 3, label: "Cached Content" },
  { id: "c3", sourceId: 3, targetId: 4, label: "Auth Request" },
  { id: "c4", sourceId: 3, targetId: 5, label: "API Calls" },
  { id: "c5", sourceId: 5, targetId: 7, label: "Queries" },
  { id: "c6", sourceId: 5, targetId: 8, label: "File Operations" },
  { id: "c7", sourceId: 5, targetId: 9, label: "Cache Reads" },
  { id: "c8", sourceId: 5, targetId: 6, label: "Async Tasks" },
  { id: "c9", sourceId: 6, targetId: 10, label: "Commands" },
  { id: "c10", sourceId: 6, targetId: 11, label: "Messages" },
  { id: "c11", sourceId: 11, targetId: 10, label: "Polls" },
  { id: "c12", sourceId: 4, targetId: 3, label: "Token Response" },
  { id: "c13", sourceId: 5, targetId: 3, label: "API Response" },
];

/**
 * Get icon component for entity type
 */
function getEntityIcon(type: string) {
  switch (type) {
    case "actor":
      return Users;
    case "aws-cloudfront":
    case "aws-api-gateway":
      return Globe;
    case "aws-lambda":
      return FunctionSquare;
    case "aws-rds":
    case "aws-elasticache":
      return Database;
    case "aws-s3":
      return HardDrive;
    case "aws-ec2":
      return Server;
    case "aws-sqs":
      return Activity;
    default:
      return Cloud;
  }
}

/**
 * Single entity component
 */
function DemoEntity({
  entity,
  isSelected,
  onSelect,
}: {
  entity: DemoEntity;
  isSelected: boolean;
  onSelect: () => void;
}) {
  const Icon = getEntityIcon(entity.type);
  const [isHovered, setIsHovered] = useState(false);

  return (
    <motion.div
      className={cn(
        "absolute rounded-lg flex items-center justify-center cursor-pointer",
        "border-2 transition-shadow duration-200",
        isSelected
          ? "border-primary shadow-lg shadow-primary/20"
          : "border-white/20 hover:border-white/40",
      )}
      style={{
        left: entity.position.x,
        top: entity.position.y,
        width: entity.size.w,
        height: entity.size.h,
        backgroundColor: entity.color,
      }}
      variants={entityVariants}
      initial="idle"
      animate={isHovered ? "hover" : isSelected ? "selected" : "idle"}
      whileHover="hover"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onClick={(e) => {
        e.stopPropagation();
        onSelect();
      }}
    >
      <div className="flex flex-col items-center gap-1">
        <Icon className="w-5 h-5 text-gray-300" />
        <span className="text-xs font-medium text-gray-200 text-center px-1">
          {entity.label}
        </span>
      </div>

      {/* Selection indicator */}
      {isSelected && (
        <motion.div
          className="absolute -top-1 -right-1 w-3 h-3 bg-primary rounded-full"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
        />
      )}
    </motion.div>
  );
}

/**
 * Connection line component
 */
function DemoConnection({
  connection,
  source,
  target,
}: {
  connection: DemoConnection;
  source: DemoEntity;
  target: DemoEntity;
}) {
  const sourcePoint = {
    x: source.position.x + source.size.w / 2,
    y: source.position.y + source.size.h / 2,
  };
  const targetPoint = {
    x: target.position.x + target.size.w / 2,
    y: target.position.y + target.size.h / 2,
  };

  // Calculate midpoint for label
  const midX = (sourcePoint.x + targetPoint.x) / 2;
  const midY = (sourcePoint.y + targetPoint.y) / 2;

  return (
    <g className="connection">
      {/* Main line */}
      <motion.line
        x1={sourcePoint.x}
        y1={sourcePoint.y}
        x2={targetPoint.x}
        y2={targetPoint.y}
        stroke="#4a5568"
        strokeWidth={2}
        strokeDasharray="6,4"
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 1, opacity: 1 }}
        transition={{ duration: 0.5, ease: "easeOut" }}
      />

      {/* Arrow head */}
      <motion.polygon
        points={`${targetPoint.x},${targetPoint.y - 8} ${targetPoint.x + 8},${targetPoint.y} ${targetPoint.x},${targetPoint.y + 8}`}
        fill="#4a5568"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.5 }}
      />

      {/* Label */}
      <motion.g
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.3 }}
      >
        <rect
          x={midX - 50}
          y={midY - 10}
          width={100}
          height={20}
          fill="#1a1a2e"
          rx={4}
        />
        <text
          x={midX}
          y={midY + 4}
          textAnchor="middle"
          fill="#9ca3af"
          fontSize={10}
        >
          {connection.label}
        </text>
      </motion.g>
    </g>
  );
}

/**
 * DemoArchitecture Component
 *
 * Displays a complete AWS C4 architecture diagram with
 * interactive entities and connections.
 */
export function DemoArchitecture({
  autoStart = true,
  className,
}: DemoArchitectureProps) {
  const { camera, zoomIn, zoomOut } = useCamera();
  const { selectedIds, setSelectedIds, clearSelection } = useSelectionStore();

  const [entities, setEntities] = useState<DemoEntity[]>([]);
  const [isAnimating, setIsAnimating] = useState(autoStart);
  const [showInfo, setShowInfo] = useState(true);

  // Initialize demo entities
  useEffect(() => {
    setEntities(demoEntities);
  }, []);

  // Animation loop for data flow
  useEffect(() => {
    if (!isAnimating) return;

    const interval = setInterval(() => {
      // Simulate data flow animation
      const randomConnection =
        demoConnections[Math.floor(Math.random() * demoConnections.length)];
      const source = demoEntities.find(
        (e) => e.id === randomConnection.sourceId,
      );
      const target = demoEntities.find(
        (e) => e.id === randomConnection.targetId,
      );
      if (source && target) {
        // Trigger visual feedback - in real implementation this would animate the connection
      }
    }, 2000);

    return () => clearInterval(interval);
  }, [isAnimating]);

  const handleEntitySelect = useCallback(
    (id: number) => {
      if (selectedIds.includes(id)) {
        clearSelection();
      } else {
        setSelectedIds([id]);
      }
    },
    [selectedIds, setSelectedIds, clearSelection],
  );

  const handleReset = useCallback(() => {
    clearSelection();
    setEntities(demoEntities);
  }, [clearSelection]);

  return (
    <div className={cn("relative w-full h-full bg-[#0d1117]", className)}>
      {/* Header */}
      <div className="absolute top-4 left-4 right-4 flex items-center justify-between z-10">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 px-4 py-2 bg-surface-dark/90 rounded-lg backdrop-blur-sm">
            <Cloud className="w-5 h-5 text-primary" />
            <span className="text-sm font-medium text-gray-200">
              AWS C4 Architecture Demo
            </span>
          </div>
          <div className="flex items-center gap-2 px-3 py-1.5 bg-surface-dark/90 rounded-lg backdrop-blur-sm">
            <Activity className="w-4 h-4 text-green-400" />
            <span className="text-xs text-green-400">
              {isAnimating ? "Simulating Traffic" : "Paused"}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Zoom controls */}
          <div className="flex items-center gap-1 px-2 py-1 bg-surface-dark/90 rounded-lg backdrop-blur-sm">
            <motion.button
              className="p-1.5 rounded hover:bg-white/10"
              variants={buttonVariants}
              initial="idle"
              whileHover="hover"
              whileTap="tap"
              onClick={() => zoomOut()}
            >
              <ZoomOut className="w-4 h-4 text-gray-400" />
            </motion.button>
            <span className="text-xs text-gray-400 w-12 text-center">
              {Math.round(camera.zoom * 100)}%
            </span>
            <motion.button
              className="p-1.5 rounded hover:bg-white/10"
              variants={buttonVariants}
              initial="idle"
              whileHover="hover"
              whileTap="tap"
              onClick={() => zoomIn()}
            >
              <ZoomIn className="w-4 h-4 text-gray-400" />
            </motion.button>
          </div>

          {/* Animation controls */}
          <motion.button
            className="flex items-center gap-2 px-3 py-1.5 bg-surface-dark/90 rounded-lg backdrop-blur-sm"
            variants={buttonVariants}
            initial="idle"
            whileHover="hover"
            whileTap="tap"
            onClick={() => setIsAnimating(!isAnimating)}
          >
            {isAnimating ? (
              <Pause className="w-4 h-4 text-gray-400" />
            ) : (
              <Play className="w-4 h-4 text-gray-400" />
            )}
            <span className="text-xs text-gray-400">
              {isAnimating ? "Pause" : "Resume"}
            </span>
          </motion.button>

          {/* Reset */}
          <motion.button
            className="flex items-center gap-2 px-3 py-1.5 bg-surface-dark/90 rounded-lg backdrop-blur-sm"
            variants={buttonVariants}
            initial="idle"
            whileHover="hover"
            whileTap="tap"
            onClick={handleReset}
          >
            <RefreshCw className="w-4 h-4 text-gray-400" />
            <span className="text-xs text-gray-400">Reset</span>
          </motion.button>

          {/* Info toggle */}
          <motion.button
            className={cn(
              "px-3 py-1.5 rounded-lg backdrop-blur-sm transition-colors",
              showInfo ? "bg-primary/20" : "bg-surface-dark/90",
            )}
            variants={buttonVariants}
            initial="idle"
            whileHover="hover"
            onClick={() => setShowInfo(!showInfo)}
          >
            <span className="text-xs text-gray-400">
              {showInfo ? "Hide Info" : "Show Info"}
            </span>
          </motion.button>
        </div>
      </div>

      {/* Canvas area */}
      <div
        className="absolute inset-0 overflow-hidden"
        style={{
          transform: `scale(${camera.zoom})`,
          transformOrigin: "center center",
        }}
      >
        {/* Grid background */}
        <div
          className="absolute inset-0 opacity-10"
          style={{
            backgroundImage: `
              linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px),
              linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)
            `,
            backgroundSize: "50px 50px",
          }}
        />

        {/* SVG Layer for connections */}
        <svg className="absolute inset-0 w-full h-full pointer-events-none">
          <AnimatePresence>
            {demoConnections.map((connection) => {
              const source = demoEntities.find(
                (e) => e.id === connection.sourceId,
              );
              const target = demoEntities.find(
                (e) => e.id === connection.targetId,
              );
              if (!source || !target) return null;
              return (
                <DemoConnection
                  key={connection.id}
                  connection={connection}
                  source={source}
                  target={target}
                />
              );
            })}
          </AnimatePresence>
        </svg>

        {/* Entity Layer */}
        <AnimatePresence>
          {entities.map((entity) => (
            <DemoEntity
              key={entity.id}
              entity={entity}
              isSelected={selectedIds.includes(entity.id)}
              onSelect={() => handleEntitySelect(entity.id)}
            />
          ))}
        </AnimatePresence>
      </div>

      {/* Info panel */}
      <AnimatePresence>
        {showInfo && (
          <motion.div
            className="absolute bottom-4 left-4 w-72 bg-surface-dark/95 rounded-lg p-4 backdrop-blur-sm border border-white/10"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
          >
            <h3 className="text-sm font-medium text-gray-200 mb-3">
              Architecture Overview
            </h3>
            <div className="space-y-2 text-xs">
              <div className="flex items-center justify-between">
                <span className="text-gray-400">Total Components</span>
                <span className="text-gray-200">{entities.length}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-gray-400">Connections</span>
                <span className="text-gray-200">{demoConnections.length}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-gray-400">Selected</span>
                <span className="text-gray-200">
                  {selectedIds.length > 0 ? selectedIds.length : "None"}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-gray-400">Zoom</span>
                <span className="text-gray-200">
                  {Math.round(camera.zoom * 100)}%
                </span>
              </div>
            </div>

            <div className="mt-4 pt-4 border-t border-white/10">
              <h4 className="text-xs font-medium text-gray-300 mb-2">
                Component Types
              </h4>
              <div className="flex flex-wrap gap-2">
                {[
                  { type: "actor", label: "Users", color: "bg-gray-500" },
                  { type: "compute", label: "Lambda", color: "bg-green-600" },
                  { type: "storage", label: "Storage", color: "bg-yellow-600" },
                  {
                    type: "database",
                    label: "Database",
                    color: "bg-purple-600",
                  },
                ].map(({ type, label, color }) => (
                  <span
                    key={type}
                    className={cn(
                      "px-2 py-0.5 rounded text-xs",
                      color,
                      "text-white",
                    )}
                  >
                    {label}
                  </span>
                ))}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Legend */}
      <motion.div
        className="absolute bottom-4 right-4 bg-surface-dark/95 rounded-lg p-3 backdrop-blur-sm border border-white/10"
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
      >
        <div className="flex items-center gap-4 text-xs">
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-primary" />
            <span className="text-gray-400">Selected</span>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-2 h-2 rounded-full bg-white/30" />
            <span className="text-gray-400">Entity</span>
          </div>
          <div className="flex items-center gap-1.5">
            <div className="w-6 h-0.5 bg-gray-500 border-dashed border-t" />
            <span className="text-gray-400">Connection</span>
          </div>
        </div>
      </motion.div>

      {/* Title */}
      <motion.div
        className="absolute top-4 left-1/2 -translate-x-1/2 text-center"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <h1 className="text-lg font-semibold text-gray-200">
          Microservices Architecture on AWS
        </h1>
        <p className="text-xs text-gray-500 mt-0.5">
          Click on components to view properties • Scroll to zoom
        </p>
      </motion.div>
    </div>
  );
}

export default DemoArchitecture;
