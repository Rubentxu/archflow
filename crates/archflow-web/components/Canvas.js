/**
 * Canvas Component - Main drawing area
 */

import { Component } from './Component.js';

export class Canvas extends Component {
    constructor(app, options) {
        super(app, options);
        this.ctx = null;
        this.isDragging = false;
        this.dragStart = { x: 0, y: 0 };
        this.isSimulating = false;
        this.particles = [];
        this.animationFrame = null;
        this.particleCount = 0;
    }

    mount() {
        if (!this.element) {
            console.error('Canvas element not found');
            return this;
        }

        this.ctx = this.element.getContext('2d');
        this.resize();

        this.bindEvents();
        this.startRenderLoop();

        return this;
    }

    bindEvents() {
        // Mouse down
        this.element.addEventListener('mousedown', (e) => {
            this.handleMouseDown(e);
        });

        // Mouse move
        this.element.addEventListener('mousemove', (e) => {
            this.handleMouseMove(e);
        });

        // Mouse up
        this.element.addEventListener('mouseup', (e) => {
            this.handleMouseUp(e);
        });

        // Mouse leave
        this.element.addEventListener('mouseleave', () => {
            this.isDragging = false;
        });

        // Wheel (zoom)
        this.element.addEventListener('wheel', (e) => {
            e.preventDefault();
            this.handleWheel(e);
        }, { passive: false });

        // Drag over
        this.element.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = 'copy';
        });

        // Drop
        this.element.addEventListener('drop', (e) => {
            e.preventDefault();
            this.handleDrop(e);
        });

        // Zoom controls
        document.getElementById('zoom-in')?.addEventListener('click', () => {
            this.app.getManager('zoom')?.zoomIn();
            this.updateZoomDisplay();
        });

        document.getElementById('zoom-out')?.addEventListener('click', () => {
            this.app.getManager('zoom')?.zoomOut();
            this.updateZoomDisplay();
        });
    }

    handleDrop(e) {
        const jsonData = e.dataTransfer.getData('application/archflow-component');
        if (!jsonData) return;

        try {
            const data = JSON.parse(jsonData);
            const rect = this.element.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            this.app.getComponent('library')?.createShapeFromLibrary(
                data.categoryId,
                data.itemId,
                x,
                y
            );
        } catch (error) {
            console.error('Failed to parse drop data:', error);
        }
    }

    updateZoomDisplay() {
        const zoomLevel = this.app.getManager('zoom')?.getLevel() || 1;
        const zoomEl = document.getElementById('zoom-level');
        if (zoomEl) {
            zoomEl.textContent = `${Math.round(zoomLevel * 100)}%`;
        }
    }

    handleMouseDown(e) {
        const rect = this.element.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        this.isDragging = true;
        this.dragStart = { x, y };

        // Delegate to WASM
        const editor = this.app.getEditor();
        if (editor) {
            editor.on_mousedown(x, y, e.button);
        }

        this.updateStatusPosition(x, y);
    }

    handleMouseMove(e) {
        const rect = this.element.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        if (this.isDragging) {
            const editor = this.app.getEditor();
            if (editor) {
                editor.on_mousemove(x, y);
            }
        }

        this.updateStatusPosition(x, y);
        this.updateCursor(e);
    }

    handleMouseUp(e) {
        const rect = this.element.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        this.isDragging = false;

        const editor = this.app.getEditor();
        if (editor) {
            editor.on_mouseup(x, y);
        }
    }

    handleWheel(e) {
        const rect = this.element.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const zoomOut = e.deltaY > 0;

        const editor = this.app.getEditor();
        if (editor) {
            editor.on_wheel(x, y, zoomOut);
        }

        this.app.getComponent('statusBar')?.updateZoom();
    }

    updateStatusPosition(x, y) {
        const statusBar = this.app.getComponent('statusBar');
        if (statusBar) {
            statusBar.setPosition(x, y);
        }
    }

    updateCursor(e) {
        const tool = this.app.getComponent('toolPalette')?.currentTool;

        this.element.className = '';

        switch (tool) {
            case 'rect':
            case 'ellipse':
            case 'line':
                this.element.classList.add('cursor-crosshair');
                break;
            case 'hand':
                this.element.classList.add(this.isDragging ? 'cursor-grabbing' : 'cursor-grab');
                break;
            default:
                this.element.classList.add('cursor-default');
        }
    }

    resize() {
        const container = this.options.container;
        if (container) {
            const rect = container.getBoundingClientRect();
            this.element.width = rect.width;
            this.element.height = rect.height;
        }
    }

    render() {
        const editor = this.app.getEditor();
        if (editor) {
            editor.render();
        }
    }

    startRenderLoop() {
        const loop = () => {
            this.render();
            this.updateStats();
            requestAnimationFrame(loop);
        };
        requestAnimationFrame(loop);
    }

    updateStats() {
        const statusBar = this.app.getComponent('statusBar');
        if (statusBar) {
            const editor = this.app.getEditor();
            if (editor) {
                statusBar.setShapeCount(editor.shape_count());
                statusBar.setSelectionCount(editor.selection_count());
            }
        }
    }

    startSimulationAnimation() {
        this.isSimulating = true;
        this.particleCount = 0;
        this.particles = [];
        this.animateSimulation();
    }

    stopSimulationAnimation() {
        this.isSimulating = false;
        if (this.animationFrame) {
            cancelAnimationFrame(this.animationFrame);
            this.animationFrame = null;
        }
        this.particles = [];
        this.ctx.clearRect(0, 0, this.element.width, this.element.height);
    }

    animateSimulation() {
        if (!this.isSimulating) return;

        // Clear canvas
        this.ctx.clearRect(0, 0, this.element.width, this.element.height);

        // Generate new particles
        if (this.particleCount < 50) {
            this.particles.push({
                x: Math.random() * this.element.width,
                y: Math.random() * this.element.height,
                vx: (Math.random() - 0.5) * 2,
                vy: (Math.random() - 0.5) * 2,
                size: Math.random() * 3 + 1,
                color: `hsl(${Math.random() * 60 + 180}, 70%, 50%)`,
                life: 1.0
            });
            this.particleCount++;
        }

        // Update and draw particles
        this.particles = this.particles.filter(particle => {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.life -= 0.01;

            // Bounce off walls
            if (particle.x < 0 || particle.x > this.element.width) particle.vx *= -1;
            if (particle.y < 0 || particle.y > this.element.height) particle.vy *= -1;

            // Draw particle
            this.ctx.globalAlpha = particle.life;
            this.ctx.fillStyle = particle.color;
            this.ctx.beginPath();
            this.ctx.arc(particle.x, particle.y, particle.size, 0, Math.PI * 2);
            this.ctx.fill();

            return particle.life > 0;
        });

        this.animationFrame = requestAnimationFrame(() => this.animateSimulation());
    }

    clear() {
        const editor = this.app.getEditor();
        if (editor) {
            const shapes = editor.get_all_shapes();
            if (shapes && shapes.length) {
                for (const shape of shapes) {
                    editor.delete_shape(shape.id);
                }
            }
        }
    }
}
