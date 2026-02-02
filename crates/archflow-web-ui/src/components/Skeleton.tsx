/**
 * Skeleton Loading Components
 *
 * Provides loading states for various UI elements.
 * Architecture Reference: EPIC-WEB-007
 */

import React from "react";
import { cn } from "../utils/cn";

interface SkeletonProps {
  className?: string;
  variant?: "text" | "rect" | "circle";
  width?: number | string;
  height?: number | string;
  animation?: "pulse" | "shimmer" | "none";
}

export function Skeleton({
  className,
  variant = "rect",
  width,
  height,
  animation = "pulse",
}: SkeletonProps) {
  const baseStyles = "bg-surface-light/10";

  const variantStyles = {
    text: "rounded h-4",
    rect: "rounded-lg",
    circle: "rounded-full",
  };

  const animationStyles = {
    pulse: "animate-pulse",
    shimmer: "animate-shimmer",
    none: "",
  };

  return (
    <div
      className={cn(
        baseStyles,
        variantStyles[variant],
        animationStyles[animation],
        className,
      )}
      style={{ width, height }}
    />
  );
}

export function EntityCardSkeleton() {
  return (
    <div className="p-3 bg-surface-dark rounded-lg">
      <div className="flex items-center gap-3 mb-3">
        <Skeleton variant="circle" width={32} height={32} />
        <div className="flex-1">
          <Skeleton variant="text" width="60%" />
          <Skeleton variant="text" width="40%" className="mt-2" />
        </div>
      </div>
      <Skeleton variant="text" width="100%" />
      <Skeleton variant="text" width="80%" className="mt-2" />
    </div>
  );
}

export function PropertiesPanelSkeleton() {
  return (
    <div className="p-4 space-y-4">
      <Skeleton variant="text" width="40%" />
      <div className="space-y-3">
        <Skeleton variant="rect" height={40} />
        <Skeleton variant="rect" height={40} />
        <Skeleton variant="rect" height={40} />
      </div>
    </div>
  );
}

export function ToolbarSkeleton() {
  return (
    <div className="flex flex-col gap-1 p-2 bg-surface-dark rounded-lg shadow-lg">
      {[...Array(6)].map((_, i) => (
        <Skeleton key={i} variant="rect" width={40} height={40} />
      ))}
    </div>
  );
}

export function SidebarSkeleton() {
  return (
    <div className="w-64 h-full bg-surface-dark border-r border-white/5 p-3">
      <Skeleton variant="text" width="40%" className="mb-3" />
      <div className="space-y-2">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="flex items-center gap-3 px-3 py-2">
            <Skeleton variant="circle" width={20} height={20} />
            <Skeleton variant="text" width="60%" />
          </div>
        ))}
      </div>
    </div>
  );
}
