import React from "react";
import { Clock, Activity, History } from "lucide-react";
import { cn } from "../../utils/cn";

interface HistoryPanelProps {
    entityId: number | null;
}

export function HistoryPanel({ entityId }: HistoryPanelProps) {
    if (!entityId) return null;

    // Mock data simulates SignalByte history
    const ticks = Array.from({ length: 6 }).map((_, i) => ({
        id: i,
        active: Math.random() > 0.5,
        timestamp: Date.now() - i * 16,
    }));

    return (
        <div className="flex flex-col h-full overflow-hidden bg-surface-light dark:bg-surface-dark">
            {/* Signal Visualizer */}
            <div className="p-4 border-b border-border-light dark:border-border-dark bg-slate-50/50 dark:bg-black/20">
                <div className="flex items-center gap-2 mb-3">
                    <Activity className="w-4 h-4 text-primary" />
                    <h4 className="text-xs font-bold uppercase text-slate-500">Signal Byte (6 Ticks)</h4>
                </div>

                <div className="flex justify-between gap-1">
                    {ticks.reverse().map((tick, i) => (
                        <div key={tick.id} className="flex flex-col items-center gap-1 group">
                            <div className={cn(
                                "w-8 h-12 rounded-sm transition-all duration-300 flex items-end justify-center pb-1",
                                tick.active
                                    ? "bg-primary shadow-[0_0_8px_rgba(14,165,233,0.5)]"
                                    : "bg-slate-200 dark:bg-slate-800"
                            )}>
                                <span className="text-[9px] font-mono text-white/80">{tick.active ? '1' : '0'}</span>
                            </div>
                            <span className="text-[9px] text-slate-400 font-mono">T-{5 - i}</span>
                        </div>
                    ))}
                </div>

                <div className="mt-4 flex justify-between text-[10px] text-slate-400 font-mono">
                    <span>State: {ticks[5].active ? "ACTIVE" : "IDLE"}</span>
                    <span>Freq: 60Hz</span>
                </div>
            </div>

            {/* Event Log */}
            <div className="flex-1 overflow-y-auto p-0">
                <div className="p-2 bg-slate-100 dark:bg-slate-900/50 sticky top-0 border-b border-border-light dark:border-border-dark">
                    <div className="flex items-center gap-2">
                        <History className="w-3.5 h-3.5 text-slate-500" />
                        <span className="text-xs font-semibold text-slate-500">EVENT LOG</span>
                    </div>
                </div>

                <div className="divide-y divide-border-light dark:divide-white/5">
                    {[1, 2, 3, 4, 5].map((i) => (
                        <div key={i} className="p-3 hover:bg-black/5 dark:hover:bg-white/5 transition-colors">
                            <div className="flex justify-between items-start mb-1">
                                <span className={cn(
                                    "text-xs font-bold",
                                    i % 2 === 0 ? "text-green-500" : "text-blue-500"
                                )}>
                                    {i % 2 === 0 ? "ACTUATOR_EXEC" : "SENSOR_RISING_EDGE"}
                                </span>
                                <span className="text-[10px] text-slate-400 font-mono">15:30:22.{100 + i}</span>
                            </div>
                            <p className="text-xs text-slate-600 dark:text-slate-400">
                                {i % 2 === 0 ? "Highlight effect triggered" : "Mouse detected over entity"}
                            </p>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
