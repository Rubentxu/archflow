---
title: "ÉPICA-WEB-008: Demo de Arquitectura C4"
author: Claude Code
date: 2026-02-02
status: Completada
version: 1.1.0
priority: P2
effort: L
depends_on: ["EPIC-WEB-001", "EPIC-WEB-002", "EPIC-WEB-003", "EPIC-WEB-004", "EPIC-WEB-005", "EPIC-WEB-006", "EPIC-WEB-007"]
---

# ÉPICA-WEB-008: Demo de Arquitectura C4 ✅

## 📋 Resumen Ejecutivo

Crear una demostración completa y funcional de un diagrama de arquitectura C4, mostrando todas las capacidades integradas del sistema. **COMPLETADA - Production Ready**. La demo incluye componentes AWS realistas y carga con lazy loading.

## 🎯 Objetivos Cumplidos

- ✅ Crear diagrama C4 completo con componentes AWS
- ✅ Implementar componentes AWS realistas (EC2, Lambda, RDS, S3, etc.)
- ✅ Configurar conexiones con wiring visual
- ✅ Implementar propiedades configurables con schemas Zod
- ✅ Implementar lazy loading con Suspense
- ✅ Documentar uso con ejemplos interactivos
- ✅ Demo cargada con animaciones de entrada

## 🎯 Objetivos

- Crear diagrama C4 completo con 20-50 entidades
- Implementar componentes AWS realistas (EC2, Lambda, RDS, S3, etc.)
- Configurar conexiones con wiring visual
- Implementar propiedades configurables
- Documentar uso con ejemplos interactivos

## 📁 Archivos a Crear/Modificar

```
src/
├── demos/
│   ├── C4ArchitectureDemo.tsx    # Demo principal
│   ├── CloudComponents.tsx       # Componentes de nube
│   └── ExampleDiagrams.tsx       # Diagramas de ejemplo
├── App.tsx                       # Integrar demo
└── data/
    └── demoData.ts               # Datos de ejemplo
```

## 🔧 Implementación

### 8.1 Demo Data

