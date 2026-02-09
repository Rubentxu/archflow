import React, { useEffect } from 'react';
import { useEntityStore } from '../../hooks/useEntityStore';
import { useSelectionStore } from '../../store/useSelectionStore';
import { useArchFlowWasm } from '../../hooks/useArchFlowWasm';
import { cn } from '../../utils/cn';
import {
    Box,
    Circle,
    Triangle,
    Diamond,
    Star,
    Cloud,
    Zap,
    Heart,
    HelpCircle,
    Plus,
    Minus,
    MessageSquare,
    Link,
    Trash2,
    Hexagon,
    X,
    AlertTriangle,
    Divide,
} from 'lucide-react';
import { ShapeType } from '../../types/wasm';

export function ShapeHistory() {
    const { entities, refreshEntities } = useEntityStore();
    const { selectedIds } = useSelectionStore(); // Only read selectedIds
    const { bridge } = useArchFlowWasm();

    // Poll for changes (temporary solution until we have event bus)
    useEffect(() => {
        refreshEntities(); // Initial fetch
        const interval = setInterval(refreshEntities, 1000);
        return () => clearInterval(interval);
    }, [refreshEntities]);

    const handleSelect = (id: number, event: React.MouseEvent) => {
        if (!bridge) return;

        const isMulti = event.metaKey || event.ctrlKey;
        const isShift = event.shiftKey;

        if (isShift) {
            // Shift + Click: Add to selection (don't toggle)
            bridge.set_entity_selected(id, true);
        } else if (isMulti) {
            // Cmd/Ctrl + Click: Toggle selection
            const isSelected = bridge.is_entity_selected(id);
            bridge.set_entity_selected(id, !isSelected);
        } else {
            // Simple Click: Select only this
            bridge.clear_selection();
            bridge.set_entity_selected(id, true);
        }
    };

    const sortedEntities = Array.from(entities.values())
        .sort((a, b) => b.id - a.id); // Newest first

    const getIcon = (shapeType: number) => {
        switch (shapeType) {
            case ShapeType.Rectangle:
                return <Box className="w-4 h-4" />;
            case ShapeType.Circle:
                return <Circle className="w-4 h-4" />;
            case ShapeType.Triangle:
                return <Triangle className="w-4 h-4" />;
            case ShapeType.Diamond:
                return <Diamond className="w-4 h-4" />;
            case ShapeType.Star:
                return <Star className="w-4 h-4" />;
            case ShapeType.Cloud:
                return <Cloud className="w-4 h-4" />;
            case ShapeType.Lightning:
                return <Zap className="w-4 h-4" />;
            case ShapeType.Heart:
                return <Heart className="w-4 h-4" />;
            case ShapeType.Question:
                return <HelpCircle className="w-4 h-4" />;
            case ShapeType.Plus:
                return <Plus className="w-4 h-4" />;
            case ShapeType.Minus:
                return <Minus className="w-4 h-4" />;
            case ShapeType.Hexagon:
                return <Hexagon className="w-4 h-4" />;
            case ShapeType.Pentagon:
                return <Hexagon className="w-4 h-4 rotate-[18deg]" />; // Fallback
            case ShapeType.Multiply:
                return <X className="w-4 h-4" />;
            case ShapeType.Exclamation:
                return <AlertTriangle className="w-4 h-4" />;
            case ShapeType.Divide:
                return <Divide className="w-4 h-4" />;
            default:
                return <Box className="w-4 h-4" />;
        }
    };

    return (
        <div className="flex flex-col h-1/3 min-h-[200px] border-t border-border-light dark:border-border-dark bg-slate-50 dark:bg-black/20 shrink-0">
            {/* Header ... */}
            <div className="px-4 py-3 border-b border-border-light dark:border-border-dark flex items-center justify-between bg-surface-light dark:bg-surface-dark">
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse" />
                    <h3 className="text-xs font-bold uppercase tracking-wider text-slate-500 dark:text-gray-400">History & Shapes</h3>
                </div>
                <span className="text-[10px] font-mono text-slate-400 bg-slate-100 dark:bg-white/5 px-1.5 py-0.5 rounded">
                    {entities.size}
                </span>
            </div>
            <div className="flex-1 overflow-y-auto p-2 space-y-1">
                {sortedEntities.map(entity => (
                    <div
                        key={entity.id}
                        onClick={(e) => handleSelect(entity.id, e)}
                        className={cn(
                            "group flex items-center gap-3 p-2 rounded-md cursor-pointer text-sm transition-all duration-200 border border-transparent select-none",
                            selectedIds.includes(entity.id)
                                ? "bg-primary/10 text-primary border-primary/20 shadow-sm"
                                : "text-slate-600 dark:text-gray-400 hover:bg-white dark:hover:bg-white/5 hover:border-slate-200 dark:hover:border-white/10 hover:shadow-sm"
                        )}
                    >
                        <div className={cn(
                            "p-1.5 rounded-md flex items-center justify-center transition-colors",
                            selectedIds.includes(entity.id) ? "bg-primary/20 text-primary" : "bg-slate-200 dark:bg-white/10 text-slate-500"
                        )}>
                            {getIcon(entity.shape)}
                        </div>

                        <div className="flex flex-col min-w-0 flex-1">
                            <span className="font-medium text-xs truncate">
                                {entity.label || `Shape ${entity.id}`}
                            </span>
                            <div className="flex items-center gap-2 mt-0.5">
                                <span className="text-[10px] text-slate-400 font-mono">#{entity.id}</span>
                                {entity.type && (
                                    <span className="text-[9px] px-1 py-0.5 rounded bg-slate-100 dark:bg-white/5 text-slate-500">
                                        {entity.type}
                                    </span>
                                )}
                            </div>
                        </div>

                        <div className={cn(
                            "w-1.5 h-1.5 rounded-full transition-colors",
                            entity.isVisible ? "bg-green-400" : "bg-gray-300"
                        )} />
                    </div>
                ))}
                {sortedEntities.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-full text-slate-400 gap-3 py-8">
                        <div className="p-3 rounded-full bg-slate-100 dark:bg-white/5">
                            <Box className="w-5 h-5 opacity-40" />
                        </div>
                        <p className="text-xs font-medium">No shapes created yet</p>
                    </div>
                )}
            </div>
        </div>
    );
}
