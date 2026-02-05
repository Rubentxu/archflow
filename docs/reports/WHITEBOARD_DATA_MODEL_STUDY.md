# Data Model Study: Whiteboard Persistence Formats

## Executive Summary

This document analyzes the data models and persistence formats used by leading whiteboard applications (tldraw, Excalidraw, Figma, draw.io) to design a professional, exportable data model for ArchFlow that supports:
- Shapes and elements
- Connectors and bindings
- Pages/boards
- Component libraries (reusable templates)
- Import/export formats
- **WASM-optimized internal processing** (SpatialHash, EntityStore SoA, Transform Hierarchy)

Based on research, we propose a **record-based schema** inspired by tldraw with Excalidraw's simplicity, using JSON as the primary format with optional binary compression. The model is designed to efficiently map to ArchFlow's internal WASM data structures for optimal batch processing of 100,000+ entities at 60 FPS.

---

## 0. WASM Internal Architecture Integration

Before designing the persistence model, we analyzed ArchFlow's internal WASM implementations to ensure the data model facilitates efficient processing:

### 0.1 EntityStore Structure of Arrays (SoA)

The internal `EntityStore` uses **SoA layout** for cache-efficient batch processing:

```rust
// HOT DATA (Cache Lines 0-2): Accessed every frame by renderer
pub struct EntityStore {
    pub transforms: Vec<[f32; 4]>,     // [x, y, w, h] - 16 bytes per entity
    pub metadata: Vec<u32>,            // Bit-packed: shape:4 | layer:4 | visibility:1 | selected:1 | locked:1
    pub colors: Vec<u32>,              // 0xRRGGBBAA
    pub texture_index: Vec<u16>,
    pub uv_rects: Vec<[f32; 4]>,
    pub color_tints: Vec<[f32; 4]>,
    pub text_glyph_start: Vec<u32>,
    pub text_glyph_count: Vec<u16>,
    pub text_scale: Vec<f32>,

    // TRANSFORM HIERARCHY (V3.0)
    pub parent_id: Vec<Option<EntityId>>,
    pub local_transform: Vec<[f32; 4]>,
    pub world_transform: Vec<[f32; 4]>,
    pub dirty_hierarchy: FixedBitSet,

    // COLD DATA (Access only on selection/inspection)
    pub arch_data: Vec<Option<Box<ArchitectureData>>>,
    pub string_pool: StringPool,
}
```

**Key Insight**: The data model must serialize/deserialize efficiently to these parallel arrays. Parent-child relationships are stored separately for hierarchical updates.

### 0.2 SpatialHash Indexes

ArchFlow maintains **two SpatialHash instances** synchronized via `draw_order`:

| SpatialHash | Cell Size | Purpose |
|-------------|-----------|---------|
| Engine | 64px | Rendering hit-testing |
| Logic | 40px | Sensor collision detection |

```rust
// Both implementations support:
// - O(1) insert/update/remove
// - Multi-cell coverage for large entities
// - Deduplication in query results
// - BTreeMap/BTreeSet for deterministic iteration
```

### 0.3 Transform Hierarchy with Dirty Tracking

```rust
// When parent moves:
parent_id[idx] = Some(parent_entity_id);
local_transform[idx] = [local_x, local_y, local_w, local_h];
world_transform[idx] = [world_x, world_y, world_w, world_h];  // Computed
dirty_hierarchy.insert(idx);  // Mark for propagation
```

**Update Flow**:
1. Mark `dirty_hierarchy` when parent changes
2. Call `update_hierarchy()` to propagate to children
3. Only affected subtrees are updated (not entire scene)

---

## 1. Analysis of Existing Solutions

### 1.1 tldraw - Record-Based Store Model

**Architecture**: tldraw uses a **record-based store** where all data is stored as "records" in a normalized database-like structure.

**Record Types**:
```typescript
type TLRecord =
    | TLAsset           // Images, videos, bookmarks
    | TLBinding         // Connections between shapes (arrows)
    | TLCamera          // Viewport state per page
    | TLDocument        // Root document metadata
    | TLInstance        // User instance state
    | TLInstancePageState // Per-page user state
    | TLPage            // Document pages
    | TLShape           // All shape types
    | TLInstancePresence // Real-time presence
    | TLPointer         // Mouse/touch state
```

**Example Shape Record**:
```json
{
    "parentId": "page:somePage",
    "id": "shape:someId",
    "typeName": "shape",
    "type": "geo",
    "x": 106,
    "y": 294,
    "rotation": 0,
    "index": "a28",
    "opacity": 1,
    "isLocked": false,
    "props": {
        "w": 200,
        "h": 200,
        "geo": "rectangle",
        "color": "black",
        "labelColor": "black",
        "fill": "none",
        "dash": "draw",
        "size": "m",
        "font": "draw",
        "text": "diagram",
        "align": "middle",
        "verticalAlign": "middle"
    },
    "meta": {}
}
```

**Key Characteristics**:
- Each record has a unique ID with prefix (`shape:`, `page:`, `binding:`)
- Separation of base properties (x, y, rotation) from shape-specific props
- `meta` object for custom user data
- Index-based z-ordering (`a28`, `a29`, etc.)

**Binding Example**:
```json
{
    "id": "binding:someId",
    "typeName": "binding",
    "type": "arrow",
    "fromId": "shape:arrowId",
    "toId": "shape:someOtherShapeId",
    "props": {
        "terminal": "end",
        "isPrecise": true,
        "isExact": false,
        "normalizedAnchor": { "x": 0.5, "y": 0.5 }
    },
    "meta": {}
}
```

