// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Cookbook - Shared Utilities
// ═══════════════════════════════════════════════════════════════════════════════
//
// Extensiones y utilidades reutilizables para los ejemplos del cookbook.
// Construido sobre archflow-client.js
//
// ═══════════════════════════════════════════════════════════════════════════════

import { createEngine, EventType, SensorType, ControllerType, ActuatorType, addBehavior, queryEntities } from './archflow-client.js';

// ═══════════════════════════════════════════════════════════════════════════════
// UTILIDADES DE COLOR
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Convertir color hex a objeto RGBA
 * @param {string} hex - Color en formato hex (#RRGGBB o #RRGGBBAA)
 * @returns {{r: number, g: number, b: number, a: number}} Componentes RGBA (0-255)
 */
export function hexToRgba(hex) {
    hex = hex.replace('#', '');

    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    const a = hex.length === 8 ? parseInt(hex.substring(6, 8), 16) : 255;

    return { r, g, b, a };
}

/**
 * Convertir RGBA a packed u32 (formato ABGR para WebGL)
 * @param {number} r - Red (0-255)
 * @param {number} g - Green (0-255)
 * @param {number} b - Blue (0-255)
 * @param {number} a - Alpha (0-255)
 * @returns {number} Color packed como u32
 */
export function rgbaToPacked(r, g, b, a) {
    return (a << 24) | (b << 16) | (g << 8) | r;
}

/**
 * Convertir color hex a packed u32
 * @param {string} hex - Color hex
 * @returns {number} Color packed como u32
 */
export function hexToPacked(hex) {
    const { r, g, b, a } = hexToRgba(hex);
    return rgbaToPacked(r, g, b, a);
}

// ═══════════════════════════════════════════════════════════════════════════════
// CREACIÓN DE SHAPES
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Crear un rectángulo con color y stroke personalizados
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} x - Posición X
 * @param {number} y - Posición Y
 * @param {number} width - Ancho
 * @param {number} height - Alto
 * @param {string} fillColor - Color de relleno (hex)
 * @param {string} strokeColor - Color del borde (hex)
 * @param {number} strokeWidth - Grosor del borde
 * @returns {Promise<number>} Entity ID
 */
export async function createRectangle(bridge, x, y, width, height, fillColor, strokeColor, strokeWidth) {
    const fill = hexToRgba(fillColor);
    const stroke = hexToRgba(strokeColor);

    // Configurar colores activos
    bridge.set_active_color(fill.r, fill.g, fill.b, fill.a);
    bridge.set_active_stroke_color(stroke.r, stroke.g, stroke.b, stroke.a);
    bridge.set_active_stroke_width(strokeWidth);

    // Crear entidad
    const id = await bridge.spawn_entity(x, y, width, height);

    // Setear tipo de shape (0 = Rectangle)
    bridge.configure_entity(id, x, y, width, height, NaN, NaN, NaN, NaN, 0, 0, 0, 0, 2);

    return id;
}

/**
 * Crear un círculo
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} x - Centro X
 * @param {number} y - Centro Y
 * @param {number} radius - Radio
 * @param {string} color - Color (hex)
 * @returns {Promise<number>} Entity ID
 */
export async function createCircle(bridge, x, y, radius, color) {
    const rgba = hexToRgba(color);

    bridge.set_active_color(rgba.r, rgba.g, rgba.b, rgba.a);

    // Para círculos, width y height son 2*radius
    const id = await bridge.spawn_entity(x, y, radius * 2, radius * 2);

    // Shape type 1 = Circle
    bridge.configure_entity(id, NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN, 0, 0, 0, 1, 2);

    return id;
}

/**
 * Crear un triángulo
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} x - Posición X
 * @param {number} y - Posición Y
 * @param {number} size - Tamaño
 * @param {string} color - Color (hex)
 * @returns {Promise<number>} Entity ID
 */
export async function createTriangle(bridge, x, y, size, color) {
    const rgba = hexToRgba(color);

    bridge.set_active_color(rgba.r, rgba.g, rgba.b, rgba.a);

    const id = await bridge.spawn_entity(x, y, size, size);

    // Shape type 2 = Triangle
    bridge.configure_entity(id, NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN, 0, 0, 0, 2, 2);

    return id;
}

// ═══════════════════════════════════════════════════════════════════════════════
// BULK OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Crear un grid de entidades
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} cols - Columnas
 * @param {number} rows - Filas
 * @param {number} spacing - Espaciado entre celdas
 * @param {number} size - Tamaño de cada entidad
 * @param {string} baseColor - Color base (hex)
 * @returns {Promise<Uint32Array>} Array de entity IDs
 */
