# ÉPICA: Especificación Completa de Componentes React UI

**Estado:** 📋 Planning  
**Prioridad:** 🔴 Alta  
**Versión:** 1.0  
**Fecha Creación:** 2026-01-21  
**Última Actualización:** 2026-01-21

---

## 📑 Tabla de Contenidos

1. [Resumen Ejecutivo](#resumen-ejecutivo)
2. [Arquitectura de Componentes](#arquitectura-de-componentes)
3. [Toolbar & Tool Selection](#toolbar--tool-selection)
4. [Properties Panel](#properties-panel)
5. [Context Menus](#context-menus)
6. [Modals & Dialogs](#modals--dialogs)
7. [Canvas Overlays](#canvas-overlays)
8. [Status & Feedback](#status--feedback)
9. [Navigation & Organization](#navigation--organization)
10. [Advanced Features UI](#advanced-features-ui)
11. [Design System](#design-system)
12. [Implementation Plan](#implementation-plan)

---

## Resumen Ejecutivo

Este documento especifica todos los componentes React necesarios para implementar la UI completa de ArchFlow Whiteboard, inspirado en las mejores prácticas de **Figma**, **Excalidraw**, **Draw.io** y **TLDraw**.

### Componentes Totales: 87 componentes

- **Core Layout:** 8 componentes
- **Toolbar & Tools:** 15 componentes
- **Properties Panel:** 18 componentes
- **Context Menus:** 12 componentes
- **Canvas Overlays:** 14 componentes
- **Modals & Dialogs:** 10 componentes
- **Status & Feedback:** 8 componentes
- **Navigation:** 2 componentes

---

## Arquitectura de Componentes

```
src/components/
├── Layout/
│   ├── AppLayout.tsx              # Main app container
│   ├── Header.tsx                 # Top header bar
│   ├── Sidebar.tsx                # Left sidebar (layers, assets)
│   ├── PropertiesPanel.tsx        # Right properties panel
│   ├── StatusBar.tsx              # Bottom status bar
│   ├── Canvas.tsx                 # Main canvas container
│   ├── FloatingToolbar.tsx        # Center-top floating toolbar
│   └── QuickActions.tsx           # Floating quick actions
│
├── Toolbar/
│   ├── ToolSelector.tsx           # Tool selection buttons
│   ├── ToolButton.tsx             # Individual tool button
│   ├── ToolGroup.tsx              # Grouped tools (e.g., shapes)
│   ├── ShapeToolMenu.tsx          # Shape selection dropdown
│   ├── ArrowToolMenu.tsx          # Arrow/line tool options
│   ├── TextToolOptions.tsx        # Text tool formatting
│   ├── SelectionToolbar.tsx       # Context toolbar for selection
│   ├── AlignmentTools.tsx         # Alignment buttons group
│   ├── DistributeTools.tsx        # Distribution buttons
│   ├── ArrangeTools.tsx           # Z-index control buttons
│   ├── GroupingTools.tsx          # Group/ungroup buttons
│   ├── ConnectionTools.tsx        # Connection creation tools
│   ├── SnapControls.tsx           # Snap-to-grid toggle
│   ├── GizmoModeSelector.tsx      # Transform gizmo mode
│   └── ToolTooltip.tsx            # Enhanced tooltip with shortcuts
│
├── Properties/
│   ├── PropertiesPanel.tsx        # Main panel container
│   ├── PropertySection.tsx        # Collapsible section
│   ├── TransformProperties.tsx    # Position, size, rotation
│   ├── AppearanceProperties.tsx   # Fill, stroke, effects
│   ├── TextProperties.tsx         # Font, size, alignment
│   ├── ConnectionProperties.tsx   # Arrow style, routing, anchors
│   ├── LayoutProperties.tsx       # Constraints, auto-layout
│   ├── EffectsProperties.tsx      # Shadow, blur, etc.
│   ├── ExportProperties.tsx       # Export settings
│   ├── ColorPicker.tsx            # Advanced color picker
│   ├── GradientEditor.tsx         # Gradient configuration
│   ├── SliderInput.tsx            # Numeric slider with input
│   ├── SegmentedControl.tsx       # Tab-like selector
│   ├── PropertyRow.tsx            # Single property row
│   ├── PropertyLabel.tsx          # Property label with tooltip
│   ├── QuickActions.tsx           # Quick property shortcuts
│   ├── BatchEditIndicator.tsx    # Multiple selection indicator
│   └── PropertyPreview.tsx        # Visual preview of changes
│
├── ContextMenu/
│   ├── ContextMenu.tsx            # Base context menu
│   ├── EntityContextMenu.tsx      # Right-click on entity
│   ├── CanvasContextMenu.tsx      # Right-click on canvas
│   ├── SelectionContextMenu.tsx   # Right-click on selection
│   ├── ConnectionContextMenu.tsx  # Right-click on arrow
│   ├── LayerContextMenu.tsx       # Layer panel context menu
│   ├── MenuItem.tsx               # Menu item with icon
│   ├── MenuSeparator.tsx          # Horizontal separator
│   ├── MenuSubmenu.tsx            # Nested submenu
│   ├── MenuShortcut.tsx           # Keyboard shortcut display
│   ├── MenuCheckbox.tsx           # Checkbox menu item
│   └── MenuRadio.tsx              # Radio menu item
│
├── Canvas/
│   ├── CanvasGrid.tsx             # Background grid
│   ├── SelectionBox.tsx           # Box selection visual
│   ├── TransformHandles.tsx       # Resize/rotate handles
│   ├── TransformGizmo.tsx         # 3D-style gizmo
│   ├── SmartGuides.tsx            # Alignment guides
│   ├── SnapIndicators.tsx         # Grid snap feedback
│   ├── ConnectionPreview.tsx      # Arrow creation preview
│   ├── AnchorPoints.tsx           # Connection anchor dots
│   ├── HoverHighlight.tsx         # Element hover effect
│   ├── SelectionIndicator.tsx     # Selected element border
│   ├── MeasurementOverlay.tsx     # Distance measurements
│   ├── ZoomIndicator.tsx          # Current zoom level
│   ├── Cursor.tsx                 # Custom cursors
│   └── MultiCursorOverlay.tsx     # Collaboration cursors
│
├── Modals/
│   ├── Modal.tsx                  # Base modal component
│   ├── ExportModal.tsx            # Export options dialog
│   ├── ImportModal.tsx            # Import file dialog
│   ├── SettingsModal.tsx          # App settings
│   ├── KeyboardShortcutsModal.tsx # Shortcuts reference
│   ├── ShareModal.tsx             # Share/collaborate dialog
│   ├── TemplatesModal.tsx         # Template gallery
│   ├── PluginsModal.tsx           # Plugin marketplace
│   ├── WelcomeModal.tsx           # First-time user guide
│   └── ConfirmDialog.tsx          # Confirmation prompts
│
├── Sidebar/
│   ├── LayersPanel.tsx            # Layer hierarchy
│   ├── LayerItem.tsx              # Single layer row
│   ├── LayerTree.tsx              # Nested layer structure
│   ├── AssetsPanel.tsx            # Reusable assets
│   ├── ComponentsPanel.tsx        # Component library
│   ├── HistoryPanel.tsx           # Undo/redo history
│   ├── SearchBar.tsx              # Layer/asset search
│   └── PanelTabs.tsx              # Sidebar tab switcher
│
├── Feedback/
│   ├── Toast.tsx                  # Toast notification
│   ├── ToastContainer.tsx         # Toast manager
│   ├── LoadingSpinner.tsx         # Loading indicator
│   ├── ProgressBar.tsx            # Progress indicator
│   ├── ErrorBoundary.tsx          # Error handling
│   ├── Tooltip.tsx                # Enhanced tooltips
│   ├── Badge.tsx                  # Status badges
│   └── Skeleton.tsx               # Loading skeletons
│
├── Navigation/
│   ├── Minimap.tsx                # Canvas minimap
│   └── Breadcrumbs.tsx            # Navigation breadcrumbs
│
└── Shared/
    ├── Button.tsx                 # Base button
    ├── IconButton.tsx             # Icon-only button
    ├── Input.tsx                  # Text input
    ├── Select.tsx                 # Dropdown select
    ├── Checkbox.tsx               # Checkbox input
    ├── Radio.tsx                  # Radio input
    ├── Switch.tsx                 # Toggle switch
    ├── Slider.tsx                 # Range slider
    ├── Divider.tsx                # Visual separator
    └── Portal.tsx                 # Portal for modals/tooltips
```

---

## Toolbar & Tool Selection

### 1. **FloatingToolbar.tsx** - Main Tool Palette (Figma-style)

```tsx
interface FloatingToolbarProps {
  position?: 'top-center' | 'top-left' | 'top-right';
  className?: string;
}

export function FloatingToolbar({ position = 'top-center' }: FloatingToolbarProps) {
  return (
    <div className="floating-toolbar">
      {/* Left Section: Tools */}
      <ToolSelector>
        <ToolButton tool="select" icon={<Pointer />} shortcut="V" />
        <ToolButton tool="hand" icon={<Hand />} shortcut="H" />
        <ToolSeparator />
        <ShapeToolMenu />
        <ArrowToolMenu />
        <ToolButton tool="text" icon={<Type />} shortcut="T" />
        <ToolButton tool="pen" icon={<Pen />} shortcut="P" />
        <ToolButton tool="eraser" icon={<Eraser />} shortcut="E" />
      </ToolSelector>

      {/* Center Section: Context Actions (when selection active) */}
      {hasSelection && (
        <SelectionToolbar>
          <AlignmentTools />
          <DistributeTools />
          <ArrangeTools />
          <GroupingTools />
        </SelectionToolbar>
      )}

      {/* Right Section: View Controls */}
      <div className="view-controls">
        <SnapControls />
        <RulerToggle />
        <GridToggle />
        <GizmoModeSelector />
      </div>
    </div>
  );
}
```

### 2. **ShapeToolMenu.tsx** - Shape Selection Dropdown

```tsx
interface ShapeTool {
  id: string;
  icon: React.ReactNode;
  label: string;
  shortcut: string;
}

const shapes: ShapeTool[] = [
  { id: 'rectangle', icon: <Square />, label: 'Rectangle', shortcut: 'R' },
  { id: 'circle', icon: <Circle />, label: 'Circle', shortcut: 'O' },
  { id: 'diamond', icon: <Diamond />, label: 'Diamond', shortcut: 'D' },
  { id: 'triangle', icon: <Triangle />, label: 'Triangle', shortcut: 'Shift+T' },
  { id: 'polygon', icon: <Polygon />, label: 'Polygon', shortcut: 'Shift+P' },
  { id: 'star', icon: <Star />, label: 'Star', shortcut: 'S' },
];

export function ShapeToolMenu() {
  const [isOpen, setIsOpen] = useState(false);
  const [activeTool, setActiveTool] = useUIStore((s) => [s.activeTool, s.setActiveTool]);

  const currentShape = shapes.find((s) => s.id === activeTool) || shapes[0];

  return (
    <Dropdown open={isOpen} onOpenChange={setIsOpen}>
      <DropdownTrigger>
        <ToolButton icon={currentShape.icon} hasDropdown />
      </DropdownTrigger>
      <DropdownContent>
        {shapes.map((shape) => (
          <DropdownItem
            key={shape.id}
            onClick={() => {
              setActiveTool(shape.id);
              setIsOpen(false);
            }}
          >
            <span className="icon">{shape.icon}</span>
            <span className="label">{shape.label}</span>
            <span className="shortcut">{shape.shortcut}</span>
          </DropdownItem>
        ))}
      </DropdownContent>
    </Dropdown>
  );
}
```

### 3. **ArrowToolMenu.tsx** - Connection Tool Options

```tsx
interface ArrowStyle {
  id: LineStyle;
  icon: React.ReactNode;
  label: string;
  description: string;
}

const arrowStyles: ArrowStyle[] = [
  {
    id: 'direct',
    icon: <ArrowRight />,
    label: 'Direct',
    description: 'Straight line',
  },
  {
    id: 'orthogonal',
    icon: <ArrowElbow />,
    label: 'Elbow',
    description: '90° angles',
  },
  {
    id: 'curved',
    icon: <ArrowCurve />,
    label: 'Curved',
    description: 'Smooth bezier',
  },
  {
    id: 'segmented',
    icon: <ArrowSegment />,
    label: 'Segmented',
    description: 'Manual control',
  },
];

export function ArrowToolMenu() {
  const [arrowStyle, setArrowStyle] = useState<LineStyle>('orthogonal');

  return (
    <ToolGroup label="Arrows">
      <ToolButton tool="arrow" icon={<ArrowRight />} shortcut="L" />
      <Dropdown>
        <DropdownTrigger>
          <IconButton icon={<ChevronDown />} size="xs" />
        </DropdownTrigger>
        <DropdownContent>
          <div className="p-2 space-y-1">
            <div className="text-xs font-semibold text-slate-500 px-2 py-1">
              Arrow Style
            </div>
            {arrowStyles.map((style) => (
              <DropdownItem
                key={style.id}
                selected={arrowStyle === style.id}
                onClick={() => setArrowStyle(style.id)}
              >
                <div className="flex items-center gap-3">
                  <span className="icon w-5 h-5">{style.icon}</span>
                  <div className="flex-1">
                    <div className="font-medium">{style.label}</div>
                    <div className="text-xs text-slate-500">
                      {style.description}
                    </div>
                  </div>
                  {arrowStyle === style.id && <Check className="w-4 h-4" />}
                </div>
              </DropdownItem>
            ))}
          </div>
          <MenuSeparator />
          <div className="p-2">
            <label className="flex items-center gap-2 text-sm">
              <Checkbox checked={true} />
              <span>Show anchor points</span>
            </label>
          </div>
        </DropdownContent>
      </Dropdown>
    </ToolGroup>
  );
}
```

### 4. **SelectionToolbar.tsx** - Context Toolbar (Figma-style)

```tsx
export function SelectionToolbar() {
  const selectedIds = useSelectionStore((s) => s.selectedIds);
  const count = selectedIds.length;

  if (count === 0) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className="selection-toolbar"
    >
      {/* Quick Actions */}
      <div className="flex items-center gap-1">
        <IconButton
          icon={<Copy />}
          tooltip="Copy (Ctrl+C)"
          onClick={() => handleCopy()}
        />
        <IconButton
          icon={<Clipboard />}
          tooltip="Paste (Ctrl+V)"
          onClick={() => handlePaste()}
        />
        <IconButton
          icon={<Trash2 />}
          tooltip="Delete (Del)"
          onClick={() => handleDelete()}
        />
      </div>

      <Divider orientation="vertical" />

      {/* Alignment */}
      <AlignmentTools />

      <Divider orientation="vertical" />

      {/* Distribution */}
      {count >= 3 && <DistributeTools />}

      <Divider orientation="vertical" />

      {/* Arrangement */}
      <ArrangeTools />

      <Divider orientation="vertical" />

      {/* Grouping */}
      {count >= 2 && (
        <>
          <IconButton
            icon={<Group />}
            tooltip="Group (Ctrl+G)"
            onClick={() => handleGroup()}
          />
          <IconButton
            icon={<Ungroup />}
            tooltip="Ungroup (Ctrl+Shift+G)"
            onClick={() => handleUngroup()}
          />
        </>
      )}

      {/* Selection Info */}
      <div className="ml-2 text-xs text-slate-500">
        {count} selected
      </div>
    </motion.div>
  );
}
```

### 5. **AlignmentTools.tsx** - Alignment Buttons

```tsx
interface AlignmentButton {
  id: AlignmentType;
  icon: React.ReactNode;
  label: string;
  shortcut: string;
}

const alignments: AlignmentButton[] = [
  { id: 'left', icon: <AlignLeft />, label: 'Align Left', shortcut: 'Ctrl+Shift+L' },
  { id: 'center-h', icon: <AlignCenterH />, label: 'Center H', shortcut: 'Ctrl+Shift+C' },
  { id: 'right', icon: <AlignRight />, label: 'Align Right', shortcut: 'Ctrl+Shift+R' },
  { id: 'top', icon: <AlignTop />, label: 'Align Top', shortcut: 'Ctrl+Shift+T' },
  { id: 'center-v', icon: <AlignCenterV />, label: 'Center V', shortcut: 'Ctrl+Shift+M' },
  { id: 'bottom', icon: <AlignBottom />, label: 'Align Bottom', shortcut: 'Ctrl+Shift+B' },
];

export function AlignmentTools() {
  const align = useAlignmentStore((s) => s.align);

  return (
    <div className="flex items-center gap-0.5">
      {alignments.map((alignment) => (
        <IconButton
          key={alignment.id}
          icon={alignment.icon}
          tooltip={`${alignment.label} (${alignment.shortcut})`}
          onClick={() => align(alignment.id)}
          size="sm"
        />
      ))}
    </div>
  );
}
```

---

## Properties Panel

### 6. **PropertiesPanel.tsx** - Figma-style Right Panel

```tsx
export function PropertiesPanel() {
  const selectedIds = useSelectionStore((s) => s.selectedIds);
  const entities = selectedIds.map((id) => useEntityStore((s) => s.getEntity(id)));

  const [activeTab, setActiveTab] = useState<'design' | 'prototype' | 'code'>('design');

  if (selectedIds.length === 0) {
    return (
      <div className="properties-panel-empty">
        <EmptyState
          icon={<Sparkles />}
          title="No selection"
          description="Select an element to view properties"
        />
      </div>
    );
  }

  const isBatchEdit = selectedIds.length > 1;

  return (
    <div className="properties-panel">
      {/* Header */}
      <div className="panel-header">
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            <TabsTrigger value="design">Design</TabsTrigger>
            <TabsTrigger value="prototype">Prototype</TabsTrigger>
            <TabsTrigger value="code">Code</TabsTrigger>
          </TabsList>
        </Tabs>

        {isBatchEdit && (
          <BatchEditIndicator count={selectedIds.length} />
        )}
      </div>

      {/* Scrollable Content */}
      <ScrollArea className="panel-content">
        {activeTab === 'design' && (
          <>
            {/* Transform Section */}
            <PropertySection title="Transform" defaultOpen>
              <TransformProperties entities={entities} />
            </PropertySection>

            {/* Appearance Section */}
            <PropertySection title="Appearance" defaultOpen>
              <AppearanceProperties entities={entities} />
            </PropertySection>

            {/* Text Section (if text selected) */}
            {entities.some((e) => e.type === 'text') && (
              <PropertySection title="Text">
                <TextProperties entities={entities} />
              </PropertySection>
            )}

            {/* Connection Section (if arrow selected) */}
            {entities.some((e) => e.type === 'connection') && (
              <PropertySection title="Connection">
                <ConnectionProperties entities={entities} />
              </PropertySection>
            )}

            {/* Effects Section */}
            <PropertySection title="Effects">
              <EffectsProperties entities={entities} />
            </PropertySection>

            {/* Export Section */}
            <PropertySection title="Export">
              <ExportProperties entities={entities} />
            </PropertySection>
          </>
        )}
      </ScrollArea>

      {/* Footer Quick Actions */}
      <div className="panel-footer">
        <QuickActions />
      </div>
    </div>
  );
}
```

### 7. **TransformProperties.tsx** - Position, Size, Rotation

```tsx
export function TransformProperties({ entities }: { entities: Entity[] }) {
  const updateProperty = useEntityStore((s) => s.updateProperty);

  const transform = useBatchValue(entities, (e) => ({
    x: e.transform.x,
    y: e.transform.y,
    w: e.transform.w,
    h: e.transform.h,
    rotation: e.transform.rotation || 0,
  }));

  const isMixed = (key: keyof typeof transform) => transform[key] === 'mixed';

  return (
    <div className="space-y-3">
      {/* Position */}
      <div className="gri
d grid-cols-2 gap-2">
        <PropertyRow label="X">
          <NumberInput
            value={isMixed('x') ? undefined : transform.x}
            onChange={(x) => updateProperty('transform.x', x)}
            placeholder={isMixed('x') ? 'Mixed' : undefined}
            unit="px"
          />
        </PropertyRow>
        <PropertyRow label="Y">
          <NumberInput
            value={isMixed('y') ? undefined : transform.y}
            onChange={(y) => updateProperty('transform.y', y)}
            placeholder={isMixed('y') ? 'Mixed' : undefined}
            unit="px"
          />
        </PropertyRow>
      </div>

      {/* Size */}
      <div className="grid grid-cols-2 gap-2">
        <PropertyRow label="W">
          <NumberInput
            value={isMixed('w') ? undefined : transform.w}
            onChange={(w) => updateProperty('transform.w', w)}
            placeholder={isMixed('w') ? 'Mixed' : undefined}
            unit="px"
            min={1}
          />
        </PropertyRow>
        <PropertyRow label="H">
          <NumberInput
            value={isMixed('h') ? undefined : transform.h}
            onChange={(h) => updateProperty('transform.h', h)}
            placeholder={isMixed('h') ? 'Mixed' : undefined}
            unit="px"
            min={1}
          />
        </PropertyRow>
      </div>

      {/* Lock Aspect Ratio */}
      <PropertyRow label="Constrain">
        <Switch checked={false} onCheckedChange={() => {}} />
      </PropertyRow>

      {/* Rotation */}
      <PropertyRow label="Rotation">
        <NumberInput
          value={isMixed('rotation') ? undefined : transform.rotation}
          onChange={(r) => updateProperty('transform.rotation', r)}
          placeholder={isMixed('rotation') ? 'Mixed' : undefined}
          unit="°"
          step={15}
          min={-360}
          max={360}
        />
      </PropertyRow>

      {/* Corner Radius (if rectangle) */}
      {entities.every((e) => e.type === 'rectangle') && (
        <PropertyRow label="Radius">
          <NumberInput
            value={0}
            onChange={() => {}}
            unit="px"
            min={0}
          />
        </PropertyRow>
      )}
    </div>
  );
}
```

### 8. **AppearanceProperties.tsx** - Fill, Stroke, Effects

```tsx
export function AppearanceProperties({ entities }: { entities: Entity[] }) {
  const appearance = useBatchValue(entities, (e) => ({
    fill: e.style.fill,
    stroke: e.style.stroke,
    strokeWidth: e.style.strokeWidth,
    opacity: e.style.opacity,
  }));

  return (
    <div className="space-y-3">
      {/* Fill */}
      <PropertyRow label="Fill">
        <ColorPicker
          value={appearance.fill}
          onChange={(fill) => updateProperty('style.fill', fill)}
          supportGradient
          supportPattern
        />
      </PropertyRow>

      {/* Stroke */}
      <PropertyRow label="Stroke">
        <ColorPicker
          value={appearance.stroke}
          onChange={(stroke) => updateProperty('style.stroke', stroke)}
        />
      </PropertyRow>

      {/* Stroke Width */}
      <PropertyRow label="Width">
        <SliderInput
          value={appearance.strokeWidth}
          onChange={(w) => updateProperty('style.strokeWidth', w)}
          min={0}
          max={20}
          step={0.5}
        />
      </PropertyRow>

      {/* Stroke Style */}
      <PropertyRow label="Style">
        <SegmentedControl
          value="solid"
          options={[
            { value: 'solid', label: 'Solid' },
            { value: 'dashed', label: 'Dashed' },
            { value: 'dotted', label: 'Dotted' },
          ]}
        />
      </PropertyRow>

      {/* Opacity */}
      <PropertyRow label="Opacity">
        <SliderInput
          value={appearance.opacity}
          onChange={(o) => updateProperty('style.opacity', o)}
          min={0}
          max={100}
          step={1}
          unit="%"
        />
      </PropertyRow>

      {/* Blend Mode */}
      <PropertyRow label="Blend">
        <Select
          value="normal"
          options={[
            'Normal',
            'Multiply',
            'Screen',
            'Overlay',
            'Darken',
            'Lighten',
          ]}
        />
      </PropertyRow>
    </div>
  );
}
```

### 9. **ConnectionProperties.tsx** - Arrow/Connection Settings

```tsx
export function ConnectionProperties({ entities }: { entities: Entity[] }) {
  const connections = entities.filter((e) => e.type === 'connection');

  const style = useBatchValue(connections, (c) => ({
    lineStyle: c.lineStyle,
    startArrow: c.startArrow,
    endArrow: c.endArrow,
    autoRoute: c.autoRoute,
  }));

  return (
    <div className="space-y-3">
      {/* Line Style */}
      <PropertyRow label="Line Style">
        <SegmentedControl
          value={style.lineStyle}
          options={[
            { value: 'direct', icon: <ArrowRight />, tooltip: 'Direct' },
            { value: 'orthogonal', icon: <ArrowElbow />, tooltip: 'Elbow' },
            { value: 'curved', icon: <ArrowCurve />, tooltip: 'Curved' },
            { value: 'segmented', icon: <ArrowSegment />, tooltip: 'Segmented' },
          ]}
        />
      </PropertyRow>

      {/* Arrow Heads */}
      <PropertyRow label="Start Arrow">
        <Select
          value={style.startArrow}
          options={['None', 'Arrow', 'Dot', 'Diamond', 'Circle']}
        />
      </PropertyRow>

      <PropertyRow label="End Arrow">
        <Select
          value={style.endArrow}
          options={['None', 'Arrow', 'Dot', 'Diamond', 'Circle']}
        />
      </PropertyRow>

      {/* Auto Routing */}
      <PropertyRow label="Auto Route">
        <Switch
          checked={style.autoRoute}
          onCheckedChange={(v) => updateProperty('autoRoute', v)}
        />
      </PropertyRow>

      {/* Jump Over Lines */}
      <PropertyRow label="Jump Over">
        <Switch checked={false} />
      </PropertyRow>

      {/* Label */}
      <PropertyRow label="Label">
        <Input
          placeholder="Add label..."
          value=""
          onChange={() => {}}
        />
      </PropertyRow>

      {/* Anchor Points */}
      <PropertyRow label="Anchors">
        <div className="flex gap-2">
          <Select value="auto" options={['Auto', 'Top', 'Right', 'Bottom', 'Left', 'Center']} />
          <ArrowRight className="w-4 h-4 text-slate-400" />
          <Select value="auto" options={['Auto', 'Top', 'Right', 'Bottom', 'Left', 'Center']} />
        </div>
      </PropertyRow>
    </div>
  );
}
```

### 10. **ColorPicker.tsx** - Advanced Color Selector

```tsx
interface ColorPickerProps {
  value: string | 'mixed';
  onChange: (color: string) => void;
  supportGradient?: boolean;
  supportPattern?: boolean;
}

export function ColorPicker({
  value,
  onChange,
  supportGradient = false,
  supportPattern = false,
}: ColorPickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [mode, setMode] = useState<'solid' | 'gradient' | 'pattern'>('solid');

  return (
    <Popover open={isOpen} onOpenChange={setIsOpen}>
      <PopoverTrigger>
        <button className="color-picker-trigger">
          <div
            className="color-preview"
            style={{ background: value === 'mixed' ? 'linear-gradient(...)' : value }}
          />
          <span className="color-label">
            {value === 'mixed' ? 'Mixed' : value}
          </span>
          <ChevronDown className="w-4 h-4" />
        </button>
      </PopoverTrigger>

      <PopoverContent className="w-64">
        {/* Mode Selector */}
        {(supportGradient || supportPattern) && (
          <Tabs value={mode} onValueChange={setMode}>
            <TabsList className="w-full">
              <TabsTrigger value="solid">Solid</TabsTrigger>
              {supportGradient && <TabsTrigger value="gradient">Gradient</TabsTrigger>}
              {supportPattern && <TabsTrigger value="pattern">Pattern</TabsTrigger>}
            </TabsList>
          </Tabs>
        )}

        {/* Color Picker */}
        {mode === 'solid' && (
          <div className="p-3 space-y-3">
            {/* HSV Picker */}
            <div className="color-picker-canvas">
              {/* Color square (saturation/value) */}
              <div className="color-square" />
              {/* Hue slider */}
              <Slider min={0} max={360} />
            </div>

            {/* Input Fields */}
            <div className="grid grid-cols-4 gap-2 text-xs">
              <Input placeholder="HEX" />
              <Input placeholder="R" />
              <Input placeholder="G" />
              <Input placeholder="B" />
            </div>

            {/* Opacity */}
            <div className="flex items-center gap-2">
              <span className="text-xs">Opacity</span>
              <Slider className="flex-1" min={0} max={100} />
              <span className="text-xs w-8">100%</span>
            </div>

            {/* Preset Colors */}
            <div className="grid grid-cols-8 gap-1">
              {PRESET_COLORS.map((color) => (
                <button
                  key={color}
                  className="w-6 h-6 rounded"
                  style={{ background: color }}
                  onClick={() => onChange(color)}
                />
              ))}
            </div>
          </div>
        )}

        {/* Gradient Editor */}
        {mode === 'gradient' && <GradientEditor />}
      </PopoverContent>
    </Popover>
  );
}
```

---

## Context Menus

### 11. **EntityContextMenu.tsx** - Right-Click on Entity

```tsx
export function EntityContextMenu({ entityId, position }: {
  entityId: number;
  position: { x: number; y: number };
}) {
  const entity = useEntityStore((s) => s.getEntity(entityId));
  const isSelected = useSelectionStore((s) => s.selectedIds.includes(entityId));

  return (
    <ContextMenu position={position}>
      {/* Quick Actions */}
      <MenuItem
        icon={<Copy />}
        label="Copy"
        shortcut="Ctrl+C"
        onClick={() => handleCopy()}
      />
      <MenuItem
        icon={<Scissors />}
        label="Cut"
        shortcut="Ctrl+X"
        onClick={() => handleCut()}
      />
      <MenuItem
        icon={<Clipboard />}
        label="Paste"
        shortcut="Ctrl+V"
        onClick={() => handlePaste()}
      />
      <MenuItem
        icon={<Copy />}
        label="Duplicate"
        shortcut="Ctrl+D"
        onClick={() => handleDuplicate()}
      />

      <MenuSeparator />

      {/* Arrangement */}
      <MenuSubmenu label="Arrange" icon={<Layers />}>
        <MenuItem icon={<ChevronsUp />} label="Bring to Front" shortcut="Ctrl+Shift+]" />
        <MenuItem icon={<ChevronUp />} label="Bring Forward" shortcut="Ctrl+]" />
        <MenuItem icon={<ChevronDown />} label="Send Backward" shortcut="Ctrl+[" />
        <MenuItem icon={<ChevronsDown />} label="Send to Back" shortcut="Ctrl+Shift+[" />
      </MenuSubmenu>

      {/* Grouping */}
      {isSelected && (
        <>
          <MenuSeparator />
          <MenuItem icon={<Group />} label="Group Selection" shortcut="Ctrl+G" />
        </>
      )}

      {entity.type === 'group' && (
        <MenuItem icon={<Ungroup />} label="Ungroup" shortcut="Ctrl+Shift+G" />
      )}

      <MenuSeparator />

      {/* Lock/Unlock */}
      <MenuItem
        icon={entity.locked ? <Unlock /> : <Lock />}
        label={entity.locked ? 'Unlock' : 'Lock'}
        shortcut="Ctrl+L"
      />

      {/* Hide/Show */}
      <MenuItem
        icon={entity.visible ? <EyeOff /> : <Eye />}
        label={entity.visible ? 'Hide' : 'Show'}
        shortcut="Ctrl+Shift+H"
      />

      <MenuSeparator />

      {/* Delete */}
      <MenuItem
        icon={<Trash2 />}
        label="Delete"
        shortcut="Del"
        onClick={() => handleDelete()}
        variant="destructive"
      />
    </ContextMenu>
  );
}
```

### 12. **CanvasContextMenu.tsx** - Right-Click on Empty Canvas

```tsx
export function CanvasContextMenu({ position }: { position: { x: number; y: number } }) {
  return (
    <ContextMenu position={position}>
      {/* Paste */}
      <MenuItem
        icon={<Clipboard />}
        label="Paste"
        shortcut="Ctrl+V"
        onClick={() => handlePaste(position)}
      />

      <MenuSeparator />

      {/* Insert Shape */}
      <MenuSubmenu label="Insert Shape" icon={<Square />}>
        <MenuItem icon={<Square />} label="Rectangle" shortcut="R" />
        <MenuItem icon={<Circle />} label="Circle" shortcut="O
" />
        <MenuItem icon={<Diamond />} label="Diamond" shortcut="D" />
        <MenuItem icon={<Triangle />} label="Triangle" shortcut="Shift+T" />
      </MenuSubmenu>

      {/* Insert Arrow */}
      <MenuItem icon={<ArrowRight />} label="Insert Arrow" shortcut="L" />

      {/* Insert Text */}
      <MenuItem icon={<Type />} label="Insert Text" shortcut="T" />

      <MenuSeparator />

      {/* Select All */}
      <MenuItem
        icon={<SelectAll />}
        label="Select All"
        shortcut="Ctrl+A"
      />

      <MenuSeparator />

      {/* Grid & Guides */}
      <MenuCheckbox label="Show Grid" checked={true} shortcut="Ctrl+'" />
      <MenuCheckbox label="Snap to Grid" checked={false} shortcut="Ctrl+Shift+'" />
      <MenuCheckbox label="Show Rulers" checked={false} shortcut="Ctrl+R" />

      <MenuSeparator />

      {/* Zoom */}
      <MenuSubmenu label="Zoom" icon={<ZoomIn />}>
        <MenuItem label="Zoom In" shortcut="Ctrl++" />
        <MenuItem label="Zoom Out" shortcut="Ctrl+-" />
        <MenuItem label="Zoom to 100%" shortcut="Ctrl+0" />
        <MenuItem label="Zoom to Fit" shortcut="Ctrl+1" />
        <MenuItem label="Zoom to Selection" shortcut="Ctrl+2" />
      </MenuSubmenu>
    </ContextMenu>
  );
}
```

---

## Canvas Overlays

### 13. **TransformGizmo.tsx** - 3D-style Transform Controls

```tsx
interface TransformGizmoProps {
  entityId: number;
  mode: 'move' | 'scale' | 'rotate' | 'all';
  visible: boolean;
}

export function TransformGizmo({ entityId, mode, visible }: TransformGizmoProps) {
  const entity = useEntityStore((s) => s.getEntity(entityId));
  const camera = useCamera();

  if (!visible) return null;

  const screenPos = camera.worldToScreen(entity.position);
  const screenScale = 1 / camera.zoom; // Constant screen size

  return (
    <div
      className="transform-gizmo"
      style={{
        position: 'absolute',
        left: screenPos.x,
        top: screenPos.y,
        transform: `scale(${screenScale})`,
      }}
    >
      {/* Move Arrows */}
      {(mode === 'move' || mode === 'all') && (
        <>
          <GizmoArrow
            direction="x"
            color="#ff4444"
            length={60}
            onDrag={(delta) => handleMove('x', delta)}
          />
          <GizmoArrow
            direction="y"
            color="#44ff44"
            length={60}
            onDrag={(delta) => handleMove('y', delta)}
          />
          <GizmoHandle
            position="center"
            color="#4444ff"
            size={12}
            onDrag={(delta) => handleMove('xy', delta)}
          />
        </>
      )}

      {/* Scale Handles */}
      {(mode === 'scale' || mode === 'all') && (
        <>
          {CORNER_POSITIONS.map((pos) => (
            <GizmoHandle
              key={pos}
              position={pos}
              color="#ffffff"
              size={10}
              onDrag={(delta) => handleScale(pos, delta)}
            />
          ))}
        </>
      )}

      {/* Rotate Ring */}
      {(mode === 'rotate' || mode === 'all') && (
        <GizmoRotateRing
          radius={80}
          color="#4444ff"
          onDrag={(angle) => handleRotate(angle)}
        />
      )}

      {/* Pivot Point */}
      <GizmoPivot
        position={entity.pivot || 'center'}
        draggable
        onDrag={(newPivot) => handlePivotMove(newPivot)}
      />
    </div>
  );
}
```

### 14. **SmartGuides.tsx** - Alignment Guides (Figma-style)

```tsx
interface Guide {
  type: 'horizontal' | 'vertical';
  position: number;
  color: string;
}

export function SmartGuides() {
  const [guides, setGuides] = useState<Guide[]>([]);
  const isDragging = useUIStore((s) => s.isDragging);

  useEffect(() => {
    if (!isDragging) {
      setGuides([]);
      return;
    }

    // Calculate guides based on nearby entities
    const newGuides = calculateSmartGuides();
    setGuides(newGuides);
  }, [isDragging]);

  if (guides.length === 0) return null;

  return (
    <svg className="smart-guides-overlay" style={{ pointerEvents: 'none' }}>
      {guides.map((guide, i) => (
        <motion.line
          key={i}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          x1={guide.type === 'vertical' ? guide.position : 0}
          y1={guide.type === 'horizontal' ? guide.position : 0}
          x2={guide.type === 'vertical' ? guide.position : '100%'}
          y2={guide.type === 'horizontal' ? guide.position : '100%'}
          stroke={guide.color}
          strokeWidth={1}
          strokeDasharray="4 2"
        />
      ))}
    </svg>
  );
}
```

### 15. **AnchorPoints.tsx** - Connection Anchor Visualization

```tsx
interface AnchorPointsProps {
  entityId: number;
  visible: boolean;
}

export function AnchorPoints({ entityId, visible }: AnchorPointsProps) {
  const entity = useEntityStore((s) => s.getEntity(entityId));
  const [hoveredAnchor, setHoveredAnchor] = useState<string | null>(null);

  if (!visible) return null;

  const anchors = calculateAnchorPositions(entity);

  return (
    <div className="anchor-points-overlay">
      {anchors.map((anchor) => (
        <motion.div
          key={anchor.id}
          className={cn('anchor-point', {
            'anchor-point-hovered': hoveredAnchor === anchor.id,
          })}
          style={{
            position: 'absolute',
            left: anchor.x,
            top: anchor.y,
          }}
          initial={{ scale: 0 }}
          animate={{ scale: hoveredAnchor === anchor.id ? 1.5 : 1 }}
          onMouseEnter={() => setHoveredAnchor(anchor.id)}
          onMouseLeave={() => setHoveredAnchor(null)}
          onClick={() => handleAnchorClick(anchor)}
        >
          <div className="anchor-point-dot" />
          {hoveredAnchor === anchor.id && (
            <div className="anchor-point-label">{anchor.label}</div>
          )}
        </motion.div>
      ))}
    </div>
  );
}

function calculateAnchorPositions(entity: Entity): Anchor[] {
  const { x, y, w, h } = entity.transform;

  return [
    { id: 'top', label: 'Top', x: x + w / 2, y: y },
    { id: 'right', label: 'Right', x: x + w, y: y + h / 2 },
    { id: 'bottom', label: 'Bottom', x: x + w / 2, y: y + h },
    { id: 'left', label: 'Left', x: x, y: y + h / 2 },
    { id: 'top-left', label: 'Top Left', x: x, y: y },
    { id: 'top-right', label: 'Top Right', x: x + w, y: y },
    { id: 'bottom-right', label: 'Bottom Right', x: x + w, y: y + h },
    { id: 'bottom-left', label: 'Bottom Left', x: x, y: y + h },
  ];
}
```

---

## Modals & Dialogs

### 16. **ExportModal.tsx** - Export Options Dialog

```tsx
export function ExportModal({ isOpen, onClose }: ModalProps) {
  const [format, setFormat] = useState<'png' | 'svg' | 'pdf' | 'json'>('png');
  const [scale, setScale] = useState(2);
  const [transparent, setTransparent] = useState(true);

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="lg">
      <ModalHeader>
        <h2>Export</h2>
      </ModalHeader>

      <ModalBody>
        <div className="space-y-4">
          {/* Format Selection */}
          <div>
            <label className="label">Format</label>
            <SegmentedControl
              value={format}
              options={[
                { value: 'png', label: 'PNG', icon: <Image /> },
                { value: 'svg', label: 'SVG', icon: <FileCode /> },
                { value: 'pdf', label: 'PDF', icon: <FileText /> },
                { value: 'json', label: 'JSON', icon: <Code /> },
              ]}
              onChange={setFormat}
            />
          </div>

          {/* Scale */}
          {(format === 'png') && (
            <div>
              <label className="label">Scale</label>
              <SegmentedControl
                value={scale}
                options={[
                  { value: 1, label: '1x' },
                  { value: 2, label: '2x' },
                  { value: 3, label: '3x' },
                ]}
                onChange={setScale}
              />
            </div>
          )}

          {/* Options */}
          <div className="space-y-2">
            <label className="flex items-center gap-2">
              <Checkbox checked={transparent} onCheckedChange={setTransparent} />
              <span>Transparent background</span>
            </label>
            <label className="flex items-center gap-2">
              <Checkbox checked={false} />
              <span>Include only selection</span>
            </label>
          </div>

          {/* Preview */}
          <div className="border rounded p-4 bg-slate-50">
            <div className="text-sm text-slate-500 mb-2">Preview</div>
            <div className="aspect-video bg-white rounded border" />
          </div>
        </div>
      </ModalBody>

      <ModalFooter>
        <Button variant="ghost" onClick={onClose}>
          Cancel
        </Button>
        <Button onClick={handleExport}>
          Export
        </Button>
      </ModalFooter>
    </Modal>
  );
}
```

### 17. **KeyboardShortcutsModal.tsx** - Shortcuts Reference

```tsx
export function KeyboardShortcutsModal({ isOpen, onClose }: ModalProps) {
  const shortcuts = useKeyboardShortcuts();

  const categories = groupBy(shortcuts, 'category');

  return (
    <Modal isOpen={isOpen} onClose={onClose} size="xl">
      <ModalHeader>
        <h2>Keyboard Shortcuts</h2>
      </ModalHeader>

      <ModalBody>
        <div className="grid grid-cols-2 gap-6">
          {Object.entries(categories).map(([category, items]) => (
            <div key={category}>
              <h3 className="font-semibold mb-2">{category}</h3>
              <div className="space-y-1">
                {items.map((shortcut) => (
                  <div
                    key={shortcut.id}
                    className="flex items-center justify-between text-sm"
                  >
                    <span className="text-slate-700">{shortcut.description}</span>
                    <kbd className="kbd">
                      {formatShortcut(shortcut.keys)}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </ModalBody>

      <ModalFooter>
        <Button onClick={onClose}>Close</Button>
      </ModalFooter>
    </Modal>
  );
}
```

---

## Status & Feedback

### 18. **StatusBar.tsx** - Bottom Status Bar

```tsx
export function StatusBar() {
  const camera = useCamera();
  const selectedIds = useSelectionStore((s) => s.selectedIds);
  const activeTool = useUIStore((s) => s.activeTool);
  const fps = usePerformanceStore((s) => s.fps);

  return (
    <div className="status-bar">
      {/* Left: Selection Info */}
      <div className="status-section">
        {selectedIds.length > 0 ? (
          <span className="text-sm">
            {selectedIds.length} selected
          </span>
        ) : (
          <span className="text-sm text-slate-500">
            No selection
          </span>
        )}
      </div>

      {/* Center: Active Tool */}
      <div className="status-section">
        <span className="text-sm font-medium">{activeTool}</span>
      </div>

      {/* Right: View Info */}
      <div className="status-section">
        <span className="text-sm">
          {Math.round(camera.zoom * 100)}%
        </span>
        <Divider orientation="vertical" />
        <span className="text-sm text-slate-500">
          {fps} FPS
        </span>
        <Divider orientation="vertical" />
        <span className="text-sm text-slate-500">
          {camera.position.x.toFixed(0)}, {camera.position.y.toFixed(0)}
        </span>
      </div>
    </div>
  );
}
```

---

## Design System

### Colors (Figma-inspired)

```css
:root {
  /* Primary */
  --color-primary: #0066ff;
  --color-primary-hover: #0052cc;
  
  /* Gizmo Colors */
  --color-gizmo-x: #ff4444; /* Red - X axis */
  --color-gizmo-y: #44ff44; /* Green - Y axis */
  --color-gizmo-z: #4444ff; /* Blue - Z/XY */
  
  /* Guides */
  --color-guide-align: #ff00ff; /* Magenta */
  --color-guide-distance: #00ffff; /* Cyan */
  
  /* Selection */
  --color-selection: #0066ff;
  --color-selection-bg: rgba(0, 102, 255, 0.1);
  
  /* Anchors */
  --color-anchor: #0066ff;
  --color-anchor-hover: #ff4444;
}
```

### Component Sizes

```typescript
export const SIZES = {
  toolbar: {
    height: 48,
    buttonSize: 36,
  },
  properties: {
    width: 280,
    minWidth: 240,
    maxWidth: 400,
  },
  sidebar: {
    width: 240,
    minWidth: 200,
    maxWidth: 320,
  },
  gizmo: {
    arrowLength: 60,
    handleSize: 10,
    rotateRadius: 80,
  },
  anchor: {
    size: 6,
    hoverSize: 8,
  },
};
```

---

## Implementation Plan

### ✅ Phase 1: Core Layout (1 semana) - COMPLETADO
- ✅ AppLayout - Implementado en App.tsx
- ✅ Header - Implementado en Header.tsx
- ✅ FloatingToolbar - Implementado en Toolbar.tsx
- ✅ StatusBar - Implementado en StatusBar.tsx
- ✅ Canvas container - Implementado en Canvas.tsx con WASM bridge
- 🔄 QuickActions - Pendiente

### ✅ Phase 2: Tool Selection (1 semana) - COMPLETADO
- ✅ ToolSelector - Implementado en Toolbar.tsx
- ✅ ShapeToolMenu - Implementado
- ✅ ArrowToolMenu - Implementado
- ✅ SelectionToolbar - Implementado
- ✅ AlignmentTools - Implementado
- ✅ DistributeTools - Implementado
- ✅ ArrangeTools - Implementado
- ✅ GroupingTools - Implementado
- ✅ ConnectionTools - Implementado
- ✅ SnapControls - Implementado
- 🔄 GizmoModeSelector - Pendiente (Sprint 9)
- ✅ ToolTooltip - Implementado

### ✅ Phase 3: Properties Panel (2 semanas) - COMPLETADO
- ✅ PropertiesPanel - Implementado completo
- ✅ TransformProperties - Implementado
- ✅ AppearanceProperties - Implementado
- ✅ TextProperties - Implementado
- ✅ ConnectionProperties - Implementado
- ✅ LayoutProperties - Implementado
- ✅ EffectsProperties - Implementado
- ✅ ExportProperties - Implementado
- ✅ ColorPicker - Implementado avanzado (HSV, HEX, presets)
- ✅ GradientEditor - Implementado
- ✅ SliderInput, SegmentedControl - Implementados
- ✅ BatchEditIndicator, PropertyPreview - Implementados
- ✅ LogicPanel - Implementado
- ✅ HistoryPanel - Implementado

### ❌ Phase 4: Context Menus (1 semana) - PENDIENTE
- ❌ ContextMenu.tsx - No implementado
- ❌ EntityContextMenu.tsx - No implementado
- ❌ CanvasContextMenu.tsx - No implementado
- ❌ SelectionContextMenu.tsx - No implementado
- ❌ ConnectionContextMenu.tsx - No implementado
- ❌ LayerContextMenu.tsx - No implementado
- ❌ MenuItem.tsx - No implementado
- ❌ MenuSeparator.tsx - No implementado
- ❌ MenuSubmenu.tsx - No implementado
- ❌ MenuShortcut.tsx - No implementado
- ❌ MenuCheckbox.tsx - No implementado
- ❌ MenuRadio.tsx - No implementado

### ✅ Phase 5: Canvas Overlays (1.5 semanas) - PARCIAL
- ✅ CanvasGrid - Implementado via Canvas.tsx
- ✅ SelectionBox - Implementado
- ✅ TransformHandles.tsx - Implementado (8 handles + rotate)
- ❌ TransformGizmo.tsx - Pendiente (Sprint 9)
- ✅ SmartGuides.tsx - Implementado (SnapFeedback.tsx)
- ✅ SnapIndicators - Implementado
- ✅ ConnectionPreview.tsx - Implementado (ConnectionRenderer.tsx)
- ❌ AnchorPoints.tsx - Pendiente (Sprint 7)
- ✅ HoverHighlight - Implementado
- ✅ SelectionIndicator - Implementado
- ❌ MeasurementOverlay.tsx - Pendiente (Sprint 9)
- ✅ ZoomIndicator - Implementado (ZoomControls.tsx)
- ✅ Cursor - Implementado
- ✅ ConnectionRenderer.tsx - Implementado

### ❌ Phase 6: Modals & Dialogs (1 semana) - PARCIAL
- ✅ Modal.tsx - Implementado (Radix UI base)
- ❌ ExportModal.tsx - Pendiente
- ❌ ImportModal.tsx - Pendiente
- ❌ SettingsModal.tsx - Pendiente
- ❌ KeyboardShortcutsModal.tsx - Pendiente
- ❌ ShareModal.tsx - Pendiente
- ❌ TemplatesModal.tsx - Pendiente
- ❌ PluginsModal.tsx - Pendiente
- ❌ WelcomeModal.tsx - Pendiente
- ❌ ConfirmDialog.tsx - Pendiente

### ✅ Phase 7: Sidebar Panels (1 semana) - COMPLETADO
- ✅ LayersPanel - Implementado (Sidebar.tsx)
- ✅ LayerItem, LayerTree - Implementados
- ✅ AssetsPanel - Implementado
- ✅ ComponentsPanel - Implementado
- ✅ HistoryPanel - Implementado (Properties/HistoryPanel.tsx)
- ✅ SearchBar - Implementado
- ✅ PanelTabs - Implementado

### ✅ Phase 8: Status & Feedback (1 semana) - COMPLETADO
- ✅ Toast.tsx - Implementado
- ✅ ToastContainer.tsx - Implementado
- ✅ LoadingSpinner.tsx - Implementado (Skeleton.tsx)
- ✅ ProgressBar.tsx - Implementado
- ✅ ErrorBoundary.tsx - Implementado
- ✅ Tooltip.tsx - Implementado
- ✅ Badge.tsx - Implementado
- ✅ Skeleton.tsx - Implementado

### ✅ Phase 9: Navigation (1 semana) - COMPLETADO
- ✅ Minimap.tsx - Implementado (via ZoomControls)
- ✅ Breadcrumbs.tsx - Implementado (via canvas store)

### ❌ Phase 10: Advanced Features (Pendiente)
- ❌ Containers UI - Pendiente
- ❌ Swimlanes UI - Pendiente
- ❌ Auto-align UI - Pendiente

---

## 📊 Resumen de Estado por Categoría

```
CORE LAYOUT: 6/8 ✅ (75%)
├── AppLayout ✅
├── Header ✅
├── Sidebar ✅
├── StatusBar ✅
├── Canvas ✅
├── FloatingToolbar ✅
└── QuickActions 🔄

TOOLBAR & TOOLS: 12/15 ✅ (80%)
├── ToolSelector ✅
├── ShapeToolMenu ✅
├── ArrowToolMenu ✅
├── SelectionToolbar ✅
├── AlignmentTools ✅
├── DistributeTools ✅
├── ArrangeTools ✅
├── GroupingTools ✅
├── ConnectionTools ✅
├── SnapControls ✅
├── GizmoModeSelector ❌
└── ToolTooltip ✅

PROPERTIES PANEL: 18/18 ✅ (100%)
└── Todos implementados ✅✅✅

CONTEXT MENUS: 0/12 ❌ (0%)
└── Ninguno implementado ❌

MODALS: 1/10 ✅ (10%)
├── Modal.tsx ✅
└── 9 modals pendientes ❌

CANVAS OVERLAYS: 9/14 ✅ (64%)
├── SelectionBox ✅
├── TransformHandles ✅
├── SmartGuides ✅
├── ConnectionPreview ✅
├── HoverHighlight ✅
├── SelectionIndicator ✅
├── ZoomIndicator ✅
├── Cursor ✅
├── ConnectionRenderer ✅
├── TransformGizmo ❌
├── AnchorPoints ❌
└── MeasurementOverlay ❌

STATUS & FEEDBACK: 8/8 ✅ (100%)
└── Todos implementados ✅✅✅

NAVIGATION: 2/2 ✅ (100%)
├── Minimap ✅
└── Breadcrumbs ✅
```

---

**Última actualización:** 2026-02-05  
**Versión:** 1.1  
**Total Componentes:** 87  
**Implementados:** ~55%  
**Pendientes:** ~45%