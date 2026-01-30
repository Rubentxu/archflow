/**
 * Status Bar Component - Bottom status information
 */

import { Component } from './Component.js';

export class StatusBar extends Component {
    constructor(app, options) {
        super(app, options);
        this.shapeCount = 0;
        this.selectionCount = 0;
        this.position = { x: 0, y: 0 };
        this.zoom = 100;
    }

    mount() {
        return this;
    }

    setShapeCount(count) {
        this.shapeCount = count;
        const el = this.element?.querySelector('#status-shape-count');
        if (el) el.textContent = count;
    }

    setSelectionCount(count) {
        this.selectionCount = count;
        const el = this.element?.querySelector('#status-selected-count');
        if (el) el.textContent = count;
    }

    setPosition(x, y) {
        this.position = { x, y };
        const el = this.element?.querySelector('#status-position');
        if (el) el.textContent = `${Math.round(x)}, ${Math.round(y)}`;
    }

    setZoom(level) {
        this.zoom = level;
        const el = this.element?.querySelector('#status-zoom');
        if (el) el.textContent = `${Math.round(level * 100)}%`;

        const toolbarZoom = document.getElementById('zoom-level');
        if (toolbarZoom) toolbarZoom.textContent = `${Math.round(level * 100)}%`;
    }

    updateShapeCount() {
        const editor = this.app.getEditor();
        if (editor) {
            this.setShapeCount(editor.shape_count());
        }
    }

    updateSelectionCount() {
        const editor = this.app.getEditor();
        if (editor) {
            this.setSelectionCount(editor.selection_count());
        }
    }

    updateZoom() {
        const editor = this.app.getEditor();
        if (editor) {
            this.setZoom(editor.get_zoom());
        }
    }
}