export async function spawnGrid(bridge, cols, rows, spacing, size, baseColor) {
    const count = cols * rows;
    const positions = new Float32Array(count * 2);
    const sizes = new Float32Array(count * 2);
    const colors = new Uint8Array(count * 4);

    const rgba = hexToRgba(baseColor);

    // Calcular offset para centrar
    const canvasWidth = 1920;
    const canvasHeight = 1080;
    const gridWidth = (cols - 1) * spacing;
    const gridHeight = (rows - 1) * spacing;
    const startX = (canvasWidth - gridWidth) / 2;
    const startY = (canvasHeight - gridHeight) / 2;

    let index = 0;
    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < cols; col++) {
            const x = startX + col * spacing;
            const y = startY + row * spacing;

            positions[index * 2 + 0] = x;
            positions[index * 2 + 1] = y;

            sizes[index * 2 + 0] = size;
            sizes[index * 2 + 1] = size;

            // Variación de color por fila
            const variation = (row * cols + col) * 5;
            colors[index * 4 + 0] = Math.min(255, rgba.r + variation);
            colors[index * 4 + 1] = rgba.g;
            colors[index * 4 + 2] = rgba.b;
            colors[index * 4 + 3] = 255;

            index++;
        }
    }

    const ids = await bridge.bulk_spawn(positions, sizes, colors);
    console.log(`Spawned ${ids.length} entities in grid`);

    return ids;
}

/**
 * Crear sistema de partículas
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} count - Número de partículas
 * @param {number} centerX - Centro X de explosión
 * @param {number} centerY - Centro Y de explosión
 * @param {number} maxSpeed - Velocidad máxima
 * @returns {Promise<{ids: Uint32Array, velocities: Float32Array}>}
 */
export async function spawnParticles(bridge, count, centerX, centerY, maxSpeed) {
    const positions = new Float32Array(count * 2);
    const sizes = new Float32Array(count * 2);
    const colors = new Uint8Array(count * 4);
    const velocities = new Float32Array(count * 2);

    for (let i = 0; i < count; i++) {
        positions[i * 2 + 0] = centerX;
        positions[i * 2 + 1] = centerY;

        const size = 5 + Math.random() * 15;
        sizes[i * 2 + 0] = size;
        sizes[i * 2 + 1] = size;

        const speed = Math.random() * maxSpeed;
        const angle = Math.random() * Math.PI * 2;
        velocities[i * 2 + 0] = Math.cos(angle) * speed;
        velocities[i * 2 + 1] = Math.sin(angle) * speed;

        // Color basado en velocidad
        const speedRatio = speed / maxSpeed;
        colors[i * 4 + 0] = Math.floor(255 * speedRatio);
        colors[i * 4 + 1] = Math.floor(255 * (1 - speedRatio));
        colors[i * 4 + 2] = 100;
        colors[i * 4 + 3] = 255;
    }

    const ids = await bridge.bulk_spawn(positions, sizes, colors);

    // Setear velocidades usando ECB
    const ecb = bridge.create_ecb(count);
    for (let i = 0; i < count; i++) {
        ecb.set_velocity(ids[i], velocities[i * 2 + 0], velocities[i * 2 + 1]);
    }
    await bridge.execute_ecb(ecb);

    console.log(`Spawned ${count} particles with velocities`);

    return { ids, velocities };
}

// ═══════════════════════════════════════════════════════════════════════════════
// MANIPULACIÓN DIRECTA
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Obtener transforms actuales (zero-copy)
 * @param {WasmBridge} bridge - Instancia del bridge
 * @returns {Float32Array} Array de transforms [x, y, width, height]
 */
export function getTransforms(bridge) {
    const ptr = bridge.get_transforms_ptr();
    const count = bridge.get_transforms_count();
    return new Float32Array(wasmMemory.buffer, ptr, count * 4);
}

/**
 * Mover una entidad a una posición absoluta
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 * @param {number} x - Nueva posición X
 * @param {number} y - Nueva posición Y
 */
export function moveEntityTo(bridge, entityId, x, y) {
    bridge.configure_entity(
        entityId,
        x, y,           // posición
        NaN, NaN,       // skip size
        NaN, NaN,       // skip velocity
        NaN,            // skip color
        0,              // skip stroke_color
        0,              // skip stroke_width
        255,            // skip shape
        2               // skip visibility
    );
}

