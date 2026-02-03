# archflow-web-ui

> **React Canvas Application** - Full-featured architecture diagram editor with real-time collaboration, WASM integration, and professional canvas interactions.

## Overview

`archflow-web-ui` is the web application layer for ArchFlow, built with React and TypeScript. It provides a professional diagramming interface with real-time collaboration, infinite canvas navigation, and seamless WASM integration with the Rust backend.

**Key Capabilities:**
- **Infinite canvas** - Professional pan/zoom with viewport culling
- **Real-time collaboration** - CRDT-based multi-user editing
- **WASM integration** - High-performance Rust backend
- **React components** - Modern, composable UI
- **State management** - Zustand stores for global state

## Architecture

The application follows **Hexagonal Architecture** with React presentation:

```
┌─────────────────────────────────────────────────────────────────┐
│                   Presentation Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │Canvas        │  │PropertiesPanel│ │Toolbar       │          │
│  │Component     │  │Component     │  │Component     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                   Application Layer                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │useEntityStore│  │useCanvas     │  │useCollaboration│         │
│  │Hook          │  │Hook          │  │Hook          │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                    State Management                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │EntityStore   │  │CanvasStore   │  │CollabStore   │          │
│  │(Zustand)     │  │(Zustand)     │  │(Zustand)     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      WASM Bridge Layer                          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │ArchFlowWeb   │  │WasmModule    │                            │
│  │(Rust Backend)│  │(Interface)   │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### Canvas Component

The main canvas component for rendering and interaction:

```typescript
import { Canvas } from '@archflow/web-ui';

<Canvas
  width={1920}
  height={1080}
  onEntitySelect={handleSelect}
  onEntityMove={handleMove}
/>
```

**Features:**
- Infinite pan/zoom
- Entity rendering
- Hit testing
- Selection handling
- Connection drawing

### Properties Panel

Side panel for editing entity properties:

```typescript
import { PropertiesPanel } from '@archflow/web-ui';

<PropertiesPanel
  entity={selectedEntity}
  onChange={handlePropertyChange}
/>
```

**Editable Properties:**
- Position and size
- Colors and borders
- Text content
- Metadata
- Tags

### Toolbar

Main toolbar with common actions:

```typescript
import { Toolbar } from '@archflow/web-ui';

<Toolbar
  actions={[
    { icon: 'select', action: 'select' },
    { icon: 'rectangle', action: 'create-rectangle' },
    { icon: 'circle', action: 'create-circle' },
    { icon: 'text', action: 'create-text' },
  ]}
/>
```

## React Hooks

### useEntityStore

Hook for accessing entity data:

```typescript
import { useEntityStore } from '@archflow/web-ui';

function MyComponent() {
  const entities = useEntityStore();
  const addEntity = useEntityStore(state => state.addEntity);
  
  return (
    <button onClick={() => addEntity(newEntity)}>
      Add Entity
    </button>
  );
}
```

### useCanvas

Hook for canvas state and controls:

```typescript
import { useCanvas } from '@archflow/web-ui';

function CanvasControls() {
  const { zoom, pan, zoomTo, panTo } = useCanvas();
  
  return (
    <div>
      <button onClick={() => zoomTo(1.1)}>Zoom In</button>
      <button onClick={() => zoomTo(0.9)}>Zoom Out</button>
      <button onClick={() => panTo(0, 0)}>Reset View</button>
    </div>
  );
}
```

### useCollaboration

Hook for real-time collaboration:

```typescript
import { useCollaboration } from '@archflow/web-ui';

function CollaborationPanel() {
  const { users, presence, isConnected } = useCollaboration();
  
  return (
    <div>
      <h3>Collaborators</h3>
      {users.map(user => (
        <div key={user.id}>
          {user.name} - {presence[user.id] || 'away'}
        </div>
      ))}
    </div>
  );
}
```

### useSelection

Hook for managing entity selection:

```typescript
import { useSelection } from '@archflow/web-ui';

