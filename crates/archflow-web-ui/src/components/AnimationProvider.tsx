/**
 * Animation Provider Component
 *
 * Wraps the application with Framer Motion for global animation support
 * and provides animation context.
 *
 * Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
 */

import React from "react";
import { MotionConfig, type Transition } from "framer-motion";
import { transitions } from "../utils/animations";

/**
 * Props for AnimationProvider
 */
interface AnimationProviderProps {
  children: React.ReactNode;
  /** Default transition for all animations */
  defaultTransition?: Transition;
  /** Reduced motion mode */
  reducedMotion?: "always" | "never";
}

/**
 * Animation Provider Component
 *
 * Provides Framer Motion context with custom default transitions
 * and reduced motion support.
 */
export function AnimationProvider({
  children,
  defaultTransition = transitions.smooth,
  reducedMotion = "never",
}: AnimationProviderProps) {
  return (
    <MotionConfig transition={defaultTransition} reducedMotion={reducedMotion}>
      {children}
    </MotionConfig>
  );
}
