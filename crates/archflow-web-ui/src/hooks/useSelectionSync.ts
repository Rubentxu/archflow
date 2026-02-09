import { useEffect, useRef } from "react";
import { useSelectionStore } from "../store/useSelectionStore";
import { useArchFlowWasm } from "./useArchFlowWasm.tsx";

/**
 * Hook to synchronize selection state between React and WASM bridge.
 * 
 * Ensures that whenever the engine's internal selection changes (e.g. via 
 * canvas interaction or logic bricks), the React UI state is updated 
 * to reflect it (showing gizmos, property panels, etc).
 */
export function useSelectionSync() {
    const { bridge, isInitialized } = useArchFlowWasm();
    const { setSelectedIds } = useSelectionStore();

    // Use a string representation to avoid array reference issues and unnecessary updates
    const lastSelectionRef = useRef<string>("");

    useEffect(() => {
        if (!bridge || !isInitialized) return;

        const syncSelection = () => {
            try {
                // Ensure method exists on bridge before calling
                if (typeof bridge.get_selection !== 'function') return;

                // Fetch selection from bridge (source of truth is the Engine)
                // get_selection returns a JS Array or Uint32Array of entity IDs
                const rawSelection = bridge.get_selection();
                const selection = Array.from(rawSelection).map(id => Number(id));

                // Sort for stable comparison string
                const sorted = [...selection].sort((a, b) => a - b);
                const selectionStr = sorted.join(",");

                // Only update store if the content has actually changed
                if (selectionStr !== lastSelectionRef.current) {
                    // console.log("[Sync] Updating React selection from Bridge:", selection);
                    setSelectedIds(selection);
                    lastSelectionRef.current = selectionStr;
                }
            } catch (err) {
                // Silent catch for sync errors to prevent console spam
                // console.warn("[Sync] Selection sync error:", err);
            }
        };

        // Initial sync
        syncSelection();

        // Poll for changes every 100ms
        // This is a robust way to catch engine changes without complex event subscriptions
        const interval = setInterval(syncSelection, 100);

        return () => clearInterval(interval);
    }, [bridge, isInitialized, setSelectedIds]);
}
