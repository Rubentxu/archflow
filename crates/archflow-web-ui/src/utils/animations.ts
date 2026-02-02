/**
 * Framer Motion Animation Configuration
 *
 * Global animation configurations, variants, and presets
 * for consistent motion design across the application.
 */

/**
 * Standard transition presets
 */
export const transitions = {
  /** Quick interaction response */
  quick: { type: "spring", stiffness: 500, damping: 30 },
  /** Smooth panel movement */
  smooth: { type: "spring", stiffness: 300, damping: 25 },
  /** Slow, elegant transition */
  elegant: { type: "spring", stiffness: 200, damping: 20 },
  /** Fade transition */
  fade: { duration: 0.2, ease: "easeOut" },
};

/**
 * Entity hover and selection variants
 */
export const entityVariants = {
  idle: { scale: 1, filter: "brightness(1)" },
  hover: { scale: 1.02, filter: "brightness(1.1)" },
  selected: { scale: 1, filter: "brightness(1.15)" },
  dragging: { scale: 1.05, filter: "brightness(1.2)" },
};

/**
 * Button interaction variants
 */
export const buttonVariants = {
  idle: { scale: 1 },
  hover: { scale: 1.05 },
  tap: { scale: 0.95 },
};

/**
 * Tooltip animation variants
 */
export const tooltipVariants = {
  hidden: { opacity: 0, y: 4, scale: 0.95 },
  visible: { opacity: 1, y: 0, scale: 1 },
  exit: { opacity: 0, y: 2, scale: 0.95 },
};

/**
 * Panel slide variants
 */
export const panelVariants = {
  slideInRight: {
    x: ["100%", "0%"],
    opacity: [0, 1],
    transition: { duration: 0.3, ease: "easeOut" },
  },
  slideInLeft: {
    x: ["-100%", "0%"],
    opacity: [0, 1],
    transition: { duration: 0.3, ease: "easeOut" },
  },
};

/**
 * Connection line animation variants
 */
export const connectionVariants = {
  idle: { strokeWidth: 2, filter: "brightness(1)" },
  hover: { strokeWidth: 3, filter: "brightness(1.2)" },
};

/**
 * Snap preview indicator variants
 */
export const snapIndicatorVariants = {
  hidden: { opacity: 0, scale: 0 },
  visible: { opacity: 1, scale: 1 },
};

/**
 * Handle resize animation variants
 */
export const handleVariants = {
  idle: { scale: 1, opacity: 0.8 },
  hover: { scale: 1.2, opacity: 1 },
};

/**
 * Toolbar tool active state
 */
export const toolVariants = {
  inactive: { scale: 1 },
  active: { scale: 1.05 },
};

/**
 * Zoom control button variants
 */
export const zoomButtonVariants = {
  idle: { scale: 1 },
  hover: { scale: 1.1 },
  tap: { scale: 0.9 },
};

/**
 * Drag handle animation
 */
export const dragHandleVariants = {
  hidden: { opacity: 0, scale: 0.8 },
  visible: { opacity: 1, scale: 1 },
};

/**
 * Context menu animation
 */
export const contextMenuVariants = {
  hidden: { opacity: 0, scale: 0.95, y: -8 },
  visible: { opacity: 1, scale: 1, y: 0 },
  exit: { opacity: 0, scale: 0.95, y: -8 },
};

/**
 * Modal/overlay backdrop variants
 */
export const backdropVariants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1 },
  exit: { opacity: 0 },
};

/**
 * Modal content animation
 */
export const modalVariants = {
  hidden: { opacity: 0, scale: 0.9, y: 20 },
  visible: { opacity: 1, scale: 1, y: 0 },
  exit: { opacity: 0, scale: 0.9, y: 20 },
};

/**
 * Create a staggered list animation
 */
export function createStaggeredVariants(staggerTime = 0.05) {
  return {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: {
        staggerChildren: staggerTime,
      },
    },
  };
}

/**
 * Child variants for staggered animations
 */
export const staggeredChildVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

/**
 * Connection point pulse animation
 */
export const connectionPointPulse = {
  idle: { scale: 1, opacity: 0.6 },
  hover: { scale: 1.3, opacity: 1 },
};

/**
 * Selection marquee animation
 */
export const marqueeVariants = {
  idle: { opacity: 0.3, scale: 1 },
  drawing: { opacity: 0.5 },
};
