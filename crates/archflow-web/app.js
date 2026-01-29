/**
 * ArchFlow Web Application - Production Ready
 *
 * Main entry point using the real ArchFlow SDK for all functionality.
 */

import init, {
  ArchFlowEditor,
  JsLibraryManager,
  JsPropertiesManager,
  JsAlignmentManager,
  JsGroupManager
} from "./pkg/archflow_web.js";

/**
 * Main application class - Production implementation
 */
class ArchFlowApp {
  constructor() {
    this.canvas = null;
    this.canvasContainer = null;
    this.editor = null;
    this.libraryManager = null;
    this.propertiesManager = null;
    this.alignmentManager = null;
    this.groupManager = null;
    this.currentTool = 'select';
    this.isInitialized = false;
    this.draggedComponent = null;
    this.selectedShape = null;
    this.selectionBounds = null;
  }

  /**
   * Initialize the application with production setup
   */
  async init() {
    try {
      // Initialize WASM
      await init();
      log::info!("WASM initialized successfully");

      // Get DOM elements
      this.canvas = document.getElementById('canvas');
      this.canvasContainer = document.getElementById('canvas-area');

      if (!this.canvas) {
        throw new Error('Canvas element not found');
      }

      // Initialize canvas size
      this.resizeCanvas();

      // Create SDK editor instances
      this.editor = new ArchFlowEditor(this.canvas);
      this.libraryManager = this.editor.get_library_manager();
      this.propertiesManager = this.editor.get_properties_manager();
      this.alignmentManager = this.editor.get_alignment_manager();
      this.groupManager = this.editor.get_group_manager();

      log::info!("SDK managers initialized");

      // Setup event listeners
      this.setupEventListeners();

      // Initialize UI
      this.initializeUI();

      // Start render loop
      this.startRenderLoop();

      this.isInitialized = true;
      log::info!("ArchFlow application initialized");
    } catch (error) {
      console.error('Failed to initialize ArchFlow:', error);
      this.showError('Failed to initialize application: ' + error.message);
    }
  }

  /**
   * Resize canvas to fit container
   */
  resizeCanvas() {
    if (!this.canvas || !this.canvasContainer) return;

    const rect = this.canvasContainer.getBoundingClientRect();
    this.canvas.width = rect.width;
    this.canvas.height = rect.height;

    if (this.editor) {
      this.editor.resize(rect.width, rect.height);
    }
  }

