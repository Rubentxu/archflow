/**
 * Selection Manager - Handles shape selection
 */

export class SelectionManager {
    constructor(app) {
        this.app = app;
        this.selectedIds = new Set();
    }

    select(id) {
        this.clear();
        this.selectedIds.add(id);
        this.updateUI();
    }

    selectMultiple(ids) {
        this.clear();
        ids.forEach(id => this.selectedIds.add(id));
        this.updateUI();
    }

    selectAll() {
        const editor = this.app.getEditor();
        if (editor) {
            const shapes = editor.get_all_shapes();
            if (shapes) {
                shapes.forEach(shape => {
                    this.selectedIds.add(shape.id);
                });
            }
        }
        this.updateUI();
    }

    deselect(id) {
        this.selectedIds.delete(id);
        this.updateUI();
    }

    clear() {
        this.selectedIds.clear();
        const editor = this.app.getEditor();
        if (editor) {
            editor.clear_selection();
        }
        this.updateUI();
    }

    toggle(id) {
        if (this.selectedIds.has(id)) {
            this.deselect(id);
        } else {
            this.select(id);
        }
    }

    getSelected() {
        return Array.from(this.selectedIds);
    }

    getSelectedIds() {
        return this.selectedIds;
    }

    hasSelection() {
        return this.selectedIds.size > 0;
    }

    count() {
        return this.selectedIds.size;
    }

    updateUI() {
        // Update properties panel
        const properties = this.app.getComponent('properties');
        properties?.updateFromSelection();

        // Update status bar
        const statusBar = this.app.getComponent('statusBar');
        statusBar?.setSelectionCount(this.selectedIds.size);
    }
}