/**
 * Escalar una entidad
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 * @param {number} scaleX - Escala X
 * @param {number} scaleY - Escala Y
 */
export function scaleEntity(bridge, entityId, scaleX, scaleY) {
    const transforms = getTransforms(bridge);
    const idx = entityId;

    const currentWidth = transforms[idx * 4 + 2];
    const currentHeight = transforms[idx * 4 + 3];

    bridge.configure_entity(
        entityId,
        NaN, NaN,
        currentWidth * scaleX,
        currentHeight * scaleY,
        NaN, NaN, NaN, NaN, 0, 0, 0, 255, 2
    );
}

/**
 * Cambiar color de una entidad
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 * @param {string} color - Color (hex)
 */
export function setEntityColor(bridge, entityId, color) {
    const packed = hexToPacked(color);
    bridge.configure_entity(
        entityId,
        NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN,
        packed, 0, 0, 255, 2
    );
}

/**
 * Eliminar una entidad
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 */
export async function deleteEntity(bridge, entityId) {
    const ecb = bridge.create_ecb(1);
    ecb.despawn(entityId);
    await bridge.execute_ecb(ecb);
}

// ═══════════════════════════════════════════════════════════════════════════════
// DRAG AND DROP
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Configurar una entidad para ser draggable (Logic Bricks)
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 */
export function makeDraggable(bridge, entityId) {
    bridge.add_sensor(
        entityId,
        SensorType.MOUSE_CLICK,
        ControllerType.AND,
        ActuatorType.MOVE
    );

    console.log(`Entity ${entityId} is now draggable`);
}

// ═══════════════════════════════════════════════════════════════════════════════
// TWEEN ANIMATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Sistema de animaciones Tween
 */
export class TweenSystem {
    constructor(bridge) {
        this.bridge = bridge;
        this.tweens = [];
        this.isRunning = false;
    }

    /**
     * Crear un tween de posición
     */
    moveTo(entityId, targetX, targetY, duration, easing = 'easeInOut', callbacks = {}) {
        const transforms = getTransforms(this.bridge);
        const idx = entityId;

        const startX = transforms[idx * 4 + 0];
        const startY = transforms[idx * 4 + 1];

        const tween = {
            entityId,
            type: 'position',
            start: { x: startX, y: startY },
            target: { x: targetX, y: targetY },
            duration,
            elapsed: 0,
            easing,
            onUpdate: callbacks.onUpdate,
            onComplete: callbacks.onComplete
        };

        this.tweens.push(tween);

        if (!this.isRunning) {
            this.start();
        }

        return tween;
    }

    /**
     * Crear un tween de escala
     */
    scaleTo(entityId, targetScale, duration, easing = 'easeInOut', callbacks = {}) {
        const transforms = getTransforms(this.bridge);
        const idx = entityId;

        const startWidth = transforms[idx * 4 + 2];
        const startHeight = transforms[idx * 4 + 3];

        const tween = {
            entityId,
            type: 'scale',
            start: { width: startWidth, height: startHeight },
            target: {
                width: startWidth * targetScale,
                height: startHeight * targetScale
            },
            duration,
            elapsed: 0,
            easing,
            onUpdate: callbacks.onUpdate,
            onComplete: callbacks.onComplete
        };

        this.tweens.push(tween);

        if (!this.isRunning) {
            this.start();
        }

        return tween;
    }

    /**
     * Crear un tween de color
     */
    colorTo(entityId, targetColor, duration, easing = 'easeInOut', callbacks = {}) {
        const colors = this.getColors(this.bridge);
        const idx = entityId;

        const currentColor = colors[idx];
        const startR = currentColor & 0xFF;
        const startG = (currentColor >> 8) & 0xFF;
        const startB = (currentColor >> 16) & 0xFF;
        const startA = (currentColor >> 24) & 0xFF;

        const target = hexToRgba(targetColor);

        const tween = {
            entityId,
            type: 'color',
            start: { r: startR, g: startG, b: startB, a: startA },
            target: { r: target.r, g: target.g, b: target.b, a: target.a },
            duration,
            elapsed: 0,
            easing,
            onUpdate: callbacks.onUpdate,
            onComplete: callbacks.onComplete
        };

        this.tweens.push(tween);

        if (!this.isRunning) {
            this.start();
        }

        return tween;
    }

    start() {
        this.isRunning = true;
        this.lastTime = performance.now();
        this.update();
    }

