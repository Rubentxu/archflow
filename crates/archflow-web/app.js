/**
 * ArchFlow Web - Main Application Entry Point
 *
 * Modular application architecture following code.html design
 */

import { initWasm } from "./index.js";
import { ToolPalette } from "./components/ToolPalette.js";
import { ComponentLibrary } from "./components/ComponentLibrary.js";
import { Canvas } from "./components/Canvas.js";
import { PropertiesPanel } from "./components/PropertiesPanel.js";
import { StatusBar } from "./components/StatusBar.js";
import { ContextMenu } from "./components/ContextMenu.js";
import { ContextMenu } from "./components/ContextMenu.js";
import { KeyboardManager } from "./core/KeyboardManager.js";
import { SelectionManager } from "./core/SelectionManager.js";
import { ZoomManager } from "./core/ZoomManager.js";

class ArchFlowApp {
  constructor() {
    this.editor = null;
    this.managers = {};
    this.components = {};
    this.isInitialized = false;
  }

  async init() {
    try {
      console.log("Initializing ArchFlow...");

      // Show loading state
      this.showLoading("Initializing application...");

      // Initialize WASM - wait for canvas to be available
      const canvas = await this.waitForElement("canvas");
      if (!canvas) {
        throw new Error("Canvas element not found");
      }
      const wasm = await initWasm(canvas);
      this.editor = wasm.editor;
      this.managers.wasm = wasm;

      console.log("WASM initialized");

      // Initialize core managers
      this.initManagers();

      // Initialize UI components
      this.initComponents();

      // Setup event listeners
      this.setupGlobalEvents();

      this.isInitialized = true;
      console.log("ArchFlow initialized successfully");

      // Hide loading state
      this.hideLoading();

      // Start render loop
      this.startRenderLoop();
    } catch (error) {
      console.error("Failed to initialize ArchFlow:", error);
      this.showError("Failed to initialize application");
      this.hideLoading();
    }
  }

  initManagers() {
    // Keyboard manager
    this.managers.keyboard = new KeyboardManager(this);
    this.managers.keyboard.bind();

    // Selection manager
    this.managers.selection = new SelectionManager(this);

    // Zoom manager
    this.managers.zoom = new ZoomManager(this);
  }

  initComponents() {
    // Main canvas
    this.components.canvas = new Canvas(this, {
      element: document.getElementById("canvas"),
      container: document.getElementById("canvas-area"),
    });

    // Header (top bar)
    this.components.header = {
      element: document.getElementById("header"),
    };

    // Tool palette (floating bar)
    this.components.toolPalette = new ToolPalette(this, {
      element: document.getElementById("floating-tools"),
    });

    // Component library (left panel)
    this.components.library = new ComponentLibrary(this, {
      element: document.getElementById("library-panel"),
    });

    // Properties panel (right)
    this.components.properties = new PropertiesPanel(this, {
      element: document.getElementById("properties"),
    });

    // Status bar (bottom)
    this.components.statusBar = new StatusBar(this, {
      element: document.getElementById("statusbar"),
    });

    // Context menu
    this.components.contextMenu = new ContextMenu(this, {
      element: document.getElementById("context-menu"),
    });
  }

  setupGlobalEvents() {
    // Panel toggles
    const toggleSidebar = document.getElementById("toggle-sidebar");
    const toggleProperties = document.getElementById("toggle-properties");
    const appEl = document.getElementById("app");

    toggleSidebar?.addEventListener("click", () => {
      appEl.classList.toggle("sidebar-collapsed");
      this.handleResize();
    });

    toggleProperties?.addEventListener("click", () => {
      appEl.classList.toggle("properties-collapsed");
      this.handleResize();
    });

    // Window resize
    window.addEventListener("resize", () => {
      this.handleResize();
    });

    // Global keyboard shortcuts
    document.addEventListener("keydown", (e) => {
      if (e.key === "Delete" || e.key === "Backspace") {
        this.deleteSelected();
      } else if (e.key === "a" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        this.selectAll();
      }
    });
  }

