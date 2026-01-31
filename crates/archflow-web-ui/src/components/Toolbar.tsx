import { useState } from 'react'

interface ToolbarProps {
  onZoomIn?: () => void
  onZoomOut?: () => void
  onRun?: () => void
}

export default function Toolbar({ onZoomIn, onZoomOut, onRun }: ToolbarProps) {
  const [activeTool, setActiveTool] = useState('select')
  const [zoom, setZoom] = useState(100)

  const tools = [
    { id: 'select', icon: 'near_me', label: 'Select' },
    { id: 'pan', icon: 'pan_tool', label: 'Pan' },
    { id: 'draw', icon: 'edit', label: 'Draw' },
    { id: 'shape', icon: 'crop_square', label: 'Shape' },
  ]

  const handleZoomIn = () => {
    const newZoom = Math.min(zoom + 25, 200)
    setZoom(newZoom)
    onZoomIn?.()
  }

  const handleZoomOut = () => {
    const newZoom = Math.max(zoom - 25, 25)
    setZoom(newZoom)
    onZoomOut?.()
  }

  return (
    <div className="absolute top-4 left-4 z-10 flex items-center gap-2 bg-surface-light dark:bg-surface-dark rounded-lg shadow-lg border border-border-light dark:border-border-dark p-2">
      {/* Tools */}
      <div className="flex items-center gap-1 pr-2 border-r border-border-light dark:border-border-dark">
        {tools.map((tool) => (
          <button
            key={tool.id}
            onClick={() => setActiveTool(tool.id)}
            className={`p-2 rounded transition-colors ${
              activeTool === tool.id
                ? 'bg-primary text-white'
                : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700'
            }`}
            title={tool.label}
          >
            <span className="material-symbols-outlined text-lg">{tool.icon}</span>
          </button>
        ))}
      </div>

      {/* Undo/Redo */}
      <div className="flex items-center gap-1 pr-2 border-r border-border-light dark:border-border-dark">
        <button className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded transition-colors">
          <span className="material-symbols-outlined text-lg">undo</span>
        </button>
        <button className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded transition-colors">
          <span className="material-symbols-outlined text-lg">redo</span>
        </button>
      </div>

      {/* Zoom Controls */}
      <div className="flex items-center gap-1">
        <button
          onClick={handleZoomOut}
          className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded transition-colors"
        >
          <span className="material-symbols-outlined text-lg">remove</span>
        </button>
        <span className="text-sm text-slate-600 dark:text-slate-300 font-medium w-12 text-center">{zoom}%</span>
        <button
          onClick={handleZoomIn}
          className="p-2 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 rounded transition-colors"
        >
          <span className="material-symbols-outlined text-lg">add</span>
        </button>
      </div>

      {/* Run Button */}
      <button
        onClick={onRun}
        className="flex items-center gap-1 px-3 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
      >
        <span className="material-symbols-outlined">play_arrow</span>
        <span>Run</span>
      </button>
    </div>
  )
}