    update() {
        if (!this.isRunning && this.tweens.length === 0) {
            return;
        }

        const currentTime = performance.now();
        const deltaTime = currentTime - this.lastTime;
        this.lastTime = currentTime;

        for (let i = this.tweens.length - 1; i >= 0; i--) {
            const tween = this.tweens[i];
            tween.elapsed += deltaTime;

            const progress = Math.min(tween.elapsed / tween.duration, 1);
            const easedProgress = this.applyEasing(progress, tween.easing);

            this.applyTween(tween, easedProgress);

            if (tween.onUpdate) {
                tween.onUpdate(easedProgress);
            }

            if (progress >= 1) {
                this.applyTween(tween, 1);

                if (tween.onComplete) {
                    tween.onComplete();
                }

                this.tweens.splice(i, 1);
            }
        }

        if (this.tweens.length > 0) {
            requestAnimationFrame(this.update.bind(this));
        } else {
            this.isRunning = false;
        }
    }

    applyTween(tween, progress) {
        const { entityId, type, start, target } = tween;

        switch (type) {
            case 'position': {
                const x = start.x + (target.x - start.x) * progress;
                const y = start.y + (target.y - start.y) * progress;

                this.bridge.configure_entity(
                    entityId,
                    x, y, NaN, NaN, NaN, NaN, NaN, NaN, 0, 0, 0, 255, 2
                );
                break;
            }

            case 'scale': {
                const width = start.width + (target.width - start.width) * progress;
                const height = start.height + (target.height - start.height) * progress;

                this.bridge.configure_entity(
                    entityId,
                    NaN, NaN, width, height, NaN, NaN, NaN, NaN, 0, 0, 0, 255, 2
                );
                break;
            }

            case 'color': {
                const r = Math.floor(start.r + (target.r - start.r) * progress);
                const g = Math.floor(start.g + (target.g - start.g) * progress);
                const b = Math.floor(start.b + (target.b - start.b) * progress);
                const a = Math.floor(start.a + (target.a - start.a) * progress);

                const packed = (a << 24) | (b << 16) | (g << 8) | r;

                this.bridge.configure_entity(
                    entityId,
                    NaN, NaN, NaN, NaN, NaN, NaN, NaN, NaN,
                    packed, 0, 0, 255, 2
                );
                break;
            }
        }
    }

    applyEasing(t, easing) {
        switch (easing) {
            case 'linear': return t;
            case 'easeIn': return t * t;
            case 'easeOut': return t * (2 - t);
            case 'easeInOut': return t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t;
            case 'easeInCubic': return t * t * t;
            case 'easeOutCubic': return --t * t * t + 1;
            case 'easeInOutCubic': return t < 0.5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1;
            case 'bounce':
                if (t < 1 / 2.75) return 7.5625 * t * t;
                if (t < 2 / 2.75) return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
                if (t < 2.5 / 2.75) return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
                return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
            default: return t;
        }
    }

    getTransforms() {
        return getTransforms(this.bridge);
    }

    getColors(bridge) {
        const ptr = bridge.get_colors_ptr();
        const count = bridge.get_transforms_count();
        return new Uint32Array(wasmMemory.buffer, ptr, count);
    }

    stop(entityId) {
        this.tweens = this.tweens.filter(t => t.entityId !== entityId);
    }