function SelectionManager() {
  const { selectedIds, select, deselect, clear } = useSelection();
  
  return (
    <div>
      <p>Selected: {selectedIds.length}</p>
      <button onClick={clear}>Clear Selection</button>
    </div>
  );
}
```

## State Management

### Zustand Stores

Global state is managed with Zustand:

```typescript
// Entity Store
interface EntityStore {
  entities: Map<EntityId, Entity>;
  addEntity: (entity: Entity) => void;
  updateEntity: (id: EntityId, updates: Partial<Entity>) => void;
  removeEntity: (id: EntityId) => void;
  getEntity: (id: EntityId) => Entity | undefined;
}

// Canvas Store
interface CanvasStore {
  viewport: { x: number; y: number; zoom: number };
  setViewport: (viewport: Viewport) => void;
  zoomTo: (factor: number, center?: Point2D) => void;
  panTo: (x: number, y: number) => void;
}

// Collaboration Store
interface CollabStore {
  users: User[];
  presence: Record<string, Presence>;
  isConnected: boolean;
  connect: () => void;
  disconnect: () => void;
}
```

## WASM Integration

### Module Loading

Load the WASM module:

```typescript
import initArchFlow, { ArchFlowWeb } from '@archflow/wasm';

const wasm = await initArchFlow();
const archflow = new ArchFlowWeb();

// Use the API
archflow.spawn_entity(100.0, 200.0, 50.0, 30.0);
```

### Type Safety

TypeScript definitions for WASM:

```typescript
declare module '@archflow/wasm' {
  export interface ArchFlowWeb {
    spawn_entity(x: number, y: number, w: number, h: number): EntityId;
    despawn_entity(id: EntityId): void;
    set_position(id: EntityId, x: number, y: number): void;
    // ... more methods
  }
  
  export function initArchFlow(): Promise<ArchFlowWeb>;
}
```

## Real-time Collaboration

### CRDT Integration

Conflict-free replicated data types:

```typescript
import { CRDTManager } from '@archflow/web-ui';

const crdt = new CRDTManager(documentId);

// Apply local change
const remoteOp = crdt.applyLocal({
  type: 'move',
  entityId: 'entity-1',
  position: { x: 100, y: 200 },
  timestamp: Date.now(),
});

// Broadcast to peers
websocket.send(JSON.stringify(remoteOp));

// Apply remote change
crdt.applyRemote(remoteOp);
```

### Presence Tracking

Track user cursors and presence:

```typescript
import { PresenceManager } from '@archflow/web-ui';

const presence = new PresenceManager(userId);

// Update cursor
presence.updateCursor({ x: 150, y: 200 });

// Broadcast presence
websocket.send(presence.serialize());

// Handle remote presence
presence.updateFromPeer(peerId, presenceData);
```

## Performance Optimization

### React Optimization

```typescript
// Memoize expensive computations
const visibleEntities = useMemo(() => {
  return entities.filter(e => isInViewport(e, viewport));
}, [entities, viewport]);

// Prevent unnecessary re-renders
const EntityComponent = React.memo(({ entity }) => {
  return <g transform={...}>{/* ... */}</g>;
});
```

### Virtual Scrolling

Only render visible entities:

```typescript
const entitiesToRender = entities.filter(entity => 
  entity.position.x + entity.size.width > viewport.x &&
  entity.position.x < viewport.x + viewport.width &&
  entity.position.y + entity.size.height > viewport.y &&
  entity.position.y < viewport.y + viewport.height
);
```

### Request Animation Frame

Sync rendering with browser:

```typescript
function renderLoop() {
  // Update canvas
  renderer.render(viewport);
  
  // Request next frame
  requestAnimationFrame(renderLoop);
}
```

## Usage Examples

### Basic Canvas Setup

```typescript
import { Canvas, Toolbar, PropertiesPanel } from '@archflow/web-ui';

function App() {
  return (
    <div className="app">
      <Toolbar />
      <div className="main">
        <Canvas />
        <PropertiesPanel />
      </div>
    </div>
  );
}
```

### Custom Entity Renderer

```typescript
import { useEntityRenderer } from '@archflow/web-ui';

