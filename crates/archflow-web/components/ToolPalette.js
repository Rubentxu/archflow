/**
 * Tool Palette Component - Left sidebar with tools
 */

import { Component } from './Component.js';

export class ToolPalette extends Component {
    constructor(app, options) {
        super(app, options);
        this.currentTool = 'select';
        this.tools = ['select', 'rect', 'ellipse', 'line', 'text', 'pencil', 'hand'];
    }

    mount() {
        this.bindEvents();
        this.selectTool('select');
        return this;
    }

    bindEvents() {
        this.element?.querySelectorAll('[data-tool]').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const tool = e.currentTarget.dataset.tool;
                this.selectTool(tool);
            });
        });

        // Bind simulate button
        const simulateBtn = document.getElementById('btn-simulate');
        simulateBtn?.addEventListener('click', () => {
            console.log("Simulate clicked");
            // Add simulation logic here
        });
    }

    selectTool(tool) {
        if (!this.tools.includes(tool)) return;

        this.currentTool = tool;

        // Update UI
        this.element?.querySelectorAll('[data-tool]').forEach(btn => {
            btn.classList.remove('active');
            if (btn.dataset.tool === tool) {
                btn.classList.add('active');
            }
        });

        // Update cursor
        this.updateCursor();

        console.log(`Tool selected: ${tool}`);
    }

    updateCursor() {
        const canvas = this.app.getComponent('canvas');
        if (!canvas?.element) return;

        canvas.element.className = '';

        switch (this.currentTool) {
            case 'rect':
            case 'ellipse':
            case 'line':
                canvas.element.classList.add('cursor-crosshair');
                break;
            case 'hand':
                canvas.element.classList.add('cursor-grab');
                break;
            case 'text':
                canvas.element.classList.add('cursor-text');
                break;
            default:
                canvas.element.classList.add('cursor-default');
        }
    }

    getCurrentTool() {
        return this.currentTool;
    }
}
