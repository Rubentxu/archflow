import React from "react";
import {
    Zap,
    Activity,
    Cpu,
    MousePointer2,
    Keyboard,
    Wifi,
    PlayCircle,
    Plus
} from "lucide-react";
import { cn } from "../../utils/cn";

interface LogicPanelProps {
    entityId: number | null;
}

export function LogicPanel({ entityId }: LogicPanelProps) {
    if (!entityId) return null;

    return (
        <div className="flex flex-col h-full overflow-hidden bg-surface-light dark:bg-surface-dark">
            {/* Toolbar */}
            <div className="flex items-center justify-between p-3 border-b border-border-light dark:border-border-dark">
                <h4 className="text-xs font-bold uppercase text-slate-500">Logic Bricks</h4>
                <button className="p-1 rounded hover:bg-black/5 dark:hover:bg-white/5 text-primary transition-colors">
                    <Plus className="w-4 h-4" />
                </button>
            </div>

            <div className="flex-1 overflow-y-auto p-3 space-y-6">
                {/* Sensors Section */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
                        <Activity className="w-3.5 h-3.5" />
                        <span>SENSORS</span>
                    </div>

                    <div className="space-y-2">
                        {/* Mock Sensor */}
                        <div className="bg-white dark:bg-[#1e293b] border border-border-light dark:border-white/10 rounded-lg p-3 shadow-sm relative overflow-hidden group">
                            <div className="absolute left-0 top-0 bottom-0 w-1 bg-yellow-500"></div>
                            <div className="flex justify-between items-start">
                                <div className="flex items-center gap-2">
                                    <MousePointer2 className="w-4 h-4 text-yellow-500" />
                                    <span className="text-sm font-medium dark:text-gray-200">Mouse Over</span>
                                </div>
                                <div className="h-2 w-2 rounded-full bg-yellow-500 animate-pulse"></div>
                            </div>
                            <div className="mt-2 flex items-center gap-2">
                                <span className="text-[10px] font-mono bg-black/5 dark:bg-white/5 px-1.5 py-0.5 rounded text-slate-500">
                                    True
                                </span>
                                <div className="flex-1 h-px bg-border-light dark:bg-white/10 border-t border-dashed"></div>
                                <div className="w-2 h-2 bg-slate-300 dark:bg-slate-600 rounded-full"></div>
                            </div>
                        </div>

                        <div className="bg-white dark:bg-[#1e293b] border border-border-light dark:border-white/10 rounded-lg p-3 shadow-sm relative overflow-hidden opacity-75">
                            <div className="absolute left-0 top-0 bottom-0 w-1 bg-yellow-500 opacity-50"></div>
                            <div className="flex justify-between items-start">
                                <div className="flex items-center gap-2">
                                    <Keyboard className="w-4 h-4 text-yellow-500" />
                                    <span className="text-sm font-medium dark:text-gray-200">Key: Space</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Controllers Section */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
                        <Cpu className="w-3.5 h-3.5" />
                        <span>CONTROLLERS</span>
                    </div>

                    <div className="space-y-2">
                        <div className="bg-white dark:bg-[#1e293b] border border-border-light dark:border-white/10 rounded-lg p-3 shadow-sm relative overflow-hidden">
                            <div className="absolute left-0 top-0 bottom-0 w-1 bg-blue-500"></div>
                            <div className="flex justify-between items-center mb-2">
                                <div className="w-2 h-2 bg-slate-300 dark:bg-slate-600 rounded-full"></div>
                                <span className="text-xs font-bold text-blue-500">AND</span>
                                <div className="w-2 h-2 bg-slate-300 dark:bg-slate-600 rounded-full"></div>
                            </div>
                            <div className="text-center">
                                <span className="text-xs text-slate-500 dark:text-slate-400">All inputs active</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Actuators Section */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
                        <Zap className="w-3.5 h-3.5" />
                        <span>ACTUATORS</span>
                    </div>

                    <div className="space-y-2">
                        <div className="bg-white dark:bg-[#1e293b] border border-border-light dark:border-white/10 rounded-lg p-3 shadow-sm relative overflow-hidden">
                            <div className="absolute left-0 top-0 bottom-0 w-1 bg-green-500"></div>
                            <div className="flex items-center gap-2 mb-2">
                                <div className="w-2 h-2 bg-slate-300 dark:bg-slate-600 rounded-full"></div>
                                <PlayCircle className="w-4 h-4 text-green-500" />
                                <span className="text-sm font-medium dark:text-gray-200">Motion</span>
                            </div>
                            <div className="pl-4">
                                <div className="text-[10px] text-slate-500">Rot: 45deg</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
