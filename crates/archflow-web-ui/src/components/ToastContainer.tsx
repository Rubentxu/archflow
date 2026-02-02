/**
 * Toast Container Component
 *
 * Displays toast notifications with animations.
 * Architecture Reference: EPIC-WEB-007
 */

import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useToastStore, ToastType } from "../store/useToastStore";
import { cn } from "../utils/cn";
import {
  CheckCircle,
  AlertCircle,
  AlertTriangle,
  Info,
  X,
} from "lucide-react";

const toastIcons: Record<ToastType, React.ReactNode> = {
  success: <CheckCircle className="w-5 h-5 text-green-500" />,
  error: <AlertCircle className="w-5 h-5 text-red-500" />,
  warning: <AlertTriangle className="w-5 h-5 text-yellow-500" />,
  info: <Info className="w-5 h-5 text-blue-500" />,
};

const toastStyles: Record<ToastType, string> = {
  success: "border-green-500/30 bg-green-500/10",
  error: "border-red-500/30 bg-red-500/10",
  warning: "border-yellow-500/30 bg-yellow-500/10",
  info: "border-blue-500/30 bg-blue-500/10",
};

export function ToastContainer() {
  const { toasts, removeToast } = useToastStore();

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
      <AnimatePresence>
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            initial={{
              opacity: 0,
              y: 20,
              scale: 0.9,
            }}
            animate={{
              opacity: 1,
              y: 0,
              scale: 1,
            }}
            exit={{
              opacity: 0,
              x: 100,
              scale: 0.9,
            }}
            transition={{
              type: "spring",
              stiffness: 400,
              damping: 25,
            }}
            className={cn(
              "pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg",
              "border backdrop-blur-sm shadow-lg",
              toastStyles[toast.type],
              toast.type === "error" && "border-red-500/50",
            )}
          >
            {toastIcons[toast.type]}
            <p className="text-sm text-gray-200 flex-1">{toast.message}</p>
            {toast.action && (
              <button
                onClick={toast.action.onClick}
                className="text-xs text-primary hover:underline"
              >
                {toast.action.label}
              </button>
            )}
            <button
              onClick={() => removeToast(toast.id)}
              className="p-1 hover:bg-surface-light/10 rounded"
            >
              <X className="w-4 h-4 text-gray-400" />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}

export function useToast() {
  const addToast = useToastStore((state) => state.addToast);

  return {
    success: (message: string, options?: Partial<Omit<Toast, "id">>) =>
      addToast({ message, type: "success", ...options }),
    error: (message: string, options?: Partial<Omit<Toast, "id">>) =>
      addToast({ message, type: "error", ...options }),
    warning: (message: string, options?: Partial<Omit<Toast, "id">>) =>
      addToast({ message, type: "warning", ...options }),
    info: (message: string, options?: Partial<Omit<Toast, "id">>) =>
      addToast({ message, type: "info", ...options }),
  };
}