---

### 1.2 Excalidraw - Element-Based Model

**Architecture**: Excalidraw uses an **element-based array model** with all elements in a flat array.

**File Format** (`.excalidraw`):
```json
{
    "type": "excalidraw",
    "version": 2,
    "source": "https://excalidraw.com",
    "elements": [
        {
            "id": "pologsyG-tAraPgiN9xP9b",
            "type": "rectangle",
            "x": 928,
            "y": 319,
            "width": 134,
            "height": 90,
            "angle": 0,
            "strokeColor": "#000000",
            "backgroundColor": "transparent",
            "fillStyle": "hachure",
            "strokeWidth": 2,
            "strokeStyle": "solid",
            "roughness": 2,
            "opacity": 100,
            "groupIds": [],
            "roundness": null,
            "seed": 123456,
            "version": 1,
            "versionNonce": 789012
        }
    ],
    "appState": {
        "gridSize": 20,
        "viewBackgroundColor": "#ffffff",
        "zoom": 1
    },
    "files": {
        "3cebd7720911620a3938ce77243696149da03861": {
            "mimeType": "image/png",
            "id": "3cebd7720911620a3938ce77243626149da03861",
            "dataURL": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgA=",
            "created": 1690295874454,
            "lastRetrieved": 1690295874454
        }
    }
}
```