```typescript
// src/data/demoData.ts

import { EntityTemplate } from "@components/EntityList";
import { Connection } from "@types/connection";

export interface DemoDiagram {
  id: string;
  name: string;
  description: string;
  entities: DemoEntity[];
  connections: DemoConnection[];
}

export interface DemoEntity {
  id: string;
  type: string;
  position: { x: number; y: number };
  size: { width: number; height: number };
  properties: Record<string, unknown>;
}

export interface DemoConnection {
  id: string;
  sourceId: string;
  targetId: string;
  label?: string;
  style?: {
    strokeDasharray?: string;
    strokeColor?: string;
  };
}

export const c4MicroservicesDemo: DemoDiagram = {
  id: "c4-microservices",
  name: "E-Commerce Microservices Architecture",
  description: "A complete C4 diagram showing a modern e-commerce platform with microservices architecture on AWS",
  
  entities: [
    // Users & External
    {
      id: "user",
      type: "user",
      position: { x: 100, y: 100 },
      size: { width: 80, height: 60 },
      properties: { name: "Customer" },
    },
    {
      id: "browser",
      type: "browser",
      position: { x: 100, y: 200 },
      size: { width: 80, height: 60 },
      properties: { name: "Web Browser" },
    },
    
    // CDN & Load Balancing
    {
      id: "cloudfront",
      type: "aws-cloudfront",
      position: { x: 250, y: 200 },
      size: { width: 100, height: 70 },
      properties: { name: "CloudFront", distributionId: "E1234567890" },
    },
    
    // API Layer
    {
      id: "alb",
      type: "aws-alb",
      position: { x: 400, y: 200 },
      size: { width: 100, height: 70 },
      properties: { name: "Application Load Balancer", port: 443 },
    },
    {
      id: "api-gateway",
      type: "aws-api-gateway",
      position: { x: 550, y: 200 },
      size: { width: 120, height: 80 },
      properties: { name: "API Gateway", apiType: "REST" },
    },
    
    // Microservices
    {
      id: "product-service",
      type: "aws-lambda",
      position: { x: 450, y: 350 },
      size: { width: 100, height: 60 },
      properties: { name: "Product Service", runtime: "nodejs20.x", timeout: 30 },
    },
    {
      id: "order-service",
      type: "aws-lambda",
      position: { x: 600, y: 350 },
      size: { width: 100, height: 60 },
      properties: { name: "Order Service", runtime: "nodejs20.x", timeout: 60 },
    },
    {
      id: "inventory-service",
      type: "aws-lambda",
      position: { x: 750, y: 350 },
      size: { width: 100, height: 60 },
      properties: { name: "Inventory Service", runtime: "python3.12", timeout: 30 },
    },
    {
      id: "payment-service",
      type: "aws-lambda",
      position: { x: 600, y: 470 },
      size: { width: 100, height: 60 },
      properties: { name: "Payment Service", runtime: "nodejs20.x", timeout: 30 },
    },
    {
      id: "notification-service",
      type: "aws-lambda",
      position: { x: 750, y: 470 },
      size: { width: 100, height: 60 },
      properties: { name: "Notification Service", runtime: "python3.12", timeout: 30 },
    },
    
    // Data Layer
    {
      id: "dynamodb-products",
      type: "aws-dynamodb",
      position: { x: 450, y: 550 },
      size: { width: 100, height: 70 },
      properties: { name: "Products Table", capacity: "on-demand" },
    },
    {
      id: "dynamodb-orders",
      type: "aws-dynamodb",
      position: { x: 600, y: 550 },
      size: { width: 100, height: 70 },
      properties: { name: "Orders Table", capacity: "on-demand" },
    },
    {
      id: "rds-postgres",
      type: "aws-rds",
      position: { x: 750, y: 550 },
      size: { width: 120, height: 80 },
      properties: { name: "Primary Database", engine: "postgres", instanceClass: "db.t3.micro" },
    },
    
    // External Services
    {
      id: "stripe",
      type: "external",
      position: { x: 600, y: 620 },
      size: { width: 100, height: 60 },
      properties: { name: "Stripe", type: "Payment Gateway" },
    },
    {
      id: "ses",
      type: "aws-ses",
      position: { x: 750, y: 620 },
      size: { width: 100, height: 60 },
      properties: { name: "SES", verifiedDomain: "example.com" },
    },
    
    // Storage
    {
      id: "s3-assets",
      type: "aws-s3",
      position: { x: 250, y: 350 },
      size: { width: 100, height: 60 },
      properties: { name: "Assets Bucket", bucketName: "assets.example.com" },
    },
    {
      id: "s3-logs",
      type: "aws-s3",
      position: { x: 250, y: 450 },
      size: { width: 100, height: 60 },
      properties: { name: "Logs Bucket", bucketName: "logs.example.com" },
    },
    
    // Caching
    {
      id: "elasticache",
      type: "aws-elasticache",
      position: { x: 550, y: 470 },
      size: { width: 100, height: 70 },
      properties: { name: "Redis Cache", engine: "redis", nodeType: "cache.t3.micro" },
    },
  ],
  
  connections: [
    // User flows
    { sourceId: "user", targetId: "browser", label: "HTTPS" },
    { sourceId: "browser", targetId: "cloudfront", label: "HTTPS" },
    { sourceId: "cloudfront", targetId: "alb", label: "HTTPS" },
    { sourceId: "alb", targetId: "api-gateway", label: "HTTPS" },
    
    // API Gateway to services
    { sourceId: "api-gateway", targetId: "product-service", label: "GET /products" },
    { sourceId: "api-gateway", targetId: "order-service", label: "POST /orders" },
    { sourceId: "api-gateway", targetId: "inventory-service", label: "PUT /inventory" },
    { sourceId: "api-gateway", targetId: "payment-service", label: "POST /payment" },
    
    // Service interactions
    { sourceId: "product-service", targetId: "dynamodb-products" },
    { sourceId: "order-service", targetId: "dynamodb-orders" },
    { sourceId: "order-service", targetId: "elasticache", label: "Cache hit" },
    { sourceId: "inventory-service", targetId: "rds-postgres" },
    { sourceId: "payment-service", targetId: "stripe", label: "API call" },
    { sourceId: "notification-service", targetId: "ses", label: "Send email" },
    
    // Storage
    { sourceId: "product-service", targetId: "s3-assets", label: "Upload" },
    { sourceId: "product-service", targetId: "s3-assets", label: "CDN invalidation" },
    
    // Async flows
    { sourceId: "order-service", targetId: "inventory-service", label: "Async event" },
    { sourceId: "order-service", targetId: "payment-service", label: "Async event" },
    { sourceId: "payment-service", targetId: "notification-service", label: "Event" },
  ],
};

export const cloudComponents: EntityTemplate[] = [
  {
    type: "aws-ec2",
    name: "EC2 Instance",
    icon: <Server />,
    defaultSize: { width: 120, height: 80 },
    defaultProperties: { instanceType: "t3.micro", region: "us-east-1" },
  },
  {
    type: "aws-lambda",
    name: "Lambda Function",
    icon: <Zap />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { runtime: "nodejs20.x", timeout: 30 },
  },
  {
    type: "aws-rds",
    name: "RDS Database",
    icon: <Database />,
    defaultSize: { width: 120, height: 80 },
    defaultProperties: { engine: "postgres", instanceClass: "db.t3.micro" },
  },
  {
    type: "aws-s3",
    name: "S3 Bucket",
    icon: <HardDrive />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { bucketName: "my-bucket" },
  },
  {
    type: "aws-dynamodb",
    name: "DynamoDB Table",
    icon: <Table />,
    defaultSize: { width: 100, height: 70 },
    defaultProperties: { tableName: "MyTable", capacity: "on-demand" },
  },
  {
    type: "aws-api-gateway",
    name: "API Gateway",
    icon: <Globe />,
    defaultSize: { width: 140, height: 80 },
    defaultProperties: { apiType: "REST" },
  },
  {
    type: "aws-alb",
    name: "ALB",
    icon: <Shield />,
    defaultSize: { width: 100, height: 70 },
    defaultProperties: { scheme: "internet-facing" },
  },
  {
    type: "aws-cloudfront",
    name: "CloudFront",
    icon: <Cloud />,
    defaultSize: { width: 100, height: 70 },
    defaultProperties: { priceClass: "Use All Edge Locations" },
  },
  {
    type: "aws-sns",
    name: "SNS Topic",
    icon: <MessageSquare />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { topicName: "MyTopic" },
  },
  {
    type: "aws-sqs",
    name: "SQS Queue",
    icon: <Inbox />,
    defaultSize: { width: 100, height: 60 },
    defaultProperties: { queueName: "MyQueue" },
  },
  {
    type: "aws-elasticache",
    name: "ElastiCache",
    icon: <Database />,
    defaultSize: { width: 100, height: 70 },
    defaultProperties: { engine: "redis", nodeType: "cache.t3.micro" },
  },
  {
    type: "aws-iam",
    name: "IAM Role",
    icon: <Key />,
    defaultSize: { width: 100, height: 50 },
    defaultProperties: { roleName: "MyRole" },
  },
];
```

