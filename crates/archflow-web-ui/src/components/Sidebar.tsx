import { useState } from 'react'

interface SidebarProps {}

interface TreeNode {
  id: string
  label: string
  icon: string
  children?: TreeNode[]
  expanded?: boolean
}

const demoTree: TreeNode[] = [
  {
    id: 'database',
    label: 'Database',
    icon: 'storage',
    expanded: true,
    children: [
      { id: 'rds', label: 'AWS RDS', icon: 'dns' },
      { id: 'dynamodb', label: 'DynamoDB', icon: 'table_chart' },
    ],
  },
  {
    id: 'compute',
    label: 'Compute',
    icon: 'memory',
    children: [
      { id: 'ec2', label: 'AWS EC2', icon: 'crop_square' },
      { id: 'lambda', label: 'Lambda', icon: 'functions' },
    ],
  },
  {
    id: 'storage',
    label: 'Storage',
    icon: 'folder',
    children: [
      { id: 's3', label: 'AWS S3', icon: 'inventory_2' },
      { id: 'ebs', label: 'EBS', icon: 'hard_drive' },
    ],
  },
]

export default function Sidebar({}: SidebarProps) {
  const [activeTab, setActiveTab] = useState<'layers' | 'tree' | 'library'>('layers')
  const [tree, setTree] = useState<TreeNode[]>(demoTree)
  const [searchQuery, setSearchQuery] = useState('')

  const toggleNode = (id: string) => {
    setTree((prev) =>
      prev.map((node) => {
        if (node.id === id) {
          return { ...node, expanded: !node.expanded }
        }
        if (node.children) {
          return {
            ...node,
            children: node.children.map((child) =>
              child.id === id ? { ...child, expanded: !child.expanded } : child
            ),
          }
        }
        return node
      })
    )
  }

  const renderNode = (node: TreeNode, depth = 0): React.ReactNode => (
    <div key={node.id}>
      <div
        className="flex items-center gap-2 px-3 py-1.5 hover:bg-slate-100 dark:hover:bg-slate-700 cursor-pointer rounded"
        style={{ paddingLeft: `${depth * 16 + 12}px` }}
        onClick={() => node.children && toggleNode(node.id)}
      >
        {node.children ? (
          <span className="material-symbols-outlined text-sm text-slate-500">
            {node.expanded ? 'expand_more' : 'chevron_right'}
          </span>
        ) : (
          <span className="w-4" />
        )}
        <span className="material-symbols-outlined text-sm text-slate-600 dark:text-slate-400">
          {node.icon}
        </span>
        <span className="text-sm text-slate-700 dark:text-slate-300">{node.label}</span>
      </div>
      {node.expanded && node.children?.map((child) => renderNode(child, depth + 1))}
    </div>
  )

  return (
    <aside className="w-64 border-r border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark overflow-y-auto">
      {/* Search */}
      <div className="p-4">
        <div className="relative">
          <span className="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-sm text-slate-400">
            search
          </span>
          <input
            type="text"
            placeholder="Search entities..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-4 py-2 rounded-lg border border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          />
        </div>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border-light dark:border-border-dark">
        <button
          onClick={() => setActiveTab('layers')}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === 'layers'
              ? 'text-primary border-b-2 border-primary'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
          }`}
        >
          <span className="material-symbols-outlined text-lg">layers</span>
          <span className="hidden sm:inline">Layers</span>
        </button>
        <button
          onClick={() => setActiveTab('tree')}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === 'tree'
              ? 'text-primary border-b-2 border-primary'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
          }`}
        >
          <span className="material-symbols-outlined text-lg">account_tree</span>
          <span className="hidden sm:inline">Tree</span>
        </button>
        <button
          onClick={() => setActiveTab('library')}
          className={`flex-1 flex items-center justify-center gap-1 py-2 text-sm font-medium transition-colors ${
            activeTab === 'library'
              ? 'text-primary border-b-2 border-primary'
              : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
          }`}
        >
          <span className="material-symbols-outlined text-lg">extension</span>
          <span className="hidden sm:inline">Library</span>
        </button>
      </div>

      {/* Content */}
      <div className="p-2">
        {activeTab === 'tree' && tree.map((node) => renderNode(node))}
        {activeTab === 'layers' && (
          <div className="text-center py-8 text-slate-500 dark:text-slate-400 text-sm">
            Layers panel coming soon
          </div>
        )}
        {activeTab === 'library' && (
          <div className="text-center py-8 text-slate-500 dark:text-slate-400 text-sm">
            Library panel coming soon
          </div>
        )}
      </div>
    </aside>
  )
}