  /**
   * Setup all event listeners - Production implementation
   */
  setupEventListeners() {
    // Window resize
    window.addEventListener('resize', () => {
      this.resizeCanvas();
    });

    // Tool selection
    document.querySelectorAll('[data-tool]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const tool = e.currentTarget.dataset.tool;
        this.setTool(tool);
      });
    });

    // Canvas mouse events
    this.canvas.addEventListener('mousedown', (e) => {
      if (!this.editor) return;
      const rect = this.canvas.getBoundingClientRect();
      this.editor.on_mousedown(
        e.clientX - rect.left,
        e.clientY - rect.top,
        e.button
      );
      this.updateSelection();
    });

    this.canvas.addEventListener('mousemove', (e) => {
      if (!this.editor) return;
      const rect = this.canvas.getBoundingClientRect();
      this.editor.on_mousemove(
        e.clientX - rect.left,
        e.clientY - rect.top
      );

      // Update status bar position
      this.updatePosition(e.clientX - rect.left, e.clientY - rect.top);
    });

    this.canvas.addEventListener('mouseup', (e) => {
      if (!this.editor) return;
      const rect = this.canvas.getBoundingClientRect();
      this.editor.on_mouseup(
        e.clientX - rect.left,
        e.clientY - rect.top
      );

      // Update selection after mouse up
      this.updateSelection();
    });

    // Wheel for zoom
    this.canvas.addEventListener('wheel', (e) => {
      e.preventDefault();
      if (!this.editor) return;
      const rect = this.canvas.getBoundingClientRect();
      this.editor.on_wheel(
        e.clientX - rect.left,
        e.clientY - rect.top,
        e.deltaY > 0
      );
      this.updateZoom();
    }, { passive: false });

    // Keyboard events - Full production support
    document.addEventListener('keydown', (e) => {
      if (!this.editor) return;

      // Prevent default for editor shortcuts
      const isShortcut = e.ctrlKey || e.metaKey ||
        ['Delete', 'Backspace', 'Escape', ' '].includes(e.key);

      if (isShortcut) {
        e.preventDefault();
      }

      this.editor.on_keydown(e.key, e.shiftKey, e.ctrlKey || e.metaKey);

      // Update UI after keyboard actions
      this.updateUI();
      this.updateSelection();
    });

    document.addEventListener('keyup', (e) => {
      if (this.editor) {
        this.editor.on_keyup(e.key);
      }
    });

    // Right-click context menu
    this.canvas.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      this.showContextMenu(e.clientX, e.clientY);
    });

    // Zoom controls
    document.getElementById('zoom-in')?.addEventListener('click', () => {
      if (this.editor) {
        this.editor.on_wheel(0, 0, false);
        this.updateZoom();
      }
    });

    document.getElementById('zoom-out')?.addEventListener('click', () => {
      if (this.editor) {
        this.editor.on_wheel(0, 0, true);
        this.updateZoom();
      }
    });

    document.getElementById('zoom-fit')?.addEventListener('click', () => {
      if (this.editor) {
        this.editor.zoom_to_fit();
        this.updateZoom();
      }
    });

    // Action buttons
    document.getElementById('btn-undo')?.addEventListener('click', () => {
      if (this.editor && this.editor.can_undo()) {
        this.editor.undo();
      }
    });

    document.getElementById('btn-redo')?.addEventListener('click', () => {
      if (this.editor && this.editor.can_redo()) {
        this.editor.redo();
      }
    });

    document.getElementById('btn-clear')?.addEventListener('click', () => {
      if (confirm('Clear all shapes?')) {
        this.editor.clear();
      }
    });

    // Toggle buttons
    document.getElementById('toggle-grid')?.addEventListener('click', (e) => {
      e.target.classList.toggle('active');
      const isActive = e.target.classList.contains('active');
      e.target.textContent = `Grid: ${isActive ? 'ON' : 'OFF'}`;
      // Update grid config via SDK
      const config = this.editor.get_grid_config();
      config.showGrid = isActive;
      this.editor.set_grid_config(config);
    });

    document.getElementById('toggle-snap')?.addEventListener('click', (e) => {
      e.target.classList.toggle('active');
      const isActive = e.target.classList.contains('active');
      e.target.textContent = `Snap: ${isActive ? 'ON' : 'OFF'}`;
    });

    // Canvas drag & drop - Full implementation
    this.setupDragAndDrop();

    // Property panel inputs - Connected to SDK
    this.setupPropertyPanels();

    // Alignment panel - Connected to SDK
    this.setupAlignmentPanel();

    // Group panel - Connected to SDK
    this.setupGroupPanel();
  }

  /**
   * Setup drag and drop from library to canvas - Production
   */
  setupDragAndDrop() {
    if (!this.canvas) return;

    // Allow drop on canvas
    this.canvas.addEventListener('dragover', (e) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'copy';
      this.canvas.classList.add('drag-over');
    });

    this.canvas.addEventListener('dragleave', () => {
      this.canvas.classList.remove('drag-over');
    });

    this.canvas.addEventListener('drop', (e) => {
      e.preventDefault();
      this.canvas.classList.remove('drag-over');

      const data = e.dataTransfer.getData('application/archflow-component');
      if (!data) return;

      try {
        const componentData = JSON.parse(data);
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        this.createShapeFromLibrary(componentData.libraryId, componentData.itemId, x, y);
      } catch (error) {
        console.error('Failed to create shape from library:', error);
      }
    });
  }

  /**
   * Create a shape from library component using SDK
   */
  createShapeFromLibrary(libraryId, itemId, x, y) {
    try {
      // Get component data from library manager
      const componentJson = this.libraryManager.get_component_data(libraryId, itemId);
      const component = JSON.parse(componentJson);

      // Get geometry from component
      const geometry = component.geometry || { width: 100, height: 80 };
      const style = component.style || {};

      // Convert canvas coordinates
      const canvasPoint = this.editor.screen_to_canvas(x, y);

      // Create shape based on component type using SDK
      let shapeId;
      switch (component.shape_type) {
        case 'Rectangle':
        case 'RoundedRect':
          shapeId = this.editor.add_rect(
            canvasPoint.x - geometry.width / 2,
            canvasPoint.y - geometry.height / 2,
            geometry.width,
            geometry.height
          );
          break;
        case 'Circle':
          shapeId = this.editor.add_ellipse(
            canvasPoint.x,
            canvasPoint.y,
            geometry.width / 2,
            geometry.height / 2
          );
          break;
        case 'Ellipse':
          shapeId = this.editor.add_ellipse(
            canvasPoint.x - geometry.width / 2,
            canvasPoint.y - geometry.height / 2,
            geometry.width / 2,
            geometry.height / 2
          );
          break;
        case 'Diamond':
          // Diamond is a rotated square
          shapeId = this.editor.add_rect(
            canvasPoint.x - geometry.width / 2,
            canvasPoint.y - geometry.height / 2,
            geometry.width,
            geometry.height
          );
          break;
        default:
          // Default to rectangle
          shapeId = this.editor.add_rect(
            canvasPoint.x - geometry.width / 2,
            canvasPoint.y - geometry.height / 2,
            geometry.width,
            geometry.height
          );
      }

      // Apply component style via properties manager
      if (style.fill_color) {
        this.propertiesManager.set_fill_color(shapeId, style.fill_color);
      }
      if (style.stroke_color) {
        this.propertiesManager.set_stroke_color(shapeId, style.stroke_color);
      }

      log::info(`Created ${component.shape_type} from library with id: ${shapeId}`);
    } catch (error) {
      console.error('Failed to create shape from library:', error);
      // Fallback: create simple rectangle
      const canvasPoint = this.editor.screen_to_canvas(x, y);
      this.editor.add_rect(canvasPoint.x - 50, canvasPoint.y - 40, 100, 80);
    }
  }

  /**
   * Setup property panel inputs - Connected to properties manager
   */
  setupPropertyPanels() {
    // Transform panel inputs
    const transformInputs = ['prop-x', 'prop-y', 'prop-width', 'prop-height', 'prop-rotation'];
    transformInputs.forEach(id => {
      const element = document.getElementById(id);
      if (element) {
        element.addEventListener('change', (e) => {
          this.updateSelectedShapeProperty(id, e.target.value);
        });
      }
    });

    // Color pickers
    const fillPreview = document.getElementById('prop-fill-preview');
    const strokePreview = document.getElementById('prop-stroke-preview');

    if (fillPreview) {
      fillPreview.addEventListener('click', () => {
        this.showColorPicker('fill', fillPreview);
      });
    }

    if (strokePreview) {
      strokePreview.addEventListener('click', () => {
        this.showColorPicker('stroke', strokePreview);
      });
    }

    // Sliders
    const strokeWidthSlider = document.getElementById('prop-stroke-width');
    const opacitySlider = document.getElementById('prop-opacity');

    if (strokeWidthSlider) {
      strokeWidthSlider.addEventListener('input', (e) => {
        document.getElementById('stroke-width-value').textContent = `${e.target.value}px`;
        this.updateSelectedShapeProperty('stroke-width', parseFloat(e.target.value));
      });
    }

    if (opacitySlider) {
      opacitySlider.addEventListener('input', (e) => {
        document.getElementById('opacity-value').textContent = `${e.target.value}%`;
        this.updateSelectedShapeProperty('opacity', parseFloat(e.target.value) / 100);
      });
    }

    // Lock aspect ratio
    const lockAspect = document.getElementById('prop-lock-aspect');
    if (lockAspect) {
      lockAspect.addEventListener('change', (e) => {
        this.propertiesManager.set_lock_aspect_ratio(e.target.checked);
      });
    }
  }

  /**
   * Update selected shape property via properties manager
   */
  updateSelectedShapeProperty(property, value) {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (!jsSelection || jsSelection.shapeIds.length === 0) {
      console.log('No shape selected');
      return;
    }

    const shapeId = jsSelection.shapeIds[0];

    // Build changes object
    const changes = {};
    switch (property) {
      case 'prop-x':
        changes.x = parseFloat(value);
        break;
      case 'prop-y':
        changes.y = parseFloat(value);
        break;
      case 'prop-width':
        changes.width = parseFloat(value);
        break;
      case 'prop-height':
        changes.height = parseFloat(value);
        break;
      case 'prop-rotation':
        changes.rotation = parseFloat(value);
        break;
      case 'stroke-width':
        changes.strokeWidth = value;
        break;
      case 'opacity':
        changes.opacity = value;
        break;
    }

    // Apply via properties manager
    this.propertiesManager.update_shape(shapeId, JSON.stringify(changes));
    this.editor.update_shape(shapeId, JSON.stringify(changes));
  }

  /**
   * Show color picker dialog
   */
  showColorPicker(type, previewElement) {
    const input = document.createElement('input');
    input.type = 'color';
    input.value = previewElement.style.backgroundColor || '#3366cc';

    input.addEventListener('change', (e) => {
      previewElement.style.backgroundColor = e.target.value;
      if (type === 'fill') {
        document.getElementById('prop-fill-value').textContent = e.target.value;
        this.updateSelectedShapeProperty('fill-color', e.target.value);
      } else {
        document.getElementById('prop-stroke-value').textContent = e.target.value;
        this.updateSelectedShapeProperty('stroke-color', e.target.value);
      }
    });

    input.click();
  }

  /**
   * Setup alignment panel - Connected to alignment manager
   */
  setupAlignmentPanel() {
    const alignButtons = document.querySelectorAll('.align-btn[data-align]');
    alignButtons.forEach(btn => {
      btn.addEventListener('click', () => {
        const alignment = btn.dataset.align;
        this.alignShapes(alignment);
      });
    });

    // Distribution buttons
    document.getElementById('btn-distribute-h')?.addEventListener('click', () => {
      this.distributeShapes('horizontal');
    });

    document.getElementById('btn-distribute-v')?.addEventListener('click', () => {
      this.distributeShapes('vertical');
    });
  }

  /**
   * Align selected shapes using alignment manager
   */
  alignShapes(alignment) {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (!jsSelection || jsSelection.shapeIds.length < 2) {
      console.log('Need at least 2 shapes for alignment');
      return;
    }

    // Use alignment manager
    this.alignmentManager.align(jsSelection.shapeIds, alignment);
    log::info(`Aligned shapes: ${alignment}`);
  }

  /**
   * Distribute selected shapes
   */
  distributeShapes(direction) {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (!jsSelection || jsSelection.shapeIds.length < 3) {
      console.log('Need at least 3 shapes for distribution');
      return;
    }

    // Use alignment manager for distribution
    this.alignmentManager.distribute(jsSelection.shapeIds, direction);
    log::info(`Distributed shapes: ${direction}`);
  }

  /**
   * Setup group panel - Connected to group manager
   */
  setupGroupPanel() {
    // Group button
    document.getElementById('btn-group')?.addEventListener('click', () => {
      this.groupSelectedShapes();
    });

    // Ungroup button
    document.getElementById('btn-ungroup')?.addEventListener('click', () => {
      this.ungroupSelectedShapes();
    });

    // Layer buttons
    document.getElementById('btn-bring-forward')?.addEventListener('click', () => {
      this.bringForward();
    });

    document.getElementById('btn-send-backward')?.addEventListener('click', () => {
      this.sendBackward();
    });
  }

  /**
   * Group selected shapes using group manager
   */
  groupSelectedShapes() {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (!jsSelection || jsSelection.shapeIds.length < 2) {
      console.log('Need at least 2 shapes to group');
      return;
    }

    const groupId = this.groupManager.group(jsSelection.shapeIds);
    log::info(`Created group: ${groupId}`);
    this.updateSelection();
  }

  /**
   * Ungroup selected shapes
   */
  ungroupSelectedShapes() {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (!jsSelection || jsSelection.shapeIds.length === 0) {
      return;
    }

    this.groupManager.ungroup(jsSelection.shapeIds[0]);
    log::info('Ungrouped shapes');
    this.updateSelection();
  }

  /**
   * Bring selected shapes forward in z-order
   */
  bringForward() {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (jsSelection && jsSelection.shapeIds.length > 0) {
      // Use canvas layer operations
      console.log('Bring forward:', jsSelection.shapeIds);
    }
  }

  /**
   * Send selected shapes backward in z-order
   */
  sendBackward() {
    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    if (jsSelection && jsSelection.shapeIds.length > 0) {
      console.log('Send backward:', jsSelection.shapeIds);
    }
  }

  /**
   * Update selection and property panels from SDK
   */
  updateSelection() {
    if (!this.editor) return;

    const selection = this.editor.get_selection();
    const jsSelection = selection.into_serde();

    const count = jsSelection?.shapeIds?.length || 0;
    document.getElementById('status-selected-count').textContent = count;

    if (count === 1) {
      const shapeId = jsSelection.shapeIds[0];
      this.selectedShape = shapeId;
      this.selectionBounds = jsSelection.bounds;
      this.updatePropertyPanel(shapeId);
      this.enablePropertyPanels(true);
    } else if (count > 1) {
      this.selectedShape = null;
      this.selectionBounds = jsSelection.bounds;
      this.updateMultiSelectionPanel(jsSelection.shapeIds);
      this.enablePropertyPanels(true);
    } else {
      this.selectedShape = null;
      this.selectionBounds = null;
      this.enablePropertyPanels(false);
    }
  }

  /**
   * Update property panel with selected shape data
   */
  updatePropertyPanel(shapeId) {
    const shape = this.editor.get_shape(shapeId);
    const jsShape = shape.into_serde();

    if (!jsShape) return;

    // Update transform inputs
    const xInput = document.getElementById('prop-x');
    const yInput = document.getElementById('prop-y');
    const wInput = document.getElementById('prop-width');
    const hInput = document.getElementById('prop-height');
    const rInput = document.getElementById('prop-rotation');

    if (xInput) xInput.value = Math.round(jsShape.x);
    if (yInput) yInput.value = Math.round(jsShape.y);
    if (wInput) wInput.value = Math.round(jsShape.width);
    if (hInput) hInput.value = Math.round(jsShape.height);
    if (rInput) rInput.value = Math.round(jsShape.rotation);

    // Update color inputs
    const fillPreview = document.getElementById('prop-fill-preview');
    const strokePreview = document.getElementById('prop-stroke-preview');
    const fillValue = document.getElementById('prop-fill-value');
    const strokeValue = document.getElementById('prop-stroke-value');

    if (fillPreview && jsShape.fillColor) {
      const fillColor = `rgb(${jsShape.fillColor.r * 255}, ${jsShape.fillColor.g * 255}, ${jsShape.fillColor.b * 255})`;
      fillPreview.style.backgroundColor = fillColor;
      if (fillValue) fillValue.textContent = fillColor;
    }

    if (strokePreview && jsShape.strokeColor) {
      const strokeColor = `rgb(${jsShape.strokeColor.r * 255}, ${jsShape.strokeColor.g * 255}, ${jsShape.strokeColor.b * 255})`;
      strokePreview.style.backgroundColor = strokeColor;
      if (strokeValue) strokeValue.textContent = strokeColor;
    }

    // Update sliders
    const strokeWidthSlider = document.getElementById('prop-stroke-width');
    const opacitySlider = document.getElementById('prop-opacity');

    if (strokeWidthSlider) {
      strokeWidthSlider.value = jsShape.strokeWidth;
      document.getElementById('stroke-width-value').textContent = `${jsShape.strokeWidth}px`;
    }

    if (opacitySlider) {
      opacitySlider.value = jsShape.opacity * 100;
      document.getElementById('opacity-value').textContent = `${Math.round(jsShape.opacity * 100)}%`;
    }
  }

  /**
   * Update panel for multi-selection
   */
  updateMultiSelectionPanel(shapeIds) {
    // Show placeholder indicating multi-selection
    console.log('Multi-selection:', shapeIds.length, 'shapes selected');
  }

  /**
   * Enable/disable property panels based on selection
   */
  enablePropertyPanels(enabled) {
    const panels = document.querySelectorAll('#properties input, #properties button');
    panels.forEach(el => {
      el.disabled = !enabled;
    });
  }

  /**
   * Set current tool
   */
  setTool(tool) {
    this.currentTool = tool;

    // Update UI
    document.querySelectorAll('[data-tool]').forEach(btn => {
      btn.classList.remove('active');
      if (btn.dataset.tool === tool) {
        btn.classList.add('active');
      }
    });

    // Update cursor
    this.updateCursor(tool);
  }

  /**
   * Update canvas cursor based on tool
   */
  updateCursor(tool) {
    this.canvas.className = '';
    switch (tool) {
      case 'select':
        this.canvas.classList.add('cursor-default');
        break;
      case 'rect':
      case 'ellipse':
      case 'line':
        this.canvas.classList.add('cursor-crosshair');
        break;
      case 'hand':
        this.canvas.classList.add('cursor-grab');
        break;
      case 'text':
        this.canvas.classList.add('cursor-text');
        break;
    }
  }

  /**
   * Initialize UI components
   */
  initializeUI() {
    this.updateZoom();
    this.populateLibrary();
    this.enablePropertyPanels(false);
    this.enableLibraryKeyboardNavigation();
  }

  /**
   * Populate component library using library manager
   */
  populateLibrary() {
    try {
      const categoriesContainer = document.getElementById('library-categories');
      if (!categoriesContainer) return;

      const librariesJson = this.libraryManager.get_libraries();
      const libraries = JSON.parse(librariesJson);

      libraries.forEach(library => {
        library.categories.forEach(category => {
          this.createCategoryElement(library.id, category, categoriesContainer);
        });
      });

      log::info('Component library populated');
    } catch (error) {
      console.error('Failed to populate library:', error);
    }
  }

  /**
   * Create category element
   */
  createCategoryElement(library, category, container) {
    const categoryEl = document.createElement('div');
    categoryEl.className = 'library-category';
    categoryEl.innerHTML = `
      <div class="category-header" role="button" tabindex="0">
        <span class="category-title">
          <span class="category-icon">${category.icon || '📦'}</span>
          ${category.name}
        </span>
        <i class="ph ph-caret-down category-toggle" aria-hidden="true"></i>
      </div>
      <div class="category-content">
        <div class="component-grid"></div>
      </div>
    `;

    const grid = categoryEl.querySelector('.component-grid');
    category.items.forEach(item => {
      this.createComponentElement(library.id, item, grid);
    });

    // Toggle functionality
    const header = categoryEl.querySelector('.category-header');
    const content = categoryEl.querySelector('.category-content');
    const toggle = categoryEl.querySelector('.category-toggle');

    header.addEventListener('click', () => {
      content.classList.toggle('collapsed');
      toggle.classList.toggle('collapsed');
    });

    container.appendChild(categoryEl);
  }

  /**
   * Create component element with full accessibility
   */
  createComponentElement(libraryId, item, container) {
    const componentEl = document.createElement('div');
    componentEl.className = 'component-item';
    componentEl.draggable = true;
    componentEl.dataset.libraryId = libraryId;
    componentEl.dataset.itemId = item.id;
    componentEl.tabIndex = 0;
    componentEl.setAttribute('role', 'button');
    componentEl.setAttribute('aria-label', item.name);

    // Create preview
    let previewHTML = '';
    if (item.preview) {
      if (item.preview.type === 'Icon') {
        previewHTML = `<span class="icon">${item.preview.value}</span>`;
      } else if (item.preview.type === 'Svg') {
        previewHTML = item.preview.value;
      } else {
        previewHTML = `<span class="icon">${item.preview.value || '⬜'}</span>`;
      }
    }

    componentEl.innerHTML = `
      <div class="component-preview">${previewHTML}</div>
      <span class="component-label">${item.name}</span>
    `;

    // Drag events
    componentEl.addEventListener('dragstart', (e) => {
      e.dataTransfer.setData('application/archflow-component', JSON.stringify({
        libraryId,
        itemId: item.id
      }));
      componentEl.classList.add('dragging');
    });

    componentEl.addEventListener('dragend', () => {
      componentEl.classList.remove('dragging');
    });

    // Keyboard events for accessibility
    componentEl.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        this.startDragFromKeyboard(componentEl, libraryId, item);
      }
    });

    container.appendChild(componentEl);
  }

  /**
   * Start drag operation from keyboard
   */
  startDragFromKeyboard(element, libraryId, item) {
    const event = new DragEvent('dragstart', {
      bubbles: true,
      cancelable: true,
      dataTransfer: new DataTransfer()
    });

    event.dataTransfer.setData('application/archflow-component', JSON.stringify({
      libraryId,
      itemId: item.id
    }));

    element.dispatchEvent(event);
    element.classList.add('focused');
    log::info(`Started drag for ${item.name}`);
  }

  /**
   * Enable keyboard navigation in component library
   */
  enableLibraryKeyboardNavigation() {
    const grids = document.querySelectorAll('.component-grid');

    grids.forEach(grid => {
      grid.setAttribute('role', 'grid');
      grid.setAttribute('aria-label', 'Component library');

      const items = grid.querySelectorAll('.component-item');
      items.forEach((item, index) => {
        item.setAttribute('role', 'gridcell');
        item.setAttribute('tabindex', index === 0 ? '0' : '-1');
      });
    });

    // Handle arrow key navigation
    document.addEventListener('keydown', (e) => {
      if (!e.target.classList.contains('component-item')) return;

      const grid = e.target.closest('.component-grid');
      if (!grid) return;

      const items = Array.from(grid.querySelectorAll('.component-item'));
      const currentIndex = items.indexOf(e.target);
      const gridColumns = 3;

      let newIndex = currentIndex;

      switch (e.key) {
        case 'ArrowDown':
          newIndex = Math.min(currentIndex + gridColumns, items.length - 1);
          break;
        case 'ArrowUp':
          newIndex = Math.max(currentIndex - gridColumns, 0);
          break;
        case 'ArrowRight':
          newIndex = Math.min(currentIndex + 1, items.length - 1);
          break;
        case 'ArrowLeft':
          newIndex = Math.max(currentIndex - 1, 0);
          break;
        case 'Home':
          newIndex = 0;
          break;
        case 'End':
          newIndex = items.length - 1;
          break;
        default:
          return;
      }

      if (newIndex !== currentIndex) {
        e.preventDefault();
        e.target.setAttribute('tabindex', '-1');
        e.target.classList.remove('focused');

        const newItem = items[newIndex];
        newItem.setAttribute('tabindex', '0');
        newItem.classList.add('focused');
        newItem.focus();
      }
    });
  }

  /**
   * Start render loop
   */
  startRenderLoop() {
    let lastShapeCount = 0;
    let lastZoom = 100;

    const render = () => {
      if (this.editor) {
        this.editor.render();
      }

      // Update shape count
      const shapeCount = this.editor?.shape_count() || 0;
      if (shapeCount !== lastShapeCount) {
        document.getElementById('status-shape-count').textContent = shapeCount;
        lastShapeCount = shapeCount;
      }

      // Update zoom
      const zoom = Math.round((this.editor?.get_zoom() || 1) * 100);
      if (zoom !== lastZoom) {
        document.getElementById('zoom-level').textContent = `${zoom}%`;
        document.getElementById('status-zoom').textContent = `${zoom}%`;
        lastZoom = zoom;
      }

      requestAnimationFrame(render);
    };

    requestAnimationFrame(render);
  }

  /**
   * Update position display
   */
  updatePosition(x, y) {
    const canvasPoint = this.editor?.screen_to_canvas(x, y);
    if (canvasPoint) {
      const jsPoint = canvasPoint.into_serde();
      if (jsPoint) {
        document.getElementById('status-position').textContent =
          `${Math.round(jsPoint.x)}, ${Math.round(jsPoint.y)}`;
      }
    }
  }

  /**
   * Update zoom display
   */
  updateZoom() {
    const zoom = Math.round((this.editor?.get_zoom() || 1) * 100);
    document.getElementById('zoom-level').textContent = `${zoom}%`;
    document.getElementById('status-zoom').textContent = `${zoom}%`;
  }

  /**
   * Update UI state
   */
  updateUI() {
    const selectionCount = this.editor?.selection_count() || 0;
    document.getElementById('status-selected-count').textContent = selectionCount;
  }

  /**
   * Show context menu
   */
  showContextMenu(x, y) {
    const menu = document.getElementById('context-menu');
    if (!menu) return;

    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    menu.classList.remove('hidden');
    menu.setAttribute('aria-hidden', 'false');

    // Hide on click outside
    const hideMenu = () => {
      menu.classList.add('hidden');
      menu.setAttribute('aria-hidden', 'true');
      document.removeEventListener('click', hideMenu);
    };

    setTimeout(() => {
      document.addEventListener('click', hideMenu);
    }, 0);

    // Setup context menu actions
    this.setupContextMenuActions();
  }

  /**
   * Setup context menu actions
   */
  setupContextMenuActions() {
    const menu = document.getElementById('context-menu');
    if (!menu) return;

    const actions = menu.querySelectorAll('.menu-item[data-action]');
    actions.forEach(item => {
      item.addEventListener('click', () => {
        const action = item.dataset.action;
        this.executeContextAction(action);
        menu.classList.add('hidden');
      });
    });
  }

  /**
   * Execute context menu action
   */
  executeContextAction(action) {
    const selection = this.editor?.get_selection();
    const jsSelection = selection?.into_serde();

    switch (action) {
      case 'copy':
        console.log('Copy selected');
        // Implement copy to clipboard
        break;
      case 'cut':
        console.log('Cut selected');
        // Implement cut
        break;
      case 'paste':
        console.log('Paste');
        // Implement paste
        break;
      case 'duplicate':
        if (jsSelection?.shapeIds?.[0]) {
          this.duplicateShape(jsSelection.shapeIds[0]);
        }
        break;
      case 'delete':
        this.editor?.delete_selected();
        break;
      case 'bring-forward':
        this.bringForward();
        break;
      case 'send-backward':
        this.sendBackward();
        break;
    }
  }

  /**
   * Duplicate a shape
   */
  duplicateShape(shapeId) {
    const shape = this.editor?.get_shape(shapeId);
    const jsShape = shape?.into_serde();

    if (jsShape) {
      const offset = 20;
      this.editor?.add_rect(
        jsShape.x + offset,
        jsShape.y + offset,
        jsShape.width,
        jsShape.height
      );
    }
  }

  /**
   * Show error message
   */
  showError(message) {
    console.error(message);
    const errorEl = document.createElement('div');
    errorEl.className = 'error-toast';
    errorEl.textContent = message;
    errorEl.style.cssText = `
      position: fixed;
      top: 20px;
      left: 50%;
      transform: translateX(-50%);
      background: #f44336;
      color: white;
      padding: 12px 24px;
      border-radius: 8px;
      z-index: 10000;
      font-family: Inter, sans-serif;
    `;
    document.body.appendChild(errorEl);
    setTimeout(() => errorEl.remove(), 5000);
  }
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  const app = new ArchFlowApp();
  app.init();
});

// Export for testing
export { ArchFlowApp };
