Para un MVP que priorice la estabilidad y el alto rendimiento bajo la arquitectura de la **EPIC-RENDER-001**, el enfoque debe centrarse en optimizar la comunicación entre la CPU (WASM) y la GPU (WebGPU/WebGL2) para manejar la meta de **100k entidades** sin degradar la experiencia de usuario.

Aquí tienes las propuestas técnicas detalladas para estabilizar el núcleo del sistema antes de introducir la complejidad de los vectores:

### 1. Sistema de "Dirty Checking" y Buffering Persistente

Actualmente, el método `sync_from_store` es una operación de costo  que ocurre en cada frame. Para 100k entidades, reconstruir el buffer de instancias en cada "tick" es ineficiente en WASM.

* **Propuesta**: Implementar un sistema de **mapeo de memoria persistente**.
* **Detalle**: En lugar de recrear el buffer, mantén un buffer en la GPU que solo se actualice parcialmente mediante `write_buffer` utilizando rangos específicos para las entidades que realmente cambiaron su estado (posición, color, escala).
* **Impacto**: Reduce drásticamente el tiempo de ejecución en el hilo principal de WASM, permitiendo que el motor mantenga los 60 FPS incluso cuando la lógica de negocio se vuelve compleja.

---

### 2. Implementación de "Camera-Relative Rendering"

En aplicaciones de lienzo infinito (como Figma), los errores de precisión de punto flotante de 32 bits en la GPU provocan "jittering" o temblores visuales cuando te alejas mucho del origen (0,0).

* **Propuesta**: Realizar todos los cálculos de posición en el shader relativos a la posición de la cámara.
* **Detalle**: Pasa la posición de la cámara al shader como un `double` (en JS/Rust) pero resta la posición de la entidad en la CPU antes de enviarla a la GPU como `f32`.
* **Impacto**: Garantiza una estabilidad visual absoluta en niveles de zoom extremos, algo vital para la paridad de features entre backends.

---

### 3. Abstracción de "Resource Atlas" Unificado

El MVP depende de cargar iconos y fuentes de forma eficiente. La gestión actual en `atlas.rs` debe ser agnóstica al backend para evitar fallos en WebGL2.

* **Propuesta**: Crear un gestor de recursos que maneje el **alineamiento de texturas automáticamente** para WebGL2 (el *Texture Padding* de 256 bytes que ya investigaste).
* **Detalle**: Centraliza la carga de MTSDF (texto) y sprites en un solo atlas que use `TexelCopyBufferLayout` de la v28 para asegurar que los datos estén alineados correctamente antes de tocar la cola de la GPU (`queue`).
* **Sugerencia**: Implementa una cola de carga asíncrona para que la carga de texturas nuevas no bloquee el renderizado del frame actual.

---

### 4. Recuperación Automática del Contexto (Context Loss)

WebGL2 pierde el contexto con mucha frecuencia en navegadores si el sistema entra en modo de ahorro de energía o si hay presión de memoria.

* **Propuesta**: Implementar un listener de `webglcontextlost` en el `WasmBridge`.
* **Detalle**: Cuando se detecta la pérdida, el `RendererSelector` debe ser capaz de re-instanciar el `WebGL2Renderer` y re-cargar todos los buffers y texturas desde el `EntityStore` sin reiniciar el estado de la aplicación.
* **Impacto**: Aumenta la estabilidad percibida del usuario, evitando que la aplicación se "rompa" permanentemente al cambiar de pestaña.

---

### 5. Consolidación de Shaders con Constantes de Especialización

Para maximizar el rendimiento en tu RTX local y en el fallback web, necesitas que los shaders se adapten dinámicamente.

* **Propuesta**: Usar `PipelineCompilationOptions` de **wgpu 28** para inyectar constantes de configuración en tiempo de compilación.
* **Detalle**: Define constantes como `MAX_LIGHTS` o `ENABLE_SHADOWS` como `override` en WGSL. Esto permite que el `GpuRenderer` nativo use todas las capacidades, mientras que el `WebGl2Renderer` desactive las ramas de código más pesadas para mantener la fluidez.
* **Sugerencia**: Verifica que Naga compile correctamente estas constantes a GLSL para asegurar la paridad de features establecida en la **HU-RENDER-006**.