    stopAll() {
        this.tweens = [];
        this.isRunning = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SELECCIÓN
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Configurar entidad para ser seleccionable con clic
 * @param {WasmBridge} bridge - Instancia del bridge
 * @param {number} entityId - ID de la entidad
 */
export function makeSelectable(bridge, entityId) {
    bridge.add_sensor(
        entityId,
        SensorType.MOUSE_CLICK,
        ControllerType.DIRECT,
        ActuatorType.SELECT
    );

    console.log(`Entity ${entityId} is now selectable`);
}

/**
 * Obtener entidades actualmente seleccionadas
 * @param {WasmBridge} bridge - Instancia del bridge
 * @returns {Uint32Array} Array de entity IDs seleccionadas
 */
export function getSelectedEntities(bridge) {
    const selected = bridge.query_by_selection(true);
    return selected || new Uint32Array(0);
}

/**
 * Limpiar selección
 * @param {WasmBridge} bridge - Instancia del bridge
 */
export function clearSelection(bridge) {
    const selected = getSelectedEntities(bridge);

    for (let i = 0; i < selected.length; i++) {
        const id = selected[i];
        bridge.set_selected(id, false);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PHYSICS
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Sistema de física simplificado
 */
export class PhysicsSystem {
    constructor(bridge) {
        this.bridge = bridge;
        this.gravity = { x: 0, y: 500 }; // pixels/s²
        this.physicsBodies = new Map();
    }

    /**
     * Configurar entidad como cuerpo físico
     * @param {number} entityId - ID de la entidad
     * @param {Object} options - Opciones de configuración
     */
    setupPhysicsBody(entityId, options = {}) {
        const {
            mass = 1.0,
            restitution = 0.7,
            gravityScale = 1.0,
            isStatic = false
        } = options;

        this.physicsBodies.set(entityId, {
            mass,
            restitution,
            gravityScale,
            isStatic
        });

        if (!isStatic) {
            this.applyGravity(entityId);
        }
    }

    /**
     * Aplicar gravedad a una entidad
     */
    applyGravity(entityId) {
        const body = this.physicsBodies.get(entityId);
        if (!body || body.isStatic) return;

        this.bridge.configure_entity(
            entityId,
            NaN, NaN, NaN, NaN,
            NaN, NaN,
            this.gravity.x * body.gravityScale,
            this.gravity.y * body.gravityScale,
            NaN, 0, 0, 255, 2
        );
    }

    /**
     * Aplicar impulso a una entidad
     */
    applyImpulse(entityId, impulseX, impulseY) {
        const body = this.physicsBodies.get(entityId);
        if (!body) return;

        const dvx = impulseX / body.mass;
        const dvy = impulseY / body.mass;

        this.bridge.configure_entity(
            entityId,
            NaN, NaN, NaN, NaN,
            dvx, dvy,
            NaN, NaN, NaN, 0, 0, 0, 255, 2
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILIDADES DE UI
// ═══════════════════════════════════════════════════════════════════════════════

/**
 * Crear elemento de información en pantalla
 * @param {string} id - ID del elemento
 * @param {string} message - Mensaje inicial
 */
export function createInfoElement(id = 'info') {
    const info = document.createElement('div');
    info.id = id;
    info.style.cssText = `
        position: fixed;
        top: 20px;
        left: 20px;
        background: rgba(0, 0, 0, 0.8);
        color: #0f0;
        padding: 15px;
        border-radius: 8px;
        font-family: 'Monaco', 'Consolas', monospace;
        font-size: 14px;
        max-width: 400px;
        z-index: 1000;
        pointer-events: none;
    `;
    info.textContent = message;
    document.body.appendChild(info);
    return info;
}

/**
 * Actualizar mensaje de información
 * @param {string} id - ID del elemento
 * @param {string} message - Nuevo mensaje
 * @param {boolean} isError - Es un error
 */
export function updateInfo(id, message, isError = false) {
    const info = document.getElementById(id);
    if (info) {
        info.textContent = message;
        info.style.color = isError ? '#f55' : '#0f0';
    }
    console.log(message);
}

/**
 * Crear botón flotante
 * @param {string} text - Texto del botón
 * @param {Function} onClick - Callback al hacer clic
 * @param {Object} options - Opciones de estilo
 */
export function createButton(text, onClick, options = {}) {
    const {
        x = 20,
        y = 20,
        bgColor = '#4488ff',
        textColor = '#ffffff',
        padding = '10px 20px'
    } = options;

    const button = document.createElement('button');
    button.textContent = text;
    button.style.cssText = `
        position: fixed;
        left: ${x}px;
        top: ${y}px;
        background: ${bgColor};
        color: ${textColor};
        border: none;
        padding: ${padding};
        border-radius: 6px;
        font-family: system-ui, -apple-system, sans-serif;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        z-index: 1000;
        transition: transform 0.1s, background-color 0.2s;
    `;

    button.addEventListener('mouseenter', () => {
        button.style.transform = 'scale(1.05)';
        button.style.backgroundColor = '#5599ff';
    });

    button.addEventListener('mouseleave', () => {
        button.style.transform = 'scale(1.0)';
        button.style.backgroundColor = bgColor;
    });

    button.addEventListener('click', (e) => {
        e.preventDefault();
        onClick(e);
    });

    document.body.appendChild(button);

    return button;
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXPORTS
// ═══════════════════════════════════════════════════════════════════════════════

export {
    createEngine,
    EventType,
    SensorType,
    ControllerType,
    ActuatorType,
    addBehavior,
    queryEntities
};
