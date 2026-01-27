/**
 * ArchFlow SDK - React Integration
 *
 * This module provides React hooks and components for the ArchFlow editor.
 */

import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { ArchFlowEditor, EditorOptions, ShapeData, Viewport, Selection, C4Level, GridOptions, EventCallback, EditorEventMap } from './index';

// === Context ===

interface ArchFlowContextValue {
  editor: ArchFlowEditor | null;
  isReady: boolean;
}

const ArchFlowContext = React.createContext<ArchFlowContextValue>({
  editor: null,
  isReady: false,
});

// === Hooks ===

/**
 * Hook to get the ArchFlow editor instance.
 */
export function useArchFlowEditor(): ArchFlowEditor | null {
  const context = React.useContext(ArchFlowContext);
  return context.editor;
}

/**
 * Hook to check if the editor is ready.
 */
export function useIsArchFlowReady(): boolean {
  const context = React.useContext(ArchFlowContext);
  return context.isReady;
}

/**
 * Hook to get the current selection.
 */
export function useSelection(): [string[], (selection: string[]) => void] {
  const editor = useArchFlowEditor();
  const [selection, setSelection] = useState<string[]>([]);

  useEffect(() => {
    if (!editor) return;

    const updateSelection = () => {
      setSelection(editor.getSelection().shapes);
    };

    const unsubscribe = editor.on('selectionchange', updateSelection);
    updateSelection();

    return unsubscribe;
  }, [editor]);

  const setSelectionFn = useCallback((newSelection: string[]) => {
    if (editor) {
      if (newSelection.length === 1) {
        editor.select(newSelection[0]);
      } else if (newSelection.length > 1) {
        editor.selectMultiple(newSelection);
      } else {
        editor.clearSelection();
      }
    }
  }, [editor]);

  return [selection, setSelectionFn];
}

/**
 * Hook to get the current viewport.
 */
export function useViewport(): [Viewport, (viewport: Partial<Viewport>) => void] {
  const editor = useArchFlowEditor();
  const [viewport, setViewportState] = useState<Viewport>({
    offset: { x: 0, y: 0 },
    zoom: 1.0,
    minZoom: 0.1,
    maxZoom: 10.0,
  });

  useEffect(() => {
    if (!editor) return;

    const updateViewport = () => {
      setViewportState(editor.getViewport());
    };

    const unsubscribe = editor.on('viewportchange', updateViewport);
    updateViewport();

    return unsubscribe;
  }, [editor]);

  const setViewport = useCallback((newViewport: Partial<Viewport>) => {
    if (editor) {
      editor.setViewport(newViewport);
    }
  }, [editor]);

  return [viewport, setViewport];
}

/**
 * Hook to get the current C4 level.
 */
export function useC4Level(): [C4Level, (level: C4Level) => void] {
  const editor = useArchFlowEditor();
  const [level, setLevel] = useState<C4Level>('context');

  useEffect(() => {
    if (!editor) return;

    const updateLevel = () => {
      // TODO: Get C4 level from editor
    };

    const unsubscribe = editor.on('c4levelchange', updateLevel);
    updateLevel();

    return unsubscribe;
  }, [editor]);

  const setLevelFn = useCallback((newLevel: C4Level) => {
    if (editor) {
      editor.setC4Level(newLevel);
    }
  }, [editor]);

  return [level, setLevelFn];
}

/**
 * Hook to get shapes by type.
 */
export function useShapesByType(): Record<string, ShapeData[]> {
  const editor = useArchFlowEditor();
  const [shapes, setShapes] = useState<ShapeData[]>([]);

  useEffect(() => {
    if (!editor) return;

    const updateShapes = () => {
      // TODO: Get all shapes from editor
    };

    const unsubscribe = editor.on('shapecreate', updateShapes);
    unsubscribe.add(editor.on('shapeupdate', updateShapes));
    unsubscribe.add(editor.on('shapedelete', updateShapes));
    updateShapes();

    return () => {
      // Cleanup
    };
  }, [editor]);

  return useMemo(() => {
    const byType: Record<string, ShapeData[]> = {};
    for (const shape of shapes) {
      if (!byType[shape.type]) {
        byType[shape.type] = [];
      }
      byType[shape.type].push(shape);
    }
    return byType;
  }, [shapes]);
}

/**
 * Hook to subscribe to editor events.
 */
export function useEditorEvent<K extends keyof EditorEventMap>(
  event: K,
  callback: EventCallback<EditorEventMap[K]>
): () => void {
  const editor = useArchFlowEditor();

  useEffect(() => {
    if (!editor) return () => {};

    return editor.on(event, callback);
  }, [editor, event, callback]);
}

// === Components ===

