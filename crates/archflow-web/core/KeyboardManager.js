/**
 * Keyboard Manager - Handles keyboard shortcuts
 */

export class KeyboardManager {
    constructor(app) {
        this.app = app;
        this.shortcuts = new Map();
        this.isBound = false;
    }

    bind() {
        if (this.isBound) return;

        document.addEventListener('keydown', (e) => {
            this.handleKeydown(e);
        });

        document.addEventListener('keyup', (e) => {
            this.handleKeyup(e);
        });

        this.isBound = true;
        this.registerDefaultShortcuts();
    }

    registerDefaultShortcuts() {
        // Delete
        this.register('Delete', () => this.app.deleteSelected());
        this.register('Backspace', () => this.app.deleteSelected());

        // Select all
        this.register('a', (e) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                this.app.selectAll();
            }
        });

        // Copy
        this.register('c', (e) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                console.log('Copy - not implemented');
            }
        });

        // Paste
        this.register('v', (e) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                console.log('Paste - not implemented');
            }
        });

        // Undo
        this.register('z', (e) => {
            if (e.ctrlKey || e.metaKey) {
                e.preventDefault();
                if (e.shiftKey) {
                    console.log('Redo - not implemented');
                } else {
                    console.log('Undo - not implemented');
                }
            }
        });

        // Tool shortcuts
        this.register('v', () => this.selectTool('select'));
        this.register('r', () => this.selectTool('rect'));
        this.register('o', () => this.selectTool('ellipse'));
        this.register('l', () => this.selectTool('line'));
        this.register('t', () => this.selectTool('text'));
        this.register('h', () => this.selectTool('hand'));

        // Arrow keys for nudge
        this.register('ArrowUp', (e) => this.nudge(0, -1));
        this.register('ArrowDown', (e) => this.nudge(0, 1));
        this.register('ArrowLeft', (e) => this.nudge(-1, 0));
        this.register('ArrowRight', (e) => this.nudge(1, 0));

        // Precision nudge with Shift
        this.register('ArrowUp', (e) => {
            if (e.shiftKey) this.nudge(0, -0.1);
        });
        this.register('ArrowDown', (e) => {
            if (e.shiftKey) this.nudge(0, 0.1);
        });
        this.register('ArrowLeft', (e) => {
            if (e.shiftKey) this.nudge(-0.1, 0);
        });
        this.register('ArrowRight', (e) => {
            if (e.shiftKey) this.nudge(0.1, 0);
        });
    }

    register(key, handler) {
        this.shortcuts.set(key, handler);
    }

    handleKeydown(e) {
        const key = this.getKeyName(e);

        if (this.shortcuts.has(key)) {
            const handler = this.shortcuts.get(key);
            handler(e);
        }
    }

    handleKeyup(e) {
        // Handle key up events if needed
    }

    getKeyName(e) {
        const parts = [];

        if (e.ctrlKey || e.metaKey) parts.push('ctrl');
        if (e.shiftKey) parts.push('shift');
        if (e.altKey) parts.push('alt');

        parts.push(e.key);

        return parts.join('+');
    }

    selectTool(tool) {
        const toolPalette = this.app.getComponent('toolPalette');
        toolPalette?.selectTool(tool);
    }

    nudge(dx, dy) {
        const editor = this.app.getEditor();
        if (editor) {
            const amount = e.shiftKey ? 1 : 10;
            editor.move_selected(dx * amount, dy * amount);
        }
    }
}
