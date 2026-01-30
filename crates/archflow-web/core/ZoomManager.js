/**
 * Zoom Manager - Handles canvas zoom
 */

export class ZoomManager {
    constructor(app) {
        this.app = app;
        this.level = 1.0;
        this.minLevel = 0.1;
        this.maxLevel = 5.0;
    }

    zoomIn() {
        this.setLevel(this.level * 1.2);
    }

    zoomOut() {
        this.setLevel(this.level / 1.2);
    }

    setLevel(newLevel) {
        this.level = Math.max(this.minLevel, Math.min(this.maxLevel, newLevel));
        this.applyZoom();
    }

    reset() {
        this.setLevel(1.0);
    }

    applyZoom() {
        const editor = this.app.getEditor();
        if (editor) {
            editor.set_zoom(this.level);
        }

        // Update UI
        const toolbar = this.app.getComponent('toolbar');
        toolbar?.setZoom(this.level);

        const statusBar = this.app.getComponent('statusBar');
        statusBar?.setZoom(this.level);
    }

    getLevel() {
        return this.level;
    }

    zoomAt(x, y, factor) {
        const newLevel = this.level * factor;
        this.setLevel(newLevel);
    }
}
