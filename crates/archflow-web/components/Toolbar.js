/**
 * Toolbar Component - Top toolbar with actions and simulation
 */

import { Component } from './Component.js';

export class Toolbar extends Component {
    constructor(app, options) {
        super(app, options);
        this.zoomLevel = 100;
        this.isSimulating = false;
    }

    mount() {
        this.bindEvents();
        this.updateZoomDisplay();
        this.addSimulationButton();
        this.addDeployButton();
        return this;
    }

    bindEvents() {
        // Undo button
        const undoBtn = this.element?.querySelector('#btn-undo');
        undoBtn?.addEventListener('click', () => this.handleUndo());

        // Redo button
        const redoBtn = this.element?.querySelector('#btn-redo');
        redoBtn?.addEventListener('click', () => this.handleRedo());

        // Clear button
        const clearBtn = this.element?.querySelector('#btn-clear');
        clearBtn?.addEventListener('click', () => this.handleClear());

        // Zoom controls
        const zoomIn = this.element?.querySelector('#zoom-in');
        const zoomOut = this.element?.querySelector('#zoom-out');
        const zoomFit = this.element?.querySelector('#zoom-fit');

        zoomIn?.addEventListener('click', () => this.handleZoomIn());
        zoomOut?.addEventListener('click', () => this.handleZoomOut());
        zoomFit?.addEventListener('click', () => this.handleZoomFit());

        // Tool buttons
        this.element?.querySelectorAll('[data-tool]').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const tool = e.currentTarget.dataset.tool;
                this.selectTool(tool);
            });
        });
    }

    addSimulationButton() {
        // Add simulation button to toolbar
        const actionsSection = this.element?.querySelector('.toolbar-section.actions');
        if (actionsSection) {
            const simulateBtn = document.createElement('button');
            simulateBtn.className = 'icon-btn';
            simulateBtn.id = 'btn-simulate';
            simulateBtn.title = 'Simulate Architecture';
            simulateBtn.innerHTML = '<i class="ph ph-play-circle"></i>';
            simulateBtn.addEventListener('click', () => this.handleSimulate());
            
            actionsSection.insertBefore(simulateBtn, actionsSection.lastElementChild);
        }
    }

    addDeployButton() {
        // Add deploy button to toolbar
        const actionsSection = this.element?.querySelector('.toolbar-section.actions');
        if (actionsSection) {
            const deployBtn = document.createElement('button');
            deployBtn.className = 'icon-btn';
            deployBtn.id = 'btn-deploy';
            deployBtn.title = 'Deploy Architecture';
            deployBtn.innerHTML = '<i class="ph ph-cloud-upload"></i>';
            deployBtn.addEventListener('click', () => this.handleDeploy());
            
            actionsSection.insertBefore(deployBtn, actionsSection.lastElementChild);
        }
    }

    selectTool(tool) {
        // Update toolbar buttons
        this.element?.querySelectorAll('[data-tool]').forEach(btn => {
            btn.classList.remove('active');
            if (btn.dataset.tool === tool) {
                btn.classList.add('active');
            }
        });

        // Update tool palette
        const toolPalette = this.app.getComponent('toolPalette');
        toolPalette?.selectTool(tool);
    }

    handleUndo() {
        console.log('Undo not implemented in minimal version');
    }

    handleRedo() {
        console.log('Redo not implemented in minimal version');
    }

    handleClear() {
        if (confirm('Clear all shapes?')) {
            this.app.getComponent('canvas')?.clear();
        }
    }

    handleSimulate() {
        const editor = this.app.getEditor();
        if (editor) {
            if (!this.isSimulating) {
                editor.start_simulation();
                this.isSimulating = true;
                this.updateSimulationButton(true);
                this.startSimulationAnimation();
                console.log('Starting simulation...');
            } else {
                editor.stop_simulation();
                this.isSimulating = false;
                this.updateSimulationButton(false);
                this.stopSimulationAnimation();
                console.log('Stopping simulation...');
            }
        }
    }

    startSimulationAnimation() {
        // Add animation effects for simulation
        const canvas = this.app.getComponent('canvas');
        if (canvas) {
            canvas.startSimulationAnimation();
        }
    }

    stopSimulationAnimation() {
        // Remove animation effects
        const canvas = this.app.getComponent('canvas');
        if (canvas) {
            canvas.stopSimulationAnimation();
        }
    }

    handleDeploy() {
        const editor = this.app.getEditor();
        if (editor) {
            // Show loading state with progress
            this.app.showLoading('Deploying architecture...');
            
            // Simulate deployment process with progress updates
            let progress = 0;
            const progressInterval = setInterval(() => {
                progress += 20;
                this.app.showLoading(`Deploying architecture... ${progress}%`);
                
                if (progress >= 100) {
                    clearInterval(progressInterval);
                    
                    // Complete deployment
                    editor.deploy_architecture();
                    this.app.hideLoading();
                    this.app.showError('Architecture deployed successfully!', 3000);
                    console.log('Architecture deployment completed');
                    
                    // Show success animation
                    this.showDeploymentSuccess();
                }
            }, 400);
        }
    }

    showDeploymentSuccess() {
        // Show success animation
        const successOverlay = document.createElement('div');
        successOverlay.className = 'deployment-success';
        successOverlay.innerHTML = `
            <div class="success-content">
                <div class="success-icon">
                    <i class="ph ph-check-circle"></i>
                </div>
                <div class="success-text">Deployment Complete!</div>
            </div>
        `;
        
        document.body.appendChild(successOverlay);
        
        // Remove after animation
        setTimeout(() => {
            successOverlay.style.opacity = '0';
            setTimeout(() => successOverlay.remove(), 300);
        }, 2000);
    }

    updateSimulationButton(isSimulating) {
        const simulateBtn = this.element?.querySelector('#btn-simulate');
        if (simulateBtn) {
            if (isSimulating) {
                simulateBtn.innerHTML = '<i class="ph ph-stop-circle"></i>';
                simulateBtn.title = 'Stop Simulation';
                simulateBtn.classList.add('simulating');
            } else {
                simulateBtn.innerHTML = '<i class="ph ph-play-circle"></i>';
                simulateBtn.title = 'Simulate Architecture';
                simulateBtn.classList.remove('simulating');
            }
        }
    }

    handleZoomIn() {
        const zoom = this.app.getManager('zoom');
        zoom?.zoomIn();
        this.updateZoomDisplay();
    }

    handleZoomOut() {
        const zoom = this.app.getManager('zoom');
        zoom?.zoomOut();
        this.updateZoomDisplay();
    }

    handleZoomFit() {
        const zoom = this.app.getManager('zoom');
        zoom?.reset();
        this.updateZoomDisplay();
    }

    updateZoomDisplay() {
        const zoomEl = this.element?.querySelector('#zoom-level');
        const zoom = this.app.getManager('zoom');
        if (zoomEl && zoom) {
            const level = Math.round(zoom.getLevel() * 100);
            zoomEl.textContent = `${level}%`;
        }
    }

    setZoom(level) {
        this.zoomLevel = level;
        this.updateZoomDisplay();
    }
}