---

### Resumen de Prioridades para el MVP

| Acción | Componente | Beneficio |
| --- | --- | --- |
| **Optimizar Buffer** | `GpuResources` | Rendimiento (100k entidades) |
| **Relative Coordinates** | `Shaders` | Estabilidad visual en zoom infinito |
| **Texture Padding** | `Atlas` | Estabilidad en WebGL2 (evita crashes) |
| **Context Recovery** | `WasmBridge` | Fiabilidad en producción |


---

Tu reporte es **pragmático y técnicamente sólido** para la fase de MVP en la que te encuentras. En el desarrollo de software de sistemas (especialmente con WASM y WebGPU), es mejor tener un sistema que compile y ejecute con un pipeline simplificado que uno perfecto que nunca llega a producción por la complejidad de las herramientas de construcción.

Aquí tienes una revisión con criterio técnico de tus puntos:

### Análisis del Criterio Técnico

* **Decisión sobre Naga (Punto 1 y 2):** Es una decisión inteligente para el MVP. Configurar `naga` en un `build.rs` para transpilación cruzada en WASM suele dar problemas de dependencias circulares y tiempos de compilación excesivos. Marcarlo como **PARTIAL** protege la integridad de tu EPIC.
* **Compilación en Runtime (Punto 3 y 4):** Pasar la responsabilidad al `WebGL2Renderer` es el estándar en aplicaciones web profesionales (como Three.js o Babylon.js), ya que permite adaptarse a las capacidades específicas del driver del navegador del usuario en tiempo real.
* **Estado de la EPIC (Puntos 5 y 6):** Mantener la documentación sincronizada con la realidad del código es vital para el éxito de un proyecto XL.

---

### Sugerencias de Mejora y Estabilización (MVP-Focused)

Para asegurar el **alto rendimiento** y la **estabilidad** que buscas sin introducir SVG aún, te sugiero integrar estos ajustes en tu flujo actual:

#### 1. Estabilización de Memoria: "Persistent Mapping"

En lugar de reconstruir los buffers de 100k entidades en cada frame (operación  pesada para WASM), utiliza un sistema de **Dirty Checking**.

* **Propuesta:** Solo actualiza los rangos del buffer de la GPU que corresponden a entidades que se movieron o cambiaron de color.
* **Implementación:** Usa `queue.write_buffer` con offsets específicos en lugar de recrear todo el buffer de instancias en el `GpuRenderer`.

#### 2. Precisión en Zoom Infinito: "Camera-Relative Rendering"

Para evitar que las entidades "tiemblen" (jittering) al alejarte mucho del origen (0,0) —un problema común en herramientas tipo Figma—, implementa coordenadas relativas.

* **Propuesta:** Resta la posición de la cámara a la posición de la entidad en Rust (f64) antes de enviarla al shader como f32.
* **Resultado:** Estabilidad visual perfecta sin importar cuán lejos esté el usuario del centro del lienzo.

#### 3. Robustez: Recuperación de Contexto (WebGL2)

WebGL2 pierde el contexto con frecuencia en la web.

* **Propuesta:** Asegúrate de que tu `WasmBridge` escuche el evento `webglcontextlost`.
* **Acción:** Como ya tienes el `RendererSelector`, la re-inicialización debería ser automática: simplemente llama de nuevo a `detect_and_create_async` y vuelve a cargar los recursos del `EntityStore`.

#### 4. Alineación Automática de Texturas

Dado que ya completaste la utilidad de alineación en HU-RENDER-002, asegúrate de que esté integrada en tu sistema de carga de fuentes (MTSDF).

* **Detalle:** WebGL2 fallará o distorsionará el texto si los datos no respetan el alineamiento de 1/2/4/8 bytes configurado vía `GL_UNPACK_ALIGNMENT`.