  handleResize() {
    if (this.components.canvas) {
      this.components.canvas.resize();
    }
  }


  startRenderLoop() {
    const render = () => {
      if (this.components.canvas) {
        this.components.canvas.render();
      }
      requestAnimationFrame(render);
    };
    requestAnimationFrame(render);
  }

  // Public API
  getEditor() {
    return this.editor;
  }

  getManager(name) {
    return this.managers[name];
  }

  getComponent(name) {
    return this.components[name];
  }

  showLoading(message = "Loading...") {
    // Remove existing loading overlay if any
    const existingOverlay = document.querySelector('.loading-overlay');
    if (existingOverlay) {
      existingOverlay.remove();
    }

    const overlay = document.createElement("div");
    overlay.className = "loading-overlay";
    overlay.innerHTML = `
      <div class="loading-content">
        <div class="loading-spinner"></div>
        <div class="loading-text">${message}</div>
      </div>
    `;
    document.body.appendChild(overlay);
  }

  hideLoading() {
    const overlay = document.querySelector('.loading-overlay');
    if (overlay) {
      overlay.style.opacity = '0';
      overlay.style.transition = 'opacity 0.3s ease';
      setTimeout(() => overlay.remove(), 300);
    }
  }

  async waitForElement(id, timeout = 5000) {
    const startTime = Date.now();
    while (Date.now() - startTime < timeout) {
      const element = document.getElementById(id);
      if (element) {
        return element;
      }
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    return null;
  }

  addShape(type, x, y, options = {}) {
    if (!this.editor) return null;

    let id = null;
    switch (type) {
      case "rect":
        id = this.editor.add_rect(
          x,
          y,
          options.width || 100,
          options.height || 80,
        );
        break;
      case "ellipse":
        id = this.editor.add_ellipse(
          x,
          y,
          options.radiusX || 50,
          options.radiusY || 40,
        );
        break;
      case "line":
        id = this.editor.add_line(
          x,
          y,
          x + options.width || 100,
          y + options.height || 0,
        );
        break;
    }

    if (id && id.length > 0) {
      this.managers.selection.select(id);
      this.components.properties.updateFromSelection();
      this.components.statusBar.updateShapeCount();
    }

    return id;
  }

  deleteSelected() {
    if (this.editor && this.managers.selection) {
      this.editor.delete_selected();
      this.managers.selection.clear();
      this.components.properties.updateFromSelection();
      this.components.statusBar.updateShapeCount();
    }
  }

  selectAll() {
    if (this.editor) {
      this.editor.select_all();
      this.managers.selection.selectAll();
      this.components.properties.updateFromSelection();
    }
  }

  showError(message, duration = 5000) {
    console.error(message);

    // Remove existing error toast if any
    const existingToast = document.querySelector('.error-toast');
    if (existingToast) {
      existingToast.remove();
    }

    const errorEl = document.createElement("div");
    errorEl.className = "error-toast";
    errorEl.textContent = message;
    errorEl.style.cssText = `
            position: fixed;
            top: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: #ef4444;
            color: white;
            padding: 12px 24px;
            border-radius: 8px;
            z-index: 10000;
            font-family: Inter, sans-serif;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        `;
    document.body.appendChild(errorEl);

    // Auto-remove after duration
    setTimeout(() => {
      if (errorEl.parentNode) {
        errorEl.style.opacity = '0';
        errorEl.style.transition = 'opacity 0.3s ease';
        setTimeout(() => errorEl.remove(), 300);
      }
    }, duration);
  }
}

// Initialize app when DOM is ready
document.addEventListener("DOMContentLoaded", () => {
  const app = new ArchFlowApp();
  app.init();

  // Export for debugging
  window.archFlow = app;
});

export { ArchFlowApp };