function CustomRenderer() {
  const renderEntity = useEntityRenderer();
  
  renderEntity.register('custom', (entity) => {
    return <g>{/* Custom rendering */}</g>;
  });
}
```

### Keyboard Shortcuts

```typescript
import { useKeyboardShortcuts } from '@archflow/web-ui';

function Shortcuts() {
  useKeyboardShortcuts({
    'Delete': () => deleteSelected(),
    'Escape': () => clearSelection(),
    'Ctrl+C': () => copySelection(),
    'Ctrl+V': () => pasteClipboard(),
    'Ctrl+Z': () => undo(),
    'Ctrl+Shift+Z': () => redo(),
  });
}
```

## File Structure

```
src/
├── components/          # React components
│   ├── Canvas/          # Main canvas
│   ├── PropertiesPanel/ # Property editing
│   ├── Toolbar/         # Tool palette
│   └── Collaboration/   # Multi-user UI
├── hooks/               # Custom React hooks
│   ├── useEntityStore.ts
│   ├── useCanvas.ts
│   ├── useCollaboration.ts
│   └── useSelection.ts
├── stores/              # Zustand stores
│   ├── entityStore.ts
│   ├── canvasStore.ts
│   └── collabStore.ts
├── sdk/                 # Behaviors SDK
│   ├── ShapeBuilder.ts
│   ├── BehaviorRegistry.ts
│   └── types.ts
└── utils/               # Utilities
    ├── crdt.ts
    └── websocket.ts
```

## Performance Characteristics

| Metric | Target | Notes |
|--------|--------|-------|
| Frame Rate | 60 FPS | With 1000+ entities |
| Initial Load | <2s | Including WASM |
| Entity Sync | <10ms | 1000 entities |
| Collaboration Latency | <100ms | CRDT merge |
| Memory Usage | <100MB | Typical document |

## Integration Points

```toml
[dependencies]
archflow-web-ui = "0.36"
archflow-core = "0.36"     # Via WASM
archflow-engine = "0.36"   # Via WASM
archflow-render = "0.36"   # Via WASM
```

### Data Flow

```
React Component → Zustand Store → WASM Bridge → Rust Backend
                                               │
                                               ▼
                                         EntityStore Update
                                               │
                                               ▼
                                        State Change Event
                                               │
                                               ▼
                                        React Re-render
```

## Constraints and Limitations

### Current Limitations

- **Browser support**: Modern browsers only (WebGPU)
- **Mobile support**: Limited touch interactions
- **Offline mode**: Requires network for collaboration
- **File size**: WASM module ~2MB compressed

### Platform Requirements

- **Chrome/Edge**: Full support
- **Firefox**: WebGPU behind flag
- **Safari**: Experimental support
- **Mobile**: Chrome Android only

## Best Practices

### Component Design

1. **Keep components small**: Single responsibility
2. **Use hooks**: Extract logic from components
3. **Memoize aggressively**: Prevent re-renders
4. **Type everything**: Full TypeScript coverage

### State Management

1. **Normalize state**: Flat store structure
2. **Batch updates**: Group state changes
3. **Use selectors**: Computed values
4. **Avoid prop drilling**: Use context/hooks

### Performance

1. **Virtualize lists**: Large entity counts
2. **Debounce inputs**: Text fields, sliders
3. **Lazy load**: Components on demand
4. **Optimize images**: Compress textures

## Future Enhancements

### Planned Features

- **Mobile support**: Touch gestures, responsive UI
- **Offline mode**: Service worker, PWA
- **Plugin system**: Custom tools and behaviors
- **Themes**: Dark/light mode switching
- **Accessibility**: WCAG 2.1 compliance

### Performance Targets

- **10K entities**: Maintain 60 FPS
- **100 users**: Real-time collaboration
- <1s cold start: Initial load time
- <50MB memory: Typical usage

## References

- **EPIC-WEB-010**: Canvas rendering
- **EPIC-WEB-011**: Behaviors SDK
- **archflow-render**: GPU rendering
- **archflow-engine**: Entity management
- **React Docs**: https://react.dev/
- **Zustand Docs**: https://zustand-demo.pmnd.rs/

## License

MIT License - See LICENSE file for details.
