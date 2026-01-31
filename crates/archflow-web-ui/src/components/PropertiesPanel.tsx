import { useState } from 'react'

interface PropertiesPanelProps {
  selectedEntity: number | null
}

interface EntityProps {
  type: string
  x: number
  y: number
  width: number
  height: number
  fillColor: string
  strokeColor: string
  borderWidth: number
}

const entityTypes = [
  { value: 'aws-ec2', label: 'AWS EC2' },
  { value: 'aws-rds', label: 'AWS RDS' },
  { value: 'aws-s3', label: 'AWS S3' },
  { value: 'aws-lambda', label: 'AWS Lambda' },
  { value: 'database', label: 'Database' },
  { value: 'queue', label: 'Queue' },
]

export default function PropertiesPanel({ selectedEntity }: PropertiesPanelProps) {
  const [props, setProps] = useState<EntityProps>({
    type: 'aws-ec2',
    x: 100,
    y: 100,
    width: 120,
    height: 80,
    fillColor: '#FF6B6B',
    strokeColor: '#13b6ec',
    borderWidth: 2,
  })

  const updateProp = <K extends keyof EntityProps>(key: K, value: EntityProps[K]) => {
    setProps((prev) => ({ ...prev, [key]: value }))
  }

  if (selectedEntity === null) {
    return (
      <aside className="w-72 border-l border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
        <div className="p-6 text-center text-slate-500 dark:text-slate-400">
          <span className="material-symbols-outlined text-4xl mb-2">select</span>
          <p className="text-sm">Select an entity to view its properties</p>
        </div>
      </aside>
    )
  }

  return (
    <aside className="w-72 border-l border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
      <div className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-slate-900 dark:text-white">Properties</h3>
          <button className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200">
            <span className="material-symbols-outlined">more_vert</span>
          </button>
        </div>

        {/* Entity Type */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Type
          </label>
          <select
            value={props.type}
            onChange={(e) => updateProp('type', e.target.value)}
            className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          >
            {entityTypes.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>

        {/* Position */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Position
          </label>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">X</label>
              <input
                type="number"
                value={props.x}
                onChange={(e) => updateProp('x', Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Y</label>
              <input
                type="number"
                value={props.y}
                onChange={(e) => updateProp('y', Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Size */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-1">
            Size
          </label>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Width</label>
              <input
                type="number"
                value={props.width}
                onChange={(e) => updateProp('width', Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">Height</label>
              <input
                type="number"
                value={props.height}
                onChange={(e) => updateProp('height', Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Style */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
            Style
          </label>
          <div className="space-y-2">
            <div>
              <label className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
                <input
                  type="color"
                  value={props.fillColor}
                  onChange={(e) => updateProp('fillColor', e.target.value)}
                  className="w-8 h-8 rounded cursor-pointer"
                />
                Fill
              </label>
            </div>
            <div>
              <label className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
                <input
                  type="color"
                  value={props.strokeColor}
                  onChange={(e) => updateProp('strokeColor', e.target.value)}
                  className="w-8 h-8 rounded cursor-pointer"
                />
                Stroke
              </label>
            </div>
            <div>
              <label className="block text-xs text-slate-500 dark:text-slate-400 mb-1">
                Border Width
              </label>
              <input
                type="number"
                min="0"
                max="10"
                value={props.borderWidth}
                onChange={(e) => updateProp('borderWidth', Number(e.target.value))}
                className="w-full px-3 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
              />
            </div>
          </div>
        </div>

        {/* Logic Bricks Section */}
        <div className="border-t border-border-light dark:border-border-dark pt-4">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-sm font-semibold text-slate-900 dark:text-white">Logic Bricks</h4>
            <button className="text-primary hover:text-primary/80 text-xs font-medium flex items-center gap-1">
              <span className="material-symbols-outlined text-sm">add</span>
              Add Rule
            </button>
          </div>
          <div className="text-center py-4 text-slate-500 dark:text-slate-400 text-sm">
            Configure sensors and actuators
          </div>
        </div>
      </div>
    </aside>
  )
}