**Key Characteristics**:
- Flat array of elements (simpler than tldraw's normalized approach)
- Embedded image data in `files` object
- `appState` for editor state
- Custom data via `customData` property
- Clipboard format similar to file format

---

### 1.3 Figma - Document Model

**Architecture**: Figma uses a **hierarchical document tree** with node-based structure.

**Key Concepts**:
- **Document**: Root node containing pages
- **Page**: Contains frames and other content
- **Frame**: Container for design elements
- **Components**: Reusable definitions
- **ComponentSets**: Variants of components

**Structure**:
```
Document
├── name: "My Design"
├── lastModified: "2024-01-01T00:00:00Z"
└── pages:
    └── Page 1
        ├── children: [Frame, Frame, ...]
        │   └── Frame
        │       ├── name: "Card Component"
        │       ├── absoluteBoundingBox: {...}
        │       ├── fills: [...]
        │       ├── strokes: [...]
        │       ├── effects: [...]
        │       └── children: [Rectangle, Text, ...]
        │           └── Text
        └── componentId: null
```

**Export Formats**:
- `.fig` (binary, proprietary)
- JSON via REST API
- SVG, PNG, PDF exports

---

### 1.4 draw.io (diagrams.net)

**Architecture**: draw.io uses **XML-based format** with embedded diagrams.

**File Format** (`.drawio`):
```xml
<mxfile host="draw.io" modified="2024-01-01T00:00:00Z" agent="5.0" etag="xxx" version="20.8.0" type="device">
    <diagram id="diagram-id" name="Page-1">
        <mxGraphModel dx="1422" dy="794" grid="1" gridSize="10" guides="1" tools="1" connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="850" pageHeight="1100" math="0" shadow="0">
            <root>
                <mxCell id="0" />
                <mxCell id="1" parent="0" />
                <mxCell id="2" value="Rectangle" style="rounded=0;whiteSpace=wrap;html=1;" vertex="1" parent="1">
                    <mxGeometry x="80" y="80" width="120" height="60" as="geometry" />
                </mxCell>
                <mxCell id="3" value="" style="endArrow=classic;html=1;entryX=0;entryY=0.5;entryDx=0;entryDy=0;" edge="1" parent="1" source="2" target="4">
                    <mxGeometry as="geometry" />
                </mxCell>
            </root>
        </mxGraphModel>
    </diagram>
</mxfile>
```

**Key Characteristics**:
- XML-based (easier to parse with standard tools)
- Cell-based hierarchical model
- Style attributes as semicolon-separated key-values
- Supports embedded images (base64)
- Templates stored as separate XML files

---

## 2. Comparative Analysis

| Aspect | tldraw | Excalidraw | Figma | draw.io |
|--------|--------|------------|-------|---------|
| **Format** | JSON | JSON | Binary/JSON | XML |
| **Structure** | Record-based | Flat array | Hierarchical tree | Cell tree |
| **Z-order** | Index string | Implicit order | z-index | Implicit order |
| **Connectors** | Bindings | start/end refs | Auto-layout | Edge cells |
| **Components** | Custom shapes | Library elements | Native | Templates |
| **Custom Data** | meta object | customData | Plugin data | User data |
| **Assets** | Separate store | Inline files | Separate storage | Inline base64 |
| **Versioning** | Migrations | version field | fileFormatVersion | etag |

---

## 3. Proposed Data Model for ArchFlow

### 3.1 Design Principles

Based on research, we propose a **hybrid approach** combining:
1. **tldraw's normalized record model** for efficient updates
2. **Excalidraw's simplicity** in element structure
3. **Component library system** inspired by Figma
4. **JSON as primary format** with optional compression

### 3.2 Core Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// ARCHFLOW DATA MODEL - Core Types
// ═══════════════════════════════════════════════════════════════════════════════

interface ArchFlowDocument {
    version: number;
    schema: ArchFlowSchema;
    store: ArchFlowStore;
    meta: DocumentMeta;
}

interface ArchFlowSchema {
    version: number;
    shapeTypes: Record<string, ShapeSchema>;
    bindingTypes: Record<string, BindingSchema>;
    migrations: Migration[];
}

interface ArchFlowStore {
    // Indexed by ID for O(1) lookup
    shapes: Record<ShapeId, ArchFlowShape>;
    bindings: Record<BindingId, ArchFlowBinding>;
    pages: Record<PageId, ArchFlowPage>;
    assets: Record<AssetId, ArchFlowAsset>;
    cameras: Record<CameraId, ArchFlowCamera>;
    instances: Record<InstanceId, ArchFlowInstance>;
}

interface DocumentMeta {
    id: string;
    name: string;
    created: string;
    modified: string;
    creator: string;
    tags: string[];
}
```

### 3.2.1 WASM-Optimized Internal Representation

For efficient batch processing, the document also includes a **parallel arrays representation** that directly maps to the internal `EntityStore`:

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// WASM INTERNAL FORMAT - Structure of Arrays for O(1) deserialization
// ═══════════════════════════════════════════════════════════════════════════════

interface ArchFlowWASMStore {
    // ═══════════════════════════════════════════════════════════
    // HOT DATA (Cache Lines 0-2): Paralle arrays for batch processing
    // ═══════════════════════════════════════════════════════════

    // Transform array: [x, y, width, height] per entity
    transforms: number[];  // Flat array, 4 * entity_count elements

    // Metadata bitfield: shape:4 | layer:4 | visible:1 | selected:1 | locked:1
    metadata: number[];

    // Colors: 0xRRGGBBAA packed integer
    colors: number[];

    // Texture atlas indices
    texture_indices: number[];

    // UV coordinates in atlas [u, v, w, h]
    uv_rects: number[];

    // Color tints [r, g, b, a]
    color_tints: number[];

    // Text glyph data
    text_glyph_starts: number[];
    text_glyph_counts: number[];
    text_scales: number[];

    // ═══════════════════════════════════════════════════════════
    // TRANSFORM HIERARCHY
    // ═══════════════════════════════════════════════════════════

    // Parent EntityId for each entity (encoded as number or null)
    parent_ids: (number | null)[];

    // Local transform relative to parent
    local_transforms: number[];

    // Pre-computed world transform (for render)
    world_transforms: number[];

    // Dirty flags for hierarchical updates
    dirty_hierarchy: boolean[];

    // ═══════════════════════════════════════════════════════════
    // SPATIAL INDEX
    // ═══════════════════════════════════════════════════════════

    // Entity bounds for SpatialHash [min_x, min_y, max_x, max_y]
    bounds: number[];

    // ═══════════════════════════════════════════════════════════
    // COLD DATA (Lazy-loaded on selection)
    // ═══════════════════════════════════════════════════════════

    // Architecture metadata (C4 model, entity types, etc.)
    arch_data: (ArchitectureData | null)[];

    // String pool for entity names/labels
    string_pool: {
        buffer: string;      // All strings concatenated
        offsets: number[];   // [start, length] per entity
    };

    // ═══════════════════════════════════════════════════════════
    // MANAGEMENT
    // ═══════════════════════════════════════════════════════════

    // Generation counters for EntityId validation
    generations: number[];

    // Entity alive status (for deserialization)
    alive_indices: number[];

    // Draw order (z-index)
    draw_order: number[];

    // Total alive count
    alive_count: number;
}

// ═══════════════════════════════════════════════════════════════════════
// SERIALIZATION: How JSON maps to WASM Internal Format
// ═══════════════════════════════════════════════════════════════════════

/*
 * Conversion flow:
 *
 * JSON (User-facing)          WASM Internal (Runtime)
 * ──────────────────          ─────────────────────────
 * shapes: { id → Shape }  →   transforms[], metadata[], colors[]
 *                           →   parent_ids[], local_transforms[]
 *                           →   bounds[]
 *                           →   string_pool
 *
 * Key optimization: Single pass through shapes array
 * populates all parallel arrays in O(n) time.
 */
```

### 3.3 Shape Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE - Base type for all canvas elements
// ═══════════════════════════════════════════════════════════════════════════════

type ShapeId = `${ShapeType}:${string}`;
type ShapeType = 'rectangle' | 'circle' | 'ellipse' | 'path' | 'text' | 'image' | 'group' | 'connector';

interface ArchFlowShape {
    id: ShapeId;
    type: ShapeType;
    parentId: PageId | ShapeId;  // Can be in a page or a group
    index: string;  // e.g., "a0", "a1", "b0" for z-ordering
    
    // Transform properties
    x: number;
    y: number;
    width: number;
    height: number;
    rotation: number;
    scaleX: number;
    scaleY: number;
    
    // Appearance
    opacity: number;
    visible: boolean;
    locked: boolean;
    
    // Type-specific properties
    props: ShapeProps;
    
    // Custom data (for extensions, plugins)
    meta: Record<string, unknown>;
    
    // Timestamps
    createdAt: string;
    modifiedAt: string;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE PROPS - Type-specific properties
// ═══════════════════════════════════════════════════════════════════════════════

type ShapeProps = 
    | RectangleProps
    | CircleProps
    | EllipseProps
    | PathProps
    | TextProps
    | ImageProps
    | GroupProps
    | ConnectorProps;

interface RectangleProps {
    // Base
    cornerRadius: number;
    
    // Fill
    fillColor: string | null;
    fillOpacity: number;
    
    // Border
    borderColor: string;
    borderWidth: number;
    borderStyle: 'solid' | 'dashed' | 'dotted';
    
    // Shadow
    shadow: ShadowConfig | null;
}

interface CircleProps {
    radius: number;
    fillColor: string | null;
    fillOpacity: number;
    borderColor: string;
    borderWidth: number;
    borderStyle: 'solid' | 'dashed' | 'dotted';
}

interface PathProps {
    points: Vec2[];          // Array of points
    closed: boolean;         // Close the path
    smooth: boolean;         // Apply smoothing
    fillColor: string | null;
    strokeColor: string;
    strokeWidth: number;
    strokeStyle: 'solid' | 'dashed' | 'dotted';
    lineCap: 'butt' | 'round' | 'square';
    lineJoin: 'miter' | 'round' | 'bevel';
}

interface TextProps {
    content: string;
    fontFamily: string;
    fontSize: number;
    fontWeight: number | 'normal' | 'bold';
    textColor: string;
    textAlign: 'left' | 'center' | 'right';
    verticalAlign: 'top' | 'middle' | 'bottom';
    lineHeight: number;
    maxWidth: number | null;
    overflow: 'wrap' | 'ellipsis' | 'none';
    link: string | null;
}

interface ImageProps {
    assetId: AssetId;
    naturalWidth: number;
    naturalHeight: number;
    cropX: number;
    cropY: number;
    cropWidth: number;
    cropHeight: number;
    opacity: number;
    preserveAspectRatio: boolean;
}

interface GroupProps {
    children: ShapeId[];
    expanded: boolean;  // Show children inline or collapsed
}

interface ConnectorProps {
    fromShape: ShapeId;
    toShape: ShapeId;
    fromAnchor: AnchorPoint | null;  // null = auto
    toAnchor: AnchorPoint | null;
    pathType: 'straight' | 'elbow' | 'curved';
    strokeColor: string;
    strokeWidth: number;
    strokeStyle: 'solid' | 'dashed' | 'dotted';
    startMarker: MarkerType;
    endMarker: MarkerType;
    label: string | null;
    labelPosition: number;  // 0-1 along the path
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUPPORTING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

interface Vec2 {
    x: number;
    y: number;
}

interface AnchorPoint {
    x: number;  // 0-1 normalized position on shape bounds
    y: number;
    side?: 'top' | 'bottom' | 'left' | 'right';  // For precision
}

type MarkerType = 'none' | 'arrow' | 'filled-arrow' | 'dot' | 'diamond' | 'open-diamond';

interface ShadowConfig {
    color: string;
    blur: number;
    offsetX: number;
    offsetY: number;
}
```

### 3.4 Binding Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// BINDING - Connections between shapes (connectors, groups, etc.)
// ═══════════════════════════════════════════════════════════════════════════════

type BindingId = `${BindingType}:${string}`;
type BindingType = 'connector' | 'constraint' | 'alignment' | 'distribution';

interface ArchFlowBinding {
    id: BindingId;
    type: BindingType;
    fromId: ShapeId;  // Source shape
    toId: ShapeId;    // Target shape
    
    // Binding-specific properties
    props: BindingProps;
    
    // Metadata
    meta: Record<string, unknown>;
    createdAt: string;
    modifiedAt: string;
}

type BindingProps = ConnectorBindingProps | ConstraintBindingProps | AlignmentBindingProps;

interface ConnectorBindingProps {
    // Connects two shapes with a line
    fromAnchor: AnchorPoint | null;
    toAnchor: AnchorPoint | null;
    pathType: 'straight' | 'elbow' | 'curved';
    // Dynamic: auto-updates when shapes move
    dynamic: boolean;
}

interface ConstraintBindingProps {
    // Constrains movement (e.g., "shape B always follows shape A")
    constraintType: 'follow' | 'limit' | 'mirror';
    offset: Vec2 | null;
    axes: ('x' | 'y' | 'both');
}

interface AlignmentBindingProps {
    // Aligns shapes relative to each other
    alignmentType: 'left' | 'right' | 'top' | 'bottom' | 'center-x' | 'center-y';
    referenceShape: ShapeId;
}
```

### 3.5 Page Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// PAGE - Container for shapes (can have multiple pages/boards)
// ═══════════════════════════════════════════════════════════════════════════════

type PageId = `page:${string}`;

interface ArchFlowPage {
    id: PageId;
    name: string;
    backgroundColor: string;
    gridSize: number | null;  // null = no grid
    gridVisible: boolean;

    // References to top-level shapes (children of page, not groups)
    shapeIds: ShapeId[];

    // Page-specific metadata
    meta: Record<string, unknown>;

    // ═══════════════════════════════════════════════════════════════════════
    // WASM-SPECIFIC: SpatialHash bounds for this page
    // ═══════════════════════════════════════════════════════════════════════
    spatialBounds: {
        minX: number;
        minY: number;
        maxX: number;
        maxY: number;
    };
}
```

### 3.5.1 Logic System Integration

The page also stores the **Logic Bricks wiring** for behavior:

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// LOGIC BRICKS - Sensor → Controller → Actuator connections
// ═══════════════════════════════════════════════════════════════════════════════

interface LogicMappingTable {
    // ═══════════════════════════════════════════════════════════════════════
    // SENSOR DEFINITIONS
    // ═══════════════════════════════════════════════════════════════════════
    sensors: SensorConfig[];

    // ═══════════════════════════════════════════════════════════════════════
    // CONTROLLER DEFINITIONS
    // ═══════════════════════════════════════════════════════════════════════
    controllers: ControllerConfig[];

    // ═══════════════════════════════════════════════════════════════════════
    // ACTUATOR DEFINITIONS
    // ═══════════════════════════════════════════════════════════════════════
    actuators: ActuatorConfig[];

    // ═══════════════════════════════════════════════════════════════════════
    // WIRING TABLE: Connections between sensors, controllers, actuators
    // ═══════════════════════════════════════════════════════════════════════
    connections: LogicConnection[];
}

type SensorConfig = {
    id: SensorId;
    type: 'mouse-over' | 'mouse-click' | 'touch' | 'proximity' | 'radar' | 'keyboard' | 'double-tap' | 'long-press' | 'right-click';
    config: Record<string, unknown>;
};

type ControllerConfig = {
    id: ControllerId;
    type: 'direct' | 'and' | 'or' | 'not' | 'blinky' | 'debounce' | 'hysteresis' | 'threshold' | 'pattern' | 'custom';
    config: Record<string, unknown>;
};

type ActuatorConfig = {
    id: ActuatorId;
    type: 'highlight' | 'select' | 'emit-event' | 'play-sound' | 'navigate' | 'execute-js' | 'custom';
    config: Record<string, unknown>;
};

type LogicConnection = {
    sensorId: SensorId;
    controllerId: ControllerId;
    actuatorId: ActuatorId;
    targetShapeId: ShapeId;
    priority: number;  // Higher = evaluated first
    enabled: boolean;
};

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * SPATIAL QUERY INTEGRATION
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * The data model integrates with SpatialHash for efficient sensor queries:
 *
 * 1. Touch/Collision Sensors:
 *    - Query: spatial_hash.query_rect(entity_bounds)
 *    - Result: All entities overlapping with this entity
 *
 * 2. Proximity Sensors:
 *    - Query: spatial_hash.query_circle(entity_center, radius)
 *    - Result: All entities within radius
 *
 * 3. Radar Sensors:
 *    - Query: spatial_hash.query_sector(entity_center, direction, fov, radius)
 *    - Result: All entities in directional cone
 *
 * Optimization: SpatialHash maintains two indexes:
 *   - Engine SpatialHash (64px cells): Rendering hit-testing
 *   - Logic SpatialHash (40px cells): Sensor collision detection
 *
 * Both are synchronized via draw_order which contains only alive entities.
 */
```

### 3.6 Asset Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// ASSET - Images, videos, fonts, and other external resources
// ═══════════════════════════════════════════════════════════════════════════════

type AssetId = `${AssetType}:${string}`;
type AssetType = 'image' | 'video' | 'font' | 'embed';

interface ArchFlowAsset {
    id: AssetId;
    type: AssetType;
    
    // Source information
    source: 'upload' | 'url' | 'library';
    originalUrl: string | null;
    
    // Storage
    storageType: 'data-url' | 'external' | 'reference';
    storageKey: string;  // Key for external storage
    dataUrl: string | null;  // For data-url storage
    
    // Metadata
    mimeType: string;
    width: number | null;
    height: number | null;
    fileSize: number | null;
    checksum: string | null;
    
    // Timestamps
    createdAt: string;
    lastAccessedAt: string | null;
}
```

### 3.7 Component Library Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// COMPONENT LIBRARY - Reusable templates and components (like Figma)
// ═══════════════════════════════════════════════════════════════════════════════

interface ComponentLibrary {
    id: string;
    name: string;
    version: string;
    description: string | null;
    author: string;
    
    // Components in this library
    components: Component[];
    
    // Dependencies on other libraries
    dependencies: LibraryDependency[];
    
    // Metadata
    createdAt: string;
    updatedAt: string;
    tags: string[];
}

interface Component {
    id: string;
    name: string;
    description: string | null;
    
    // Component definition (snapshot of a shape or group)
    definition: ComponentDefinition;
    
    // Variants (like Figma's ComponentSets)
    variants: ComponentVariant[];
    
    // Properties that can be overridden
    overridableProperties: string[];
    
    // Preview image
    previewAssetId: AssetId | null;
    
    // Metadata
    author: string;
    tags: string[];
    usageCount: number;
}

interface ComponentDefinition {
    // Can be a single shape or a group
    type: 'single' | 'group';
    shape: ArchFlowShape | null;
    group: {
        shape: ArchFlowShape;
        children: ShapeId[];
    } | null;
    
    // Default values for overridable properties
    defaultProps: Record<string, unknown>;
}

interface ComponentVariant {
    id: string;
    name: string;  // e.g., "Primary", "Secondary", "Small", "Large"
    props: Record<string, unknown>;
}

interface LibraryDependency {
    libraryId: string;
    minVersion: string;
}
```

### 3.8 Camera Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// CAMERA - Viewport state (can have multiple cameras for different views)
// ═══════════════════════════════════════════════════════════════════════════════

type CameraId = `camera:${string}`;

interface ArchFlowCamera {
    id: CameraId;
    pageId: PageId;
    
    // Position and zoom
    x: number;
    y: number;
    zoom: number;
    
    // Constraints
    minZoom: number;
    maxZoom: number;
    
    // Metadata
    name: string | null;
}
```

### 3.9 Instance Schema

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// INSTANCE - User-specific state (not persisted in document)
// ═══════════════════════════════════════════════════════════════════════════════

interface ArchFlowInstance {
    id: string;
    userId: string;
    
    // Current page
    currentPageId: PageId;
    
    // Camera for this user
    cameraId: CameraId;
    
    // Selection state
    selectedShapeIds: ShapeId[];
    
    // Hover state
    hoveredShapeId: ShapeId | null;
    
    // Tool state
    activeTool: string;
    toolState: Record<string, unknown>;
    
    // Presence
    cursor: Vec2 | null;
    cursorColor: string;
    
    // Timestamps
    lastActiveAt: string;
}
```

---

## 4. Example Documents

### 4.1 Simple Document with Shapes and Connector

```json
{
    "version": 1,
    "schema": {
        "version": 1,
        "shapeTypes": {
            "rectangle": { /* schema */ },
            "circle": { /* schema */ },
            "connector": { /* schema */ }
        },
        "bindingTypes": {
            "connector": { /* schema */ }
        },
        "migrations": []
    },
    "store": {
        "pages": {
            "page:main": {
                "id": "page:main",
                "name": "Page 1",
                "backgroundColor": "#ffffff",
                "gridSize": 20,
                "gridVisible": true,
                "shapeIds": ["shape:rect1", "shape:circle1", "shape:conn1"],
                "meta": {}
            }
        },
        "shapes": {
            "shape:rect1": {
                "id": "shape:rect1",
                "type": "rectangle",
                "parentId": "page:main",
                "index": "a0",
                "x": 100,
                "y": 100,
                "width": 150,
                "height": 80,
                "rotation": 0,
                "scaleX": 1,
                "scaleY": 1,
                "opacity": 1,
                "visible": true,
                "locked": false,
                "props": {
                    "cornerRadius": 8,
                    "fillColor": "#2196F3",
                    "fillOpacity": 1,
                    "borderColor": "#1976D2",
                    "borderWidth": 2,
                    "borderStyle": "solid",
                    "shadow": { "color": "#00000033", "blur": 10, "offsetX": 0, "offsetY": 4 }
                },
                "meta": { "authorId": "user123" },
                "createdAt": "2024-01-01T00:00:00Z",
                "modifiedAt": "2024-01-01T00:00:00Z"
            },
            "shape:circle1": {
                "id": "shape:circle1",
                "type": "circle",
                "parentId": "page:main",
                "index": "a1",
                "x": 400,
                "y": 100,
                "width": 80,
                "height": 80,
                "rotation": 0,
                "scaleX": 1,
                "scaleY": 1,
                "opacity": 1,
                "visible": true,
                "locked": false,
                "props": {
                    "radius": 40,
                    "fillColor": "#4CAF50",
                    "fillOpacity": 1,
                    "borderColor": "#388E3C",
                    "borderWidth": 2,
                    "borderStyle": "solid"
                },
                "meta": {},
                "createdAt": "2024-01-01T00:00:00Z",
                "modifiedAt": "2024-01-01T00:00:00Z"
            },
            "shape:conn1": {
                "id": "shape:conn1",
                "type": "connector",
                "parentId": "page:main",
                "index": "a2",
                "x": 0,
                "y": 0,
                "width": 0,
                "height": 0,
                "rotation": 0,
                "scaleX": 1,
                "scaleY": 1,
                "opacity": 1,
                "visible": true,
                "locked": false,
                "props": {
                    "fromShape": "shape:rect1",
                    "toShape": "shape:circle1",
                    "fromAnchor": null,
                    "toAnchor": null,
                    "pathType": "elbow",
                    "strokeColor": "#666666",
                    "strokeWidth": 2,
                    "strokeStyle": "solid",
                    "startMarker": "none",
                    "endMarker": "arrow",
                    "label": null,
                    "labelPosition": 0.5
                },
                "meta": {},
                "createdAt": "2024-01-01T00:00:00Z",
                "modifiedAt": "2024-01-01T00:00:00Z"
            }
        },
        "bindings": {},
        "assets": {},
        "cameras": {},
        "instances": {}
    },
    "meta": {
        "id": "doc:example",
        "name": "Example Diagram",
        "created": "2024-01-01T00:00:00Z",
        "modified": "2024-01-01T00:00:00Z",
        "creator": "user123",
        "tags": ["diagram", "example"]
    }
}
```

### 4.2 Component Library Export

```json
{
    "library": {
        "id": "lib:ui-components",
        "name": "UI Components",
        "version": "1.0.0",
        "description": "Common UI components for wireframing",
        "author": "design-team",
        "components": [
            {
                "id": "comp:button",
                "name": "Button",
                "description": "Standard button component",
                "definition": {
                    "type": "group",
                    "shape": {
                        "id": "shape:button-bg",
                        "type": "rectangle",
                        "props": {
                            "cornerRadius": 4,
                            "fillColor": "#2196F3",
                            "borderColor": "#1976D2",
                            "borderWidth": 0
                        }
                    },
                    "children": ["shape:button-bg", "shape:button-text"]
                },
                "variants": [
                    { "id": "v1", "name": "Primary", "props": { "fillColor": "#2196F3" } },
                    { "id": "v2", "name": "Secondary", "props": { "fillColor": "#757575" } },
                    { "id": "v3", "name": "Danger", "props": { "fillColor": "#F44336" } }
                ],
                "overridableProperties": ["fillColor", "textContent", "width"],
                "previewAssetId": null,
                "author": "design-team",
                "tags": ["ui", "button", "interactive"],
                "usageCount": 42
            }
        ],
        "dependencies": [],
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z",
        "tags": ["ui", "components"]
    }
}
```

---

## 5. API Mapping

### 5.1 Fluent API → Data Model

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// FLUENT API EXAMPLES → HOW THEY MAP TO THE DATA MODEL
// ═══════════════════════════════════════════════════════════════════════════════

// Creating a shape
const rect = ArchFlow.createShape('rectangle')
    .position(100, 200)
    .size(150, 80)
    .fillColor(0x2196F3)
    .border(2, 0x1976D2)
    .cornerRadius(8)
    .shadow('md')
    .build();

// Maps to:
{
    "id": "shape:generated-id",
    "type": "rectangle",
    "x": 100,
    "y": 200,
    "width": 150,
    "height": 80,
    "props": {
        "fillColor": "#2196F3",
        "borderColor": "#1976D2",
        "borderWidth": 2,
        "cornerRadius": 8,
        "shadow": { "color": "#00000033", "blur": 10, "offsetX": 0, "offsetY": 4 }
    }
}

// Connecting shapes
const connector = rect.connectTo(circle).build();

// Maps to:
{
    "id": "shape:connector-id",
    "type": "connector",
    "props": {
        "fromShape": "shape:rect-id",
        "toShape": "shape:circle-id",
        "pathType": "elbow",
        "endMarker": "arrow"
    }
}

// Creating a sticky note
const note = ArchFlow.createStickyNote()
    .position(300, 400)
    .size(200, 150)
    .textContent("Remember to...")
    .build();

// Maps to a group with rectangle + text
{
    "id": "shape:note-id",
    "type": "group",
    "props": {
        "children": ["shape:rect-id", "shape:text-id"],
        "expanded": true
    }
}
```

---

## 6. Export/Import Formats

### 6.1 JSON (Default)

```typescript
function exportToJSON(document: ArchFlowDocument): string {
    return JSON.stringify(document, null, 2);
}

function importFromJSON(json: string): ArchFlowDocument {
    return JSON.parse(json);
}
```

### 6.2 Compressed JSON (for large documents)

```typescript
import { gzip, ungzip } from 'fflate';

async function exportToCompressedJSON(document: ArchFlowDocument): Promise<Uint8Array> {
    const json = JSON.stringify(document);
    return gzip(new TextEncoder().encode(json));
}

async function importFromCompressedJSON(data: Uint8Array): Promise<ArchFlowDocument> {
    const json = new TextDecoder().decode(ungzip(data));
    return JSON.parse(json);
}
```

### 6.3 SVG Export (for sharing)

```typescript
function exportToSVG(document: ArchFlowDocument, pageId: string): string {
    const page = document.store.pages[pageId];
    const shapes = page.shapeIds
        .map(id => document.store.shapes[id])
        .filter(s => s.visible);

    const svgElements = shapes.map(shape => {
        switch (shape.type) {
            case 'rectangle':
                return `<rect x="${shape.x}" y="${shape.y}" width="${shape.width}" height="${shape.height}" 
                        fill="${shape.props.fillColor}" stroke="${shape.props.borderColor}" />`;
            case 'text':
                return `<text x="${shape.x}" y="${shape.y}" fill="${shape.props.textColor}">${shape.props.content}</text>`;
            // ... other types
        }
    }).join('\n');

    return `<svg xmlns="http://www.w3.org/2000/svg">
        ${svgElements}
    </svg>`;
}
```

### 6.4 Library Export/Import

```typescript
function exportLibrary(library: ComponentLibrary): Uint8Array {
    // Can be plain JSON or compressed
    const json = JSON.stringify(library, null, 2);
    return new TextEncoder().encode(json);
}

function importLibrary(data: Uint8Array): ComponentLibrary {
    const json = new TextDecoder().decode(data);
    return JSON.parse(json);
}

// Also support library as document embedded
function exportLibraryAsDocument(library: ComponentLibrary): ArchFlowDocument {
    // Convert library to a special document format
    return {
        version: 1,
        schema: { /* library schema */ },
        store: {
            pages: {
                'page:library': {
                    id: 'page:library',
                    name: library.name,
                    // ...
                }
            },
            // Library components stored as special shapes
            shapes: libraryToShapes(library),
            // ...
        },
        meta: { /* ... */ }
    };
}
```

---

## 7. Recommendations

### 7.1 Primary Format: JSON + Optional Binary

**Pros**:
- Human-readable for debugging
- Easy to version control
- Well-supported across languages
- tldraw and Excalidraw both use it

**Cons**:
- Larger file sizes
- Slower parsing for very large documents

**Recommendation**: Use JSON as default, with optional binary compression.

**WASM Optimization**: Include parallel arrays format for internal WASM state:

```typescript
// Binary format structure for WASM:
// [header: 4 bytes] [transforms: 4*n bytes] [metadata: 4*n bytes] [colors: 4*n bytes] ...
```

### 7.2 Store Structure: Normalized Records (like tldraw)

**Why**:
- O(1) lookup by ID
- Efficient updates (only changed records)
- Easy to implement undo/redo
- Natural fit for collaborative editing

**WASM Mapping**: Single pass through records populates parallel arrays.

### 7.3 Component Library: Standalone Files

**Structure**:
```
libraries/
├── ui-components/
│   ├── library.json      # Library metadata
│   ├── components/       # Individual component definitions
│   │   ├── button.json
│   │   ├── card.json
│   │   └── input.json
│   └── preview.png
└── diagrams/
    ├── library.json
    └── components/
        ├── flow-chart.json
        └── uml-diagram.json
```

### 7.4 WASM-Specific Optimization Strategy

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// PERFORMANCE RECOMMENDATIONS FOR 100K+ ENTITIES AT 60 FPS
// ═══════════════════════════════════════════════════════════════════════════════

interface WASMOptimizationStrategy {
    // ═══════════════════════════════════════════════════════════════════════
    // 1. SERIALIZATION: Binary format for large documents
    // ═══════════════════════════════════════════════════════════════════════
    serialization: {
        format: 'json' | 'binary';
        compression: 'none' | 'gzip' | 'lz4';
        parallel_arrays: boolean;  // Direct SoA mapping
    };

    // ═══════════════════════════════════════════════════════════════════════
    // 2. SPATIAL INDEXING: Pre-built for sensors
    // ═══════════════════════════════════════════════════════════════════════
    spatial_index: {
        engine_cell_size: 64;    // Rendering
        logic_cell_size: 40;     // Collision detection
        pre_built: boolean;      // Include in save file
    };

    // ═══════════════════════════════════════════════════════════════════════
    // 3. HIERARCHY: Lazy world transform computation
    // ═══════════════════════════════════════════════════════════════════════
    hierarchy: {
        store_local: boolean;
        compute_world_on_load: boolean;
        dirty_propagation: boolean;
    };

    // ═══════════════════════════════════════════════════════════════════════
    // 4. LAZY LOADING: Cold data on demand
    // ═══════════════════════════════════════════════════════════════════════
    lazy_loading: {
        arch_data: boolean;      // C4 model data
        string_pool: boolean;    // Names/labels
        components: boolean;     // Library components
    };
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * BENCHMARK TARGETS
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * | Scenario                    | Current | Target | Technique
 * |----------------------------|---------|--------|----------------------------------
 * | Load 100K entities         | TBD     | <500ms | Binary + parallel arrays
 * | Query nearby (SpatialHash) | O(n²)   | O(k)   | Pre-built spatial index
 * | Update hierarchy           | O(n)    | O(d)   | Dirty propagation (d=depth)
 * | Serialize document         | TBD     | <100ms | Binary format
 * | Frame render (no changes)  | TBD     | <16ms  | SoA + pre-computed world transforms
 */
```

### 7.5 Versioning Strategy

```typescript
interface Migration {
    fromVersion: number;
    toVersion: number;
    migrate: (document: unknown) => unknown;
}

const migrations: Migration[] = [
    {
        fromVersion: 1,
        toVersion: 2,
        migrate: (doc) => {
            // Transform v1 to v2
            return transformedDoc;
        }
    }
];
```

### 7.6 Data Model Compliance Checklist

```typescript
interface ComplianceChecklist {
    // ═══════════════════════════════════════════════════════════════════════
    // USER-FACING (JSON)
    // ═══════════════════════════════════════════════════════════════════════
    userFacing: {
        [x] Record-based store (O(1) lookup by ID)
        [x] Shape types: rectangle, circle, ellipse, path, text, image, group, connector
        [x] Binding types: connector, constraint, alignment, distribution
        [x] Component library support with variants
        [x] Page/board containers
        [x] Asset management for images/videos
        [x] Camera/viewport state
        [x] Custom metadata (meta object)
        [x] Version migration support
    };

    // ═══════════════════════════════════════════════════════════════════════
    // WASM-INTERNAL (Runtime)
    // ═══════════════════════════════════════════════════════════════════════
    wasmInternal: {
        [x] Structure of Arrays (SoA) for transforms, metadata, colors
        [x] Bit-packed metadata (shape:4 | layer:4 | visible:1 | selected:1 | locked:1)
        [x] Parent-child hierarchy with dirty tracking
        [x] Pre-computed world transforms
        [x] String pool for zero-allocation names
        [x] Pre-built SpatialHash indexes (64px engine, 40px logic)
        [x] Generation counters for EntityId validation
        [x] Draw order for z-index rendering
        [x] Cold data lazy loading (arch_data, string_pool)
    };

    // ═══════════════════════════════════════════════════════════════════════
    // LOGIC BRICKS (Behavior)
    // ═══════════════════════════════════════════════════════════════════════
    logicBricks: {
        [x] Sensor configurations (mouse-over, touch, proximity, radar, etc.)
        [x] Controller types (direct, and, or, not, blinky, debounce, hysteresis, threshold, pattern, custom)
        [x] Actuator definitions (highlight, select, emit-event, play-sound, navigate, execute-js, custom)
        [x] Wiring table (Sensor → Controller → Actuator connections)
        [x] Priority-based evaluation
        [x] Spatial query integration (SpatialHash → Sensor evaluation)
    };
}
```

---

## 8. References

- [tldraw Schema Documentation](https://github.com/tldraw/tldraw/blob/main/packages/tlschema/DOCS.md)
- [tldraw Store API](https://github.com/tldraw/tldraw/blob/main/packages/store/api-report.api.md)
- [Excalidraw JSON Schema](https://github.com/excalidraw/excalidraw/blob/master/dev-docs/docs/codebase/json-schema.mdx)
- [Figma Documentation](https://www.figma.com/developers/api)

---

*Document created: 2026-02-02*  
*Based on analysis of tldraw, Excalidraw, Figma, and draw.io data models*  
*Updated with WASM internal architecture integration*
