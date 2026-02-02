/**
 * Connection Store - Zustand store for managing connections between entities
 *
 * Stores connection data with routing information and visual state.
 */

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { v4 as uuidv4 } from "uuid";

/** Connection point on an entity */
export interface ConnectionPoint {
  x: number;
  y: number;
  side: "top" | "bottom" | "left" | "right";
}

/** Connection between two entities */
export interface Connection {
  id: string;
  sourceEntityId: number;
  targetEntityId: number;
  sourcePoint: ConnectionPoint;
  targetPoint: ConnectionPoint;
  style: ConnectionStyle;
  routingPoints?: RoutingPoint[];
}

/** Visual style for a connection */
export interface ConnectionStyle {
  strokeColor: string;
  strokeWidth: number;
  hasArrow: boolean;
  arrowSize: number;
  lineType: "solid" | "dashed" | "dotted";
  animation?: "flow" | "pulse";
}

/** Routing point for custom connection paths */
export interface RoutingPoint {
  x: number;
  y: number;
}

/** State for connection creation */
interface ConnectionCreationState {
  isCreating: boolean;
  sourceEntityId: number | null;
  sourcePoint: ConnectionPoint | null;
  tempEndPoint: { x: number; y: number } | null;
}

/** Connection Store interface */
interface ConnectionStore {
  // Connections
  connections: Connection[];

  // Creation state
  creation: ConnectionCreationState;

  // Selected connections
  selectedConnectionIds: string[];

  // Actions - Connections
  addConnection: (connection: Omit<Connection, "id">) => string;
  removeConnection: (id: string) => void;
  updateConnection: (id: string, updates: Partial<Connection>) => void;
  clearConnections: () => void;

  // Actions - Creation
  startConnection: (entityId: number, point: ConnectionPoint) => void;
  updateTempEndPoint: (point: { x: number; y: number }) => void;
  completeConnection: (
    targetEntityId: number,
    targetPoint: ConnectionPoint,
  ) => string | null;
  cancelConnection: () => void;

  // Actions - Selection
  selectConnection: (id: string, additive?: boolean) => void;
  deselectConnection: (id: string) => void;
  clearSelection: () => void;

  // Queries
  getConnectionsForEntity: (entityId: number) => Connection[];
  getConnection: (id: string) => Connection | undefined;
}

/** Default connection style */
const defaultConnectionStyle: ConnectionStyle = {
  strokeColor: "#13b6ec",
  strokeWidth: 2,
  hasArrow: true,
  arrowSize: 8,
  lineType: "solid",
};

export const useConnectionStore = create<ConnectionStore>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    connections: [],
    creation: {
      isCreating: false,
      sourceEntityId: null,
      sourcePoint: null,
      tempEndPoint: null,
    },
    selectedConnectionIds: [],

    // Connection actions
    addConnection: (connection) => {
      const id = uuidv4();
      set((state) => ({
        connections: [...state.connections, { ...connection, id }],
      }));
      return id;
    },

    removeConnection: (id) => {
      set((state) => ({
        connections: state.connections.filter((c) => c.id !== id),
        selectedConnectionIds: state.selectedConnectionIds.filter(
          (sid) => sid !== id,
        ),
      }));
    },

    updateConnection: (id, updates) => {
      set((state) => ({
        connections: state.connections.map((c) =>
          c.id === id ? { ...c, ...updates } : c,
        ),
      }));
    },

    clearConnections: () => {
      set({ connections: [], selectedConnectionIds: [] });
    },

    // Creation actions
    startConnection: (entityId, point) => {
      set({
        creation: {
          isCreating: true,
          sourceEntityId: entityId,
          sourcePoint: point,
          tempEndPoint: null,
        },
      });
    },

    updateTempEndPoint: (point) => {
      set((state) => ({
        creation: {
          ...state.creation,
          tempEndPoint: point,
        },
      }));
    },

    completeConnection: (targetEntityId, targetPoint) => {
      const { creation, connections } = get();

      if (
        !creation.isCreating ||
        !creation.sourceEntityId ||
        !creation.sourcePoint
      ) {
        return null;
      }

      // Check if connection already exists
      const exists = connections.some(
        (c) =>
          c.sourceEntityId === creation.sourceEntityId &&
          c.targetEntityId === targetEntityId,
      );

      if (exists) {
        get().cancelConnection();
        return null;
      }

      const id = uuidv4();
      const newConnection: Connection = {
        id,
        sourceEntityId: creation.sourceEntityId,
        targetEntityId,
        sourcePoint: creation.sourcePoint,
        targetPoint,
        style: { ...defaultConnectionStyle },
      };

      set((state) => ({
        connections: [...state.connections, newConnection],
        creation: {
          isCreating: false,
          sourceEntityId: null,
          sourcePoint: null,
          tempEndPoint: null,
        },
      }));

      return id;
    },

    cancelConnection: () => {
      set({
        creation: {
          isCreating: false,
          sourceEntityId: null,
          sourcePoint: null,
          tempEndPoint: null,
        },
      });
    },

    // Selection actions
    selectConnection: (id, additive = false) => {
      set((state) => ({
        selectedConnectionIds: additive
          ? [...state.selectedConnectionIds, id]
          : [id],
      }));
    },

    deselectConnection: (id) => {
      set((state) => ({
        selectedConnectionIds: state.selectedConnectionIds.filter(
          (sid) => sid !== id,
        ),
      }));
    },

    clearSelection: () => {
      set({ selectedConnectionIds: [] });
    },

    // Queries
    getConnectionsForEntity: (entityId) => {
      return get().connections.filter(
        (c) => c.sourceEntityId === entityId || c.targetEntityId === entityId,
      );
    },

    getConnection: (id) => {
      return get().connections.find((c) => c.id === id);
    },
  })),
);