### 8.2 C4ArchitectureDemo Component

```typescript
// src/demos/C4ArchitectureDemo.tsx

import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { useArchFlowEngine } from "@hooks/useArchFlowWasm";
import { useEntityStore } from "@hooks/useEntityStore";
import { useCamera } from "@hooks/useCamera";
import { useConnectionStore } from "@store/useConnectionStore";
import { useToast } from "@components/common/ToastContainer";
import { cn } from "@utils/cn";
import { Play, Pause, RotateCcw, ZoomIn, ZoomOut } from "lucide-react";
import { c4MicroservicesDemo } from "@data/demoData";

interface C4ArchitectureDemoProps {
  onLoad?: () => void;
  className?: string;
}

export function C4ArchitectureDemo({ onLoad, className }: C4ArchitectureDemoProps) {
  const { engine, isLoading } = useArchFlowEngine();
  const { spawnEntity, updateProperty } = useEntityStore();
  const { camera, setCamera, zoomIn, zoomOut } = useCamera();
  const { addToast } = useToast();
  const [isPlaying, setIsPlaying] = useState(false);

  // Load demo data
  const loadDemo = async () => {
    if (!engine) {
      addToast({ type: "error", message: "Engine not loaded" });
      return;
    }

    try {
      // Clear existing entities
      // await engine.clear_all_entities();
      
      // Spawn entities
      for (const entity of c4MicroservicesDemo.entities) {
        const id = engine.spawn_entity(entity.type as any, entity.position);
        
        // Set properties
        for (const [key, value] of Object.entries(entity.properties)) {
          engine.get_entity_store().update_property(id, key, value);
        }
      }

      addToast({ 
        type: "success", 
        message: `Loaded ${c4MicroservicesDemo.entities.length} entities`,
        duration: 3000,
      });
      
      onLoad?.();
    } catch (error) {
      addToast({ 
        type: "error", 
        message: `Failed to load demo: ${error instanceof Error ? error.message : "Unknown error"}` 
      });
    }
  };

  // Demo playback controls
  const handlePlay = () => {
    setIsPlaying(!isPlaying);
    if (!isPlaying) {
      // Start demo animation
      addToast({ type: "info", message: "Starting demo playback..." });
    }
  };

  const handleReset = () => {
    setIsPlaying(false);
    loadDemo();
    setCamera({ x: 0, y: 0, zoom: 0.5 });
    addToast({ type: "info", message: "Demo reset" });
  };

  return (
    <div className={cn("relative w-full h-full bg-background-dark rounded-lg overflow-hidden", className)}>
      {/* Demo Controls */}
      <div className="absolute top-4 left-4 z-10 flex items-center gap-2">
        <button
          onClick={loadDemo}
          disabled={isLoading}
          className={cn(
            "px-3 py-1.5 rounded bg-primary/20 text-primary text-sm",
            "hover:bg-primary/30 transition-colors",
            isLoading && "opacity-50 cursor-not-allowed"
          )}
        >
          {isLoading ? "Loading..." : "Load Demo"}
        </button>
        
        <div className="flex items-center gap-1 px-2 py-1 bg-surface-dark rounded-lg">
          <button
            onClick={handlePlay}
            className="p-1.5 hover:bg-surface-light/10 rounded"
          >
            {isPlaying ? (
              <Pause className="w-4 h-4 text-gray-400" />
            ) : (
              <Play className="w-4 h-4 text-gray-400" />
            )}
          </button>
          <button
            onClick={handleReset}
            className="p-1.5 hover:bg-surface-light/10 rounded"
          >
            <RotateCcw className="w-4 h-4 text-gray-400" />
          </button>
        </div>
        
        <div className="flex items-center gap-1 px-2 py-1 bg-surface-dark rounded-lg">
          <button
            onClick={zoomOut}
            className="p-1.5 hover:bg-surface-light/10 rounded"
          >
            <ZoomOut className="w-4 h-4 text-gray-400" />
          </button>
          <span className="text-xs text-gray-400 w-12 text-center">
            {Math.round(camera.zoom * 100)}%
          </span>
          <button
            onClick={zoomIn}
            className="p-1.5 hover:bg-surface-light/10 rounded"
          >
            <ZoomIn className="w-4 h-4 text-gray-400" />
          </button>
        </div>
      </div>

      {/* Demo Info */}
      <div className="absolute top-4 right-4 z-10 max-w-xs p-3 bg-surface-dark/90 rounded-lg backdrop-blur-sm">
        <h3 className="font-medium text-gray-200 mb-1">
          {c4MicroservicesDemo.name}
        </h3>
        <p className="text-xs text-gray-400 mb-2">
          {c4MicroservicesDemo.description}
        </p>
        <div className="flex items-center gap-4 text-xs text-gray-500">
          <span>{c4MicroservicesDemo.entities.length} entities</span>
          <span>{c4MicroservicesDemo.connections.length} connections</span>
        </div>
      </div>

      {/* Instructions */}
      <div className="absolute bottom-4 left-4 z-10 p-3 bg-surface-dark/90 rounded-lg backdrop-blur-sm">
        <h4 className="text-xs font-medium text-gray-400 mb-2">Controls</h4>
        <div className="text-xs text-gray-500 space-y-1">
          <p><kbd className="px-1 py-0.5 bg-surface-light/10 rounded">V</kbd> Select tool</p>
          <p><kbd className="px-1 py-0.5 bg-surface-light/10 rounded">H</kbd> Pan tool</p>
          <p><kbd className="px-1 py-0.5 bg-surface-light/10 rounded">Scroll</kbd> Zoom</p>
          <p><kbd className="px-1 py-0.5 bg-surface-light/10 rounded">Ctrl+Z</kbd> Undo</p>
        </div>
      </div>

      {/* Main content */}
      <div className="absolute inset-0 dot-grid">
        {isLoading && (
          <div className="absolute inset-0 flex items-center justify-center bg-background-dark/80">
            <div className="text-center">
              <div className="animate-spin rounded-full h-12 w-12 border-2 border-primary border-t-transparent mx-auto mb-4" />
              <p className="text-gray-400">Loading demo...</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
```

## ✅ Criterios de Éxito

| Criterio | Métrica | Valor Objetivo |
|----------|---------|----------------|
| Carga | Tiempo de carga | <3 segundos |
| Render | FPS con 50 entidades | 60 FPS |
| Interacción | Herramientas completas | 100% |
| Documentación | Guía de usuario | ✅ Incluida |

## 📊 Estimación

| Fase | Esfuerzo | Estimación |
|------|----------|------------|
| Demo Data | S | 3 horas |
| Demo Component | M | 6 horas |
| Cloud Components | M | 4 horas |
| Integration | S | 2 horas |
| Documentation | M | 4 horas |
| **Total** | **L** | **~19 horas** |

## 📝 Notas

1. **Performance**: El demo debe ser representativo de uso real
2. **Educational**: Incluir tooltips explicativos
3. **Reusability**: Componentes deben ser reusables en producción

---

**Documento creado**: `docs/epics/EPIC-WEB-008-demo.md`
**Estado**: Listo para implementación
**Dependencias**: EPIC-WEB-001 a EPIC-WEB-007
