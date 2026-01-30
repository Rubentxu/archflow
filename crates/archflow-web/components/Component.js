/**
 * Base Component Class
 */

export class Component {
    constructor(app, options = {}) {
        this.app = app;
        this.element = options.element || null;
        this.options = options;
        this.isActive = false;
    }

    mount() {
        if (this.element) {
            this.bindEvents();
        }
        return this;
    }

    bindEvents() {
        // Override in subclasses
    }

    destroy() {
        // Cleanup
    }

    show() {
        this.element?.classList.remove('hidden');
        this.isActive = true;
    }

    hide() {
        this.element?.classList.add('hidden');
        this.isActive = false;
    }

    enable() {
        this.element?.classList.remove('disabled');
        this.element?.removeAttribute('disabled');
    }

    disable() {
        this.element?.classList.add('disabled');
        this.element?.setAttribute('disabled', '');
    }
}