interface ArchFlowCanvasProps extends Omit<EditorOptions, 'canvas'> {
  children?: React.ReactNode;
  onReady?: (editor: ArchFlowEditor) => void;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * The main ArchFlow canvas component.
 *
 * Example usage:
 * ```tsx
 * <ArchFlowCanvas
 *   width={800}
 *   height={600}
 *   grid={{ type: 'dots', spacing: 20 }}
 *   c4Level="context"
 * >
 *   <Toolbar />
 * </ArchFlowCanvas>
 * ```
 */
export const ArchFlowCanvas: React.FC<ArchFlowCanvasProps> = ({
  width = '100%',
  height = '100%',
  backgroundColor = '#ffffff',
  grid,
  c4Level,
  children,
  onReady,
  className,
  style,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [editor, setEditor] = useState<ArchFlowEditor | null>(null);
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    if (!canvasRef.current) return;

    const newEditor = new ArchFlowEditor({
      canvas: canvasRef.current,
      width: typeof width === 'number' ? width : undefined,
      height: typeof height === 'number' ? height : undefined,
      backgroundColor,
      grid,
      c4Level,
    });

    setEditor(newEditor);
    setIsReady(true);

    if (onReady) {
      onReady(newEditor);
    }

    return () => {
      newEditor.destroy();
    };
  }, []);

  useEffect(() => {
    if (editor && grid) {
      editor.setGridConfig(grid);
    }
  }, [editor, grid]);

  useEffect(() => {
    if (editor && c4Level) {
      editor.setC4Level(c4Level);
    }
  }, [editor, c4Level]);

  const contextValue = useMemo(() => ({
    editor,
    isReady,
  }), [editor, isReady]);

  return (
    <ArchFlowContext.Provider value={contextValue}>
      <div
        className={className}
        style={{
          position: 'relative',
          width,
          height,
          backgroundColor,
          overflow: 'hidden',
          ...style,
        }}
      >
        <canvas
          ref={canvasRef}
          style={{
            display: 'block',
            width: '100%',
            height: '100%',
          }}
        />
        {children}
      </div>
    </ArchFlowContext.Provider>
  );
};

/**
 * Component that renders selection handles for the selected shapes.
 */
export const SelectionHandles: React.FC = () => {
  const editor = useArchFlowEditor();
  const [selection] = useSelection();

  if (!editor || selection.length === 0) {
    return null;
  }

  // TODO: Render selection handles for each selected shape

  return null;
};

/**
 * Component that renders the viewport controls.
 */
export const ViewportControls: React.FC = () => {
  const [viewport, setViewport] = useViewport();
  const editor = useArchFlowEditor();

  const handleZoomIn = useCallback(() => {
    editor?.zoomIn(1.2);
  }, [editor]);

  const handleZoomOut = useCallback(() => {
    editor?.zoomOut(1.2);
  }, [editor]);

  const handleZoomToFit = useCallback(() => {
    editor?.zoomToFit();
  }, [editor]);

  return (
    <div className="archflow-viewport-controls" style={{
      position: 'absolute',
      bottom: 16,
      right: 16,
      display: 'flex',
      gap: 4,
      backgroundColor: 'white',
      borderRadius: 4,
      boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
      padding: 4,
    }}>
      <button onClick={handleZoomOut} title="Zoom Out">−</button>
      <span style={{ padding: '0 8px', lineHeight: '28px' }}>
        {Math.round(viewport.zoom * 100)}%
      </span>
      <button onClick={handleZoomIn} title="Zoom In">+</button>
      <button onClick={handleZoomToFit} title="Zoom to Fit">⊡</button>
    </div>
  );
};

/**
 * Component that renders the C4 level selector.
 */
export const C4LevelSelector: React.FC = () => {
  const [level, setLevel] = useC4Level();

  const levels: C4Level[] = ['context', 'container', 'component', 'code'];

  return (
    <div className="archflow-c4-selector" style={{
      position: 'absolute',
      top: 16,
      left: '50%',
      transform: 'translateX(-50%)',
      display: 'flex',
      gap: 4,
      backgroundColor: 'white',
      borderRadius: 4,
      boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
      padding: 4,
    }}>
      {levels.map((l) => (
        <button
          key={l}
          onClick={() => setLevel(l)}
          style={{
            padding: '6px 12px',
            border: 'none',
            borderRadius: 4,
            backgroundColor: level === l ? '#3366cc' : 'transparent',
            color: level === l ? 'white' : '#333',
            cursor: 'pointer',
            textTransform: 'capitalize',
          }}
        >
          {l}
        </button>
      ))}
    </div>
  );
};

/**
 * Component that renders the grid controls.
 */
export const GridControls: React.FC = () => {
  const editor = useArchFlowEditor();

  const showDots = useCallback(() => {
    editor?.setGridConfig({ type: 'dots', spacing: 20, visible: true });
  }, [editor]);

  const showLines = useCallback(() => {
    editor?.setGridConfig({ type: 'lines', spacing: 50, visible: true });
  }, [editor]);

  const hideGrid = useCallback(() => {
    editor?.setGridConfig({ visible: false });
  }, [editor]);

  return (
    <div className="archflow-grid-controls" style={{
      position: 'absolute',
      top: 16,
      right: 16,
      display: 'flex',
      gap: 4,
      backgroundColor: 'white',
      borderRadius: 4,
      boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
      padding: 4,
    }}>
      <button onClick={showDots} title="Dot Grid">·</button>
      <button onClick={showLines} title="Line Grid">#</button>
      <button onClick={hideGrid} title="No Grid">✕</button>
    </div>
  );
};

// === Export ===

export {
  ArchFlowCanvas as Canvas,
  SelectionHandles,
  ViewportControls,
  C4LevelSelector,
  GridControls,
};