/**
 * Smart routing utility functions for connections
 */
export const connectionRouting = {
  /** Calculate a simple straight-line path */
  calculateStraightPath(
    _source: ConnectionPoint,
    _target: ConnectionPoint,
  ): RoutingPoint[] {
    return [];
  },

  /** Calculate an orthogonal (right-angle) path */
  calculateOrthogonalPath(
    source: ConnectionPoint,
    target: ConnectionPoint,
    padding = 20,
  ): RoutingPoint[] {
    const points: RoutingPoint[] = [];
    const midX = (source.x + target.x) / 2;
    const midY = (source.y + target.y) / 2;

    // Determine routing based on source and target sides
    if (source.side === "right" && target.side === "left") {
      // Horizontal routing
      points.push({ x: source.x + padding, y: source.y });
      points.push({ x: midX, y: source.y });
      points.push({ x: midX, y: target.y });
      points.push({ x: target.x - padding, y: target.y });
    } else if (source.side === "top" && target.side === "bottom") {
      // Vertical routing
      points.push({ x: source.x, y: source.y + padding });
      points.push({ x: source.x, y: midY });
      points.push({ x: target.x, y: midY });
      points.push({ x: target.x, y: target.y - padding });
    } else {
      // L-shaped routing for other combinations
      if (Math.abs(target.x - source.x) > Math.abs(target.y - source.y)) {
        // Horizontal first
        points.push({
          x: source.x + padding * Math.sign(target.x - source.x),
          y: source.y,
        });
        points.push({ x: midX, y: source.y });
        points.push({ x: midX, y: target.y });
        points.push({ x: target.x, y: target.y });
      } else {
        // Vertical first
        points.push({
          x: source.x,
          y: source.y + padding * Math.sign(target.y - source.y),
        });
        points.push({ x: source.x, y: midY });
        points.push({ x: target.x, y: midY });
        points.push({ x: target.x, y: target.y });
      }
    }

    return points;
  },

  /** Calculate a curved (Bezier) path */
  calculateCurvedPath(
    source: ConnectionPoint,
    target: ConnectionPoint,
    curvature = 0.5,
  ): {
    start: RoutingPoint;
    control1: RoutingPoint;
    control2: RoutingPoint;
    end: RoutingPoint;
  } {
    const midX = (source.x + target.x) / 2;
    const midY = (source.y + target.y) / 2;

    // Perpendicular offset for control points
    const dx = target.x - source.x;
    const dy = target.y - source.y;
    const perpendicularX = -dy * curvature;
    const perpendicularY = dx * curvature;

    return {
      start: { x: source.x, y: source.y },
      control1: { x: midX + perpendicularX, y: midY + perpendicularY },
      control2: { x: midX - perpendicularX, y: midY - perpendicularY },
      end: { x: target.x, y: target.y },
    };
  },
};
