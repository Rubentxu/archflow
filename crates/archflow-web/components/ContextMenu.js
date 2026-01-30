/**
 * Context Menu Component - Right-click menu
 */

import { Component } from './Component.js';

export class ContextMenu extends Component {
    constructor(app, options) {
        super(app, options);
        this.isVisible = false;
    }

    mount() {
        this.bindEvents();
        return this;
    }

    bindEvents() {
        // Context menu on canvas
        const canvas = this.app.getComponent('canvas');
        canvas?.element?.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            this.show(e.clientX, e.clientY);
        });

        // Menu item clicks
        this.element?.querySelectorAll('.menu-item[data-action]').forEach(item => {
            item.addEventListener('click', () => {
                const action = item.dataset.action;
                this.executeAction(action);
                this.hide();
            });
        });

        // Hide on click outside
        document.addEventListener('click', (e) => {
            if (this.isVisible && !this.element?.contains(e.target)) {
                this.hide();
            }
        });

        // Hide on escape
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.isVisible) {
                this.hide();
            }
        });
    }

    show(x, y) {
        // Position menu
        this.element.style.left = `${x}px`;
        this.element.style.top = `${y}px`;
        this.element.classList.remove('hidden');
        this.element.setAttribute('aria-hidden', 'false');
        this.isVisible = true;

        // Update menu state based on selection
        this.updateMenuState();
    }

    hide() {
        this.element?.classList.add('hidden');
        this.element?.setAttribute('aria-hidden', 'true');
        this.isVisible = false;
    }

    updateMenuState() {
        const selection = this.app.getEditor()?.get_selection();
        const hasSelection = selection?.shapeIds?.length > 0;

        // Enable/disable menu items based on selection
        this.element?.querySelectorAll('.menu-item[data-action]').forEach(item => {
            const action = item.dataset.action;
            const requiresSelection = ['copy', 'cut', 'duplicate', 'delete', 'bring-forward', 'send-backward'].includes(action);

            if (requiresSelection && !hasSelection) {
                item.classList.add('disabled');
            } else {
                item.classList.remove('disabled');
            }
        });
    }

    executeAction(action) {
        const editor = this.app.getEditor();

        switch (action) {
            case 'copy':
                console.log('Copy - not implemented');
                break;
            case 'cut':
                console.log('Cut - not implemented');
                break;
            case 'paste':
                console.log('Paste - not implemented');
                break;
            case 'duplicate':
                this.duplicateSelected();
                break;
            case 'delete':
                this.app.deleteSelected();
                break;
            case 'bring-forward':
                console.log('Bring forward - not implemented');
                break;
            case 'send-backward':
                console.log('Send backward - not implemented');
                break;
        }
    }

    duplicateSelected() {
        const selection = this.app.getEditor()?.get_selection();
        if (!selection?.shapeIds?.length) return;

        const shapeId = selection.shapeIds[0];
        const shape = this.app.getEditor()?.get_shape(shapeId);

        if (shape) {
            const offset = 20;
            this.app.addShape('rect', shape.x + offset, shape.y + offset, {
                width: shape.width,
                height: shape.height
            });
        }
    }
}
