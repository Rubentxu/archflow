# ArchFlow SDK - Estudio Final de Diseño y Arquitectura

## Resumen Ejecutivo

Este documento presenta el estudio final y la propuesta arquitectónica definitiva para el SDK de ArchFlow, un kit de desarrollo que expone la funcionalidad del motor de renderizado de alto rendimiento basado en Rust a desarrolladores web. El SDK sigue patrones establecidos por herramientas líderes como tldraw y Figma, pero con una propuesta de valor diferenciadora: **delegar completamente toda la lógica de negocio, estado y renderizado al motor Rust**, mientras que la capa JavaScript actúa únicamente como un adapter minimalista de presentación y eventos.

La investigación realizada revela que la decisión arquitectónica crítica no es qué funcionalidades incluir, sino **dónde reside cada funcionalidad**. El análisis comparativo entre tldraw, Figma y otras soluciones del mercado demuestra que los SDKs más exitosos son aquellos que minimizan la superficie de API en JavaScript mientras maximizan la capacidad expresiva del motor subyacente. ArchFlow adopta este principio de forma radical: el motor Rust gestiona el estado, las transformaciones geométricas, la indexación espacial, el sistema CRDT para colaboración y el renderizado mediante WebGPU. La capa JavaScript se limita a traducir eventos del DOM a llamadas WASM y a consumir buffers compartidos para el renderizado en canvas.

El documento también aborda una pregunta fundamental que surgió durante el análisis: ¿deben implementarse características como el canvas infinito, los fondos, las grids y el soporte SVG en Rust o en JavaScript? La respuesta, respaldada por análisis de rendimiento y casos de uso, es que **Rust debe gestionar todo el procesamiento y estado relacionado con estas características**, mientras que JavaScript únicamente maneja la configuración inicial y la presentación visual final. Esta decisión maximiza el rendimiento, garantiza la consistencia del estado y facilita la implementación de características colaborativas en tiempo real.

---

## 1. Introducción y Contexto del Proyecto

### 1.1 Motivación para un SDK de Alto Rendimiento

ArchFlow nace con una ambición clara: proporcionar una plataforma de diagramación y colaboración que combine la potencia del lenguaje Rust con la accesibilidad del ecosistema web. El motor de renderizado desarrollado en Rust ofrece capacidades excepcionales en términos de rendimiento, tipo-seguridad y manejo de memoria, pero estas ventajas serían inaccesibles para la mayoría de desarrolladores web sin una capa de abstracción apropiada.

El objetivo del SDK no es simplemente hacer disponible el motor Rust, sino hacerlo de una manera que resulte natural y productiva para desarrolladores accustomed a APIs JavaScript modernas. Esto significa proporcionar tipado completo en TypeScript, integración fluida con frameworks como React, Vue o Vanilla JavaScript, y un modelo de programación que oculte la complejidad del bridge Rust-WASM mientras preserva el rendimiento subyacente.

### 1.2 Requisitos del Producto según el PRD

El Product Requirements Document define varios requisitos fundamentales que el SDK debe satisfacer para alinearse con la visión del producto:

El requisito de **colaboración en tiempo real** establecido en la sección 3.4 del PRD especifica la necesidad de un sistema multiusuario con cursores en vivo, sincronización de selección y cambios, e integración con Git para estrategias de branch y merge. Este requisito tiene implicaciones directas en la arquitectura del SDK: debe exponer primitivas de sincronización, gestionar la resolución de conflictos y proporcionar eventos que permitan a la capa JavaScript actualizar la UI de forma reactiva.

El requisito de **diagramas con animaciones multicapa estilo C4** descrito en la sección 3.1.1 demanda un sistema de renderizado que soporte múltiples capas semánticas con transiciones suaves entre niveles de zoom. Esta característica requiere una arquitectura de renderizado sofisticada donde el estado de la animación, las transformaciones de vista y la lógica de nivel C4 residan en el motor Rust para garantizar coherencia y rendimiento.

### 1.3 Alcance del Estudio

Este estudio aborda las siguientes cuestiones fundamentales para el diseño del SDK:

La primera cuestión es la **delegación de responsabilidades entre Rust y JavaScript**, determinando qué funcionalidades deben implementarse en cada capa para maximizar rendimiento, mantenibilidad y experiencia de desarrollador. La segunda cuestión es el **diseño de la API pública**, estableciendo las interfaces TypeScript que los desarrolladores utilizarán y los patrones de programación que las gobiernan. La tercera cuestión es la **estrategia de renderizado**, decidiendo entre canvas 2D, WebGPU, SVG o híbridos según los requisitos de rendimiento y funcionalidad. La cuarta cuestión es el **sistema de plugins y extensibilidad**, definiendo cómo los desarrolladores podrán extender el SDK sin comprometer la arquitectura core.

---

## 2. Análisis de la Arquitectura de Crates Existente

### 2.1 Inventario y Propósito de Cada Crate

El motor Rust de ArchFlow está organizado en una serie de crates que implementan responsabilidades específicas y bien delimitadas. Comprender esta organización es esencial para diseñar un SDK que exponga estas capacidades de forma coherente.

El crate **archflow-core** constituye el fundamento sobre el cual se construyen todas las demás funcionalidades. Define tipos geométricos fundamentales como Vec2, Vec3, Mat3 para álgebra lineal, Rect para rectángulos y bounds, Color con soporte para espacios de color RGBA y HSLA, EntityId para identificación única de entidades, Transform para matrices de transformación 2D, y Animation con soporte para keyframes y easing. Este crate es absolutamente esencial y no puede omitirse del SDK bajo ninguna circunstancia.

El crate **archflow-renderers** encapsula el sistema de renderizado de alto rendimiento basado en WebGPU. Implementa BatchRenderer2D para renderizado por lotes que minimiza cambios de estado GPU, RenderContext como abstracción sobre el contexto WebGPU, Renderable trait para objetos renderizables, y MaterialId para gestión de materiales. Este crate es el diferenciador clave de rendimiento frente a competidores y debe exponerse completamente a través del SDK.

El crate **archflow-geometry** proporciona algoritmos geométricos especializados utilizando kurbo como biblioteca base. Incluye operaciones de intersección para detección de colisiones, operaciones booleanas sobre polígonos, cálculo de convex hull, y Signed Distance Fields para efectos visuales avanzados. Esta funcionalidad es crítica para hit-testing preciso y efectos visuales sofisticados.

El crate **archflow-spatial** implementa estructuras de optimización espacial, específicamente RTree para indexación espacial eficiente, ViewportManager para gestión del área visible, y SpatialHash para colisiones optimizadas. Este crate es fundamental para el rendimiento con grandes cantidades de objetos en el canvas.

El crate **archflow-records** implementa el sistema de almacenamiento con versionado, incluyendo RecordStore con control de cambios, FractionalIndex para ordenamiento de registros, y delta encoding para sincronización eficiente. Este crate es la base del sistema de undo/redo y de la serialización del estado.

El crate **archflow-collab** proporciona el protocolo de colaboración en tiempo real, con Network como cliente/servidor WebSocket, SharedBuffer utilizando SharedArrayBuffer para zero-copy, y sync protocol basado en CRDT para sincronización conflict-free. Este crate es absolutamente necesario según el requisito de colaboración del PRD.

### 2.2 Matriz de Decisión para el SDK

Cada crate debe clasificarse según su relevancia para el SDK público:

| Crate | Relevancia SDK | Tipo de Exposición | Justificación |
|-------|----------------|-------------------|---------------|
| archflow-core | Esencial | Completa | Tipos base requeridos por todas las APIs |
| archflow-renderers | Esencial | Completa | Diferenciador de rendimiento principal |
| archflow-geometry | Esencial | Selectiva | Solo APIs de alto nivel para hit-testing |
| archflow-spatial | Importante | Abstracción | Ocultar R-tree, exponer query APIs |
| archflow-records | Esencial | Completa | Estado, ChangeSet, FractionalIndex |
| archflow-collab | Esencial | Completa | Requisito PRD de colaboración |
| archflow-ecs-hybrid | Opcional | Feature-gated | Solo para animaciones avanzadas |

### 2.3 Dependencias y Orden de Compilación

La organización de crates refleja dependencias lógicas que deben respetarse en el SDK:

El nivel más bajo contiene archflow-core, que no tiene dependencias internas y es consumido por todos los demás crates. Sobre él se construyen archflow-geometry y archflow-spatial, que utilizan los tipos base pero son independientes entre sí. archflow-records consume core, geometry y spatial para implementar el sistema de estado. archflow-renderers consume core, geometry y records para el sistema de renderizado. archflow-collab consume records para la sincronización de estado. Finalmente, archflow-ecs-hybrid consume records y renderers para la sincronización con el sistema de entidades.

Esta arquitectura en capas tiene implicaciones directas para el SDK: las APIs de nivel superior pueden consumirse sin conocer los detalles de implementación de las capas inferiores, pero siempre deben construirse sobre las abstracciones correctas.

---

## 3. Decisión Architectural Fundamental: Rust versus JavaScript

### 3.1 Principio de Delegation Complete al Motor Rust

La decisión más importante en el diseño del SDK es **qué funcionalidad reside en Rust y qué funcionalidad reside en JavaScript**. Después de analizar las arquitecturas de tldraw, Figma, y otras soluciones del mercado, junto con los requisitos específicos del PRD, la conclusión es clara: **el SDK debe delegar completamente toda la lógica de negocio, estado, transformaciones, colisiones y renderizado al motor Rust**.

Esta decisión se fundamenta en varios argumentos técnicos y de producto. El primer argumento es la **consistencia del estado**: cuando el estado reside exclusivamente en Rust, no existe la posibilidad de que JavaScript y Rust tengan visiones不一致 del estado. Los bugs de sincronización entre capas, que son extremadamente difíciles de diagnosticar y corregir, se eliminan por diseño.

El segundo argumento es la **facilidad de colaboración**: el sistema CRDT de archflow-collab opera sobre el estado en Rust. Si cualquier lógica de negocio residiera en JavaScript, la colaboración en tiempo real requeriría sincronización bidireccional entre JavaScript y Rust, duplicando la complejidad del sistema de merge.

El tercer argumento es el **rendimiento**: las transformaciones geométricas, la detección de colisiones, la indexación espacial y el renderizado por lotes son operaciones intensivas que Rust maneja con eficiencia excepcional. JavaScript, aunque potente, introduce overhead de garbage collection y tipado dinámico que afecta el rendimiento en escenarios de alta carga.

El cuarto argumento es la **tipo-seguridad**: Rust garantiza en tiempo de compilación invariantes que JavaScript solo puede verificar en tiempo de ejecución. Al mantener la lógica crítica en Rust, el SDK puede ofrecer garantías más fuertes sobre el comportamiento del sistema.

### 3.2 El Rol de JavaScript: Adapter Minimalista

Si Rust gestiona toda la funcionalidad sustancial, ¿cuál es el rol de JavaScript? El rol es **adapter de presentación y eventos**, con responsabilidades muy específicas y delimitadas.

La primera responsabilidad de JavaScript es la **inicialización del motor**: cargar el módulo WASM, configurar el contexto gráfico, y establecer los buffers compartidos para comunicación zero-copy. Esta es una responsabilidad de setup que ocurre una vez al inicio de la aplicación.

La segunda responsabilidad es la **traducción de eventos del DOM**: convertir eventos de mouse, touch y keyboard en llamadas al motor Rust. El motor Rust no tiene acceso directo al DOM del navegador, por lo que JavaScript debe actuar como intermediario, translate eventos nativos a comandos que el motor puede interpretar.

La tercera responsabilidad es el **renderizado final**: consumir los buffers compartidos que el motor Rust produce y realizar los draw calls finales en el canvas. Aquí es importanteClarificar: el motor Rust prepara toda la información de renderizado, pero JavaScript ejecuta los comandos de dibujo en el contexto gráfico. Esta división permite que Rust calcule qué dibujar mientras JavaScript se encarga de cómo dibujarlo en el contexto específico del navegador.

La cuarta responsabilidad es la **integración con frameworks UI**: proporcionar bindings para React, Vue, Angular u otros frameworks que permitan una experiencia de desarrollo familiar. Estos bindings son adaptadores que traducen entre el modelo de programación del framework y el API del SDK.

### 3.3 Modelo Conceptual de Capas

```
┌─────────────────────────────────────────────────────────────────┐
│                    JavaScript Layer (Adapter)                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Event Binding  │  │  Framework      │  │  Canvas         │  │
│  │  (DOM → WASM)   │  │  Integration    │  │  Draw Calls     │  │
│  │                 │  │  (React/Vue)    │  │  (Buffers)      │  │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  │
└───────────┼────────────────────┼────────────────────┼───────────┘
            │                    │                    │
            ▼                    ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Rust WASM Layer (Core SDK)                    │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              archflow-wasm-collab (SharedBuffer)              │ │
│  │              Zero-copy JS ↔ Rust communication                │ │
│  └───────────────────────────┬─────────────────────────────────┘ │
│                              │                                     │
│  ┌───────────────────────────┴─────────────────────────────────┐  │
│  │   archflow-collab         │    archflow-workspace           │  │
│  │   (CRDT + Sync)           │    (Event Sourcing)             │  │
│  │   REQUERIDO PRD 3.4       │    (Undo/Redo)                  │  │
│  └───────────────────────────┴─────────────────────────────────┘  │
│                                                                    │
│  ┌─────────────────────────┬────────────────────────────────────┐  │
│  │   archflow-records      │    archflow-ecs-hybrid            │  │
│  │   (Store, ChangeSet)    │    (Animations + Layers)          │  │
│  │                         │    OPCIONAL via feature flag      │  │
│  └─────────────────────────┴────────────────────────────────────┘  │
│                                                                    │
│  ┌─────────────────────────┬────────────────────────────────────┐  │
│  │   archflow-spatial      │    archflow-geometry               │  │
│  │   (R-Tree Adaptativo)   │    (Kurbo Geometry)                │  │
│  │   Activo si >100 shapes │    (Hit-testing)                   │  │
│  └─────────────────────────┴────────────────────────────────────┘  │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │              archflow-renderers (WebGPU Batch)                │  │
│  │              + archflow-primitives (Drag, Resize, Routing)   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 3.4 Excepciones al Principio de Delegation

Aunque el principio general es delegar toda la funcionalidad a Rust, existen excepciones donde tiene sentido que JavaScript maneje ciertas responsabilidades.

La primera excepción es la **gestión de DOM UI**: menús contextuales, toolbars, paneles de propiedades y otros elementos de interfaz de usuario que viven naturalmente en el DOM. Estos elementos no son parte del canvas y su estado no necesita sincronización con otros usuarios, por lo que pueden gestionarse completamente en JavaScript.

La segunda excepción es la ** internacionalización y localización**: las etiquetas, mensajes de error y textos de UI pueden manejarse en JavaScript utilizando bibliotecas estándar como i18next. El motor Rust puede recibir keys de internacionalización y delegar la resolución a JavaScript.

La tercera excepción es el **caching deAssets**: imágenes, fuentes y otros recursos pueden cachearse en JavaScript para evitar transferências repetidas al motor Rust. El motor puede solicitar recursos por ID y JavaScript proporciona los datos cacheados.

---

## 4. Análisis de Características Específicas: Rust versus JavaScript

### 4.1 Canvas Infinito

#### 4.1.1 Definición y Requisitos

Un canvas infinito es un espacio de trabajo sin límites perceptibles donde los usuarios pueden crear, organizar y manipular contenido arbitrariamente. En el contexto de ArchFlow, el canvas infinito debe soportar navegación fluida mediante pan y zoom, creación de contenido en cualquier ubicación, y rendimiento consistente independientemente de la extensión total del espacio.

#### 4.1.2 Análisis de Ubicación

**Opción A: Implementación completa en JavaScript**

Esta opción implica que JavaScript gestione las coordenadas del viewport, el offset de scroll, el nivel de zoom y qué porción del espacio está visible. El motor Rust simplemente renderiza el contenido que JavaScript le indica.

Los argumentos a favor son la flexibilidad de configuración, la familiaridad con patrones web, y la facilidad de debugging. Los argumentos en contra son la duplicación de lógica de transformación entre capas, la dificultad de mantener consistencia en colaboración multiusuario, y el overhead de convertir coordenadas constantemente.

**Opción B: Implementación completa en Rust**

Esta opción implica que el motor Rust gestione el espacio de coordenadas infinito, las transformaciones de viewport, el culling de contenido fuera de pantalla, y el renderizado optimizado de solo lo visible.

Los argumentos a favor son la consistencia absoluta del estado entre usuarios, la optimización automática del culling sin duplicación de lógica, y la simplificación del código JavaScript que solo necesita pasar eventos de input. Los argumentos en contra son la necesidad de exponer APIs más complejas desde Rust.

#### 4.1.3 Decisión: Implementación en Rust

La decisión es **implementar el canvas infinito completamente en Rust**. Esta decisión se basa en los siguientes razonamientos.

Primero, la colaboración en tiempo real requiere que todos los usuarios tengan la misma vista del canvas. Si cada cliente calcula independientemente qué mostrar, las pequeñas diferencias en float precision o lógica de rounding pueden causar divergencias visuales que dificultan la colaboración.

Segundo, el culling es crítico para el rendimiento con grandes diagramas. El motor Rust conoce la estructura interna de los datos y puede realizar culling inteligente basado en la estructura del documento, no solo en coordenadas de bounding boxes.

Tercero, la navegación entre niveles C4 del modelo requiere transiciones semánticas que el motor Rust puede gestionar como parte del sistema de transformación, asegurando que las animaciones sean suaves y consistentes.

#### 4.1.4 API Propuesta

```rust
// Rust API (expuesta via WASM)
impl Canvas {
    /// Configura el viewport del canvas infinito
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.request_render();
    }
    
    /// Obtiene el viewport actual
    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }
    
    /// Convierte coordenadas de pantalla a coordenadas del canvas
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        (screen - self.viewport.offset) / self.viewport.zoom
    }
    
    /// Convierte coordenadas del canvas a coordenadas de pantalla
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        canvas * self.viewport.zoom + self.viewport.offset
    }
    
    /// Ajusta el zoom para mostrar todo el contenido
    pub fn zoom_to_fit(&mut self) {
        let bounds = self.get_content_bounds();
        // Cálculo de viewport óptimo
        self.set_viewport(calculated_viewport);
    }
    
    /// Ajusta el zoom para mostrar la selección
    pub fn zoom_to_selection(&mut self) {
        let selection_bounds = self.get_selection_bounds();
        // Cálculo de viewport óptimo para selección
        self.set_viewport(calculated_viewport);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub offset: Vec2,    // Posición del origen en pantalla
    pub zoom: f32,       // Nivel de zoom (1.0 = 100%)
    pub min_zoom: f32,
    pub max_zoom: f32,
}
```

### 4.2 Sistemas de Fondos y Grids

#### 4.2.1 Definición y Tipos de Fondos

Los fondos en un editor de diagramas sirven múltiples propósitos: proporcionan contexto espacial, asisten en la alineación, establecen la escala visual, y contribuyen a la estética del documento. ArchFlow debe soportar varios tipos de fondos.

El primer tipo es la **grid de puntos**, una disposición de puntos que indica unidades de espacio sin crear líneas visuales prominentes. El segundo tipo es la **grid de líneas**, una cuadrícula de líneas horizontales y verticales. El tercer tipo es el **graph paper**, una cuadrícula isométrica o triangular para diseños técnicos. El cuarto tipo es el **imagen de fondo**, una imagen o patrón que se repite o stretch para llenar el canvas. El quinto tipo es el **color sólido**, un fondo de color liso.

#### 4.2.2 Análisis de Ubicación

**Argumentos para Rust:**

La grid debe formar parte del sistema de renderizado para beneficiarse del batching y las optimizaciones de WebGPU. Si la grid se renderiza pixel a pixel en JavaScript, cada frame requiere miles de operaciones de dibujo que degradan el rendimiento.

Además, la grid interactúa con el sistema de zoom: cuando el usuario hace zoom out, la grid debe adaptarse para mantener una densidad visual apropiada. Esta lógica de LOD (Level of Detail) es más fácil de implementar en Rust donde se tiene acceso a toda la información de renderizado.

Finalmente, en colaboración, todos los usuarios deben ver la misma grid. Si la grid se generara en JavaScript, las diferencias de rendering podrían causar inconsistencias visuales sutiles.

**Argumentos para JavaScript:**

La grid es puramente decorativa y no participa en la lógica de negocio. Algunos desarrolladores podrían querer personalizarla completamente sin involucrar el motor Rust.

#### 4.2.3 Decisión: Grid en Rust, Personalización en JavaScript

La decisión es **implementar el rendering de grids completamente en Rust**, pero **exponer APIs de configuración desde JavaScript**.

Esta decisión equilibra rendimiento y flexibilidad. El motor Rust renderiza la grid de forma optimizada, pero JavaScript puede especificar qué tipo de grid mostrar, los colores, el espaciado, y otras propiedades de presentación.

#### 4.2.4 API Propuesta

```rust
// Rust - Sistema de fondo interno
impl BackgroundRenderer {
    pub fn render_grid(
        &self,
        viewport: Viewport,
        grid_config: &GridConfig,
        render_ctx: &mut RenderContext,
    ) {
        match grid_config.grid_type {
            GridType::Dots => self.render_dots(viewport, grid_config, render_ctx),
            GridType::Lines => self.render_lines(viewport, grid_config, render_ctx),
            GridType::Isometric => self.render_isometric(viewport, grid_config, render_ctx),
        }
    }
    
    // Optimización: solo renderizar lo visible
    fn render_dots(&self, viewport: Viewport, config: &GridConfig, ctx: &mut RenderContext) {
        let visible_bounds = viewport.visible_bounds();
        let spacing = config.spacing * viewport.zoom;
        
        // Calcular primera posición visible
        let start_x = (visible_bounds.min.x / spacing).floor() * spacing;
        let start_y = (visible_bounds.min.y / spacing).floor() * spacing;
        
        // Batch rendering de todos los puntos visibles
        let mut batch = DrawBatch::new();
        for x in (start_x..visible_bounds.max.x).step_by(spacing) {
            for y in (start_y..visible_bounds.max.y).step_by(spacing) {
                batch.add_point([x, y], config.dot_radius, config.dot_color);
            }
        }
        batch.execute(ctx);
    }
}

pub enum GridType {
    Dots,
    Lines,
    Isometric,
}

pub struct GridConfig {
    pub grid_type: GridType,
    pub spacing: f32,
    pub dot_radius: f32,
    pub dot_color: Color,
    pub line_color: Color,
    pub line_width: f32,
    pub show_grid: bool,
}
```

```typescript
// JavaScript - API de configuración
export interface GridOptions {
    type?: 'dots' | 'lines' | 'isometric';
    spacing?: number;
    dotRadius?: number;
    dotColor?: string;
    lineColor?: string;
    lineWidth?: number;
    visible?: boolean;
}

export class BackgroundManager {
    constructor(private editor: ArchFlowEditor) {}
    
    setGrid(options: GridOptions): void {
        this.editor.setGridConfig({
            grid_type: options.type,
            spacing: options.spacing,
            dot_radius: options.dotRadius,
            dot_color: Color.parse(options.dotColor),
            line_color: Color.parse(options.lineColor),
            line_width: options.lineWidth,
            show_grid: options.visible,
        });
    }
    
    hideGrid(): void {
        this.setGrid({ visible: false });
    }
    
    showDots(spacing: number = 20): void {
        this.setGrid({ type: 'dots', spacing, visible: true });
    }
    
    showLines(spacing: number = 50): void {
        this.setGrid({ type: 'lines', spacing, visible: true });
    }
}
```

### 4.3 Soporte SVG

#### 4.3.1 Análisis del Soporte SVG

SVG (Scalable Vector Graphics) es un formato XML para gráficos vectoriales que tiene ventajas y desventajas en el contexto de un editor de diagramas.

Las ventajas de SVG son la escalabilidad perfecta sin pérdida de calidad, la accesibilidad integrada a través del DOM, la facilidad de inspección y debugging, y el soporte nativo en navegadores web.

Las desventajas de SVG son el rendimiento degradado con miles de elementos debido al overhead del DOM, la dificultad de implementar renderizado por lotes, y las limitaciones en animaciones complejas.

#### 4.3.2 Análisis de Ubicación

**SVG como formato de importación/exportación:**

El soporte SVG debe residir principalmente en Rust para la importación de archivos SVG existentes y la exportación de diagramas a formato SVG. Rust puede parsear XML de forma eficiente y convertir los elementos SVG a la representación interna de ArchFlow.

**SVG como formato de renderizado:**

Renderizar directamente en formato SVG nativo (donde cada elemento es un elemento DOM) no es viable para el rendimiento requerido por ArchFlow. En su lugar, ArchFlow utilizará su propio sistema de renderizado basado en WebGPU que produce gráficos vectoriales visualmente equivalentes pero con rendimiento superior.

**SVG como salida:**

Los usuarios podrán exportar sus diagramas a SVG para uso en documentos, presentaciones o impresión. Esta conversión ocurre en Rust con acceso completo a la información de renderizado.

#### 4.3.3 Decisión: Importación/Exportación en Rust, Renderizado Interno Optimizado

La decisión es **implementar importación y exportación SVG completamente en Rust**, utilizando el motor de renderizado interno optimizado para la visualización en tiempo real.

#### 4.3.4 API Propuesta

```rust
// Rust - Importador/Exportador SVG
impl SvgExporter {
    pub fn export_to_svg(
        &self,
        document: &Document,
        options: &SvgExportOptions,
    ) -> String {
        let mut svg = String::new();
        
        // Header
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            options.width, options.height
        ));
        
        // Styles
        svg.push_str("<style>");
        for (class, style) in &options.styles {
            svg.push_str(&format!(".{} {{ {} }}", class, style));
        }
        svg.push_str("</style>");
        
        // Convertir formas a elementos SVG
        for shape in document.shapes() {
            svg.push_str(&self.shape_to_svg_element(shape));
        }
        
        // Footer
        svg.push_str("</svg>");
        
        svg
    }
    
    fn shape_to_svg_element(&self, shape: &dyn Shape) -> String {
        match shape.shape_type() {
            ShapeType::Rectangle => self.rect_to_svg(shape),
            ShapeType::Ellipse => self.ellipse_to_svg(shape),
            ShapeType::Path => self.path_to_svg(shape),
            // ... otros tipos
        }
    }
}

impl SvgImporter {
    pub fn import_from_svg(&self, svg_content: &str) -> Result<Document, ImportError> {
        let parser = SvgParser::new(svg_content);
        let mut document = Document::new();
        
        for element in parser.elements() {
            let shape = self.convert_element(&element)?;
            document.add_shape(shape);
        }
        
        Ok(document)
    }
}
```

```typescript
// JavaScript - API de importación/exportación
export interface SvgExportOptions {
    width: number;
    height: number;
    includeBackground?: boolean;
    scale?: number;
    styles?: Record<string, string>;
}

export class SvgManager {
    constructor(private editor: ArchFlowEditor) {}
    
    async exportAsSvg(options: SvgExportOptions): Promise<Blob> {
        const svgContent = this.editor.exportToSvg(options);
        return new Blob([svgContent], { type: 'image/svg+xml' });
    }
    
    downloadSvg(filename: string, options: SvgExportOptions): void {
        this.exportAsSvg(options).then(blob => {
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            a.click();
            URL.revokeObjectURL(url);
        });
    }
    
    async importFromSvg(svgFile: File): Promise<void> {
        const content = await svgFile.text();
        this.editor.importFromSvg(content);
    }
    
    async importFromSvgUrl(url: string): Promise<void> {
        const response = await fetch(url);
        const content = await response.text();
        this.editor.importFromSvg(content);
    }
}
```

### 4.4 Sistema de Capas y Niveles C4

#### 4.4.1 Requisitos del PRD

El PRD especifica el requisito de **diagramas con animaciones multicapa estilo C4** en la sección 3.1.1. El modelo C4 organiza la documentación de arquitectura en cuatro niveles: Context (contexto de negocio), Container (aplicaciones y procesos), Component (componentes de código), y Code (clases y funciones detalladas).

#### 4.4.2 Análisis de Ubicación

El sistema de capas C4 requiere coordinación estrecha entre varios subsistemas del motor Rust. El sistema de transformación debe gestionar transiciones suaves entre niveles de zoom, el sistema de renderizado debe mostrar/ocultar elementos apropiados según el nivel actual, y el sistema de estado debe mantener la jerarquía de elementos correctamente.

Esta lógica es inherentemente parte del motor Rust porque involucra invariantes del documento que deben mantenerse consistentes para todos los usuarios. Si JavaScript gestionara las capas, podría fácilmente causar inconsistencias donde diferentes usuarios ven diferentes conjuntos de elementos.

#### 4.4.3 Decisión: Sistema de Capas Completamente en Rust

La decisión es **implementar el sistema de capas C4 completamente en Rust**, exponiendo únicamente APIs de control y configuración hacia JavaScript.

#### 4.4.4 API Propuesta

```rust
// Rust - Sistema de capas C4
pub enum C4Level {
    Context,      // Nivel 1: Sistema y usuarios
    Container,    // Nivel 2: Aplicaciones
    Component,    // Nivel 3: Componentes de código
    Code,         // Nivel 4: Clases y métodos
}

pub struct Layer {
    pub id: LayerId,
    pub c4_level: C4Level,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub shapes: Vec<ShapeId>,
}

impl Document {
    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        let id = LayerId::new();
        self.layers.insert(id, layer);
        id
    }
    
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.get(&id)
    }
    
    pub fn set_layer_visibility(&mut self, id: LayerId, visible: bool) {
        if let Some(layer) = self.layers.get_mut(&id) {
            layer.visible = visible;
            self.request_render();
        }
    }
    
    pub fn get_shapes_for_level(&self, level: C4Level) -> Vec<&dyn Shape> {
        self.layers
            .values()
            .filter(|l| l.c4_level == level && l.visible)
            .flat_map(|l| l.shapes.iter())
            .filter_map(|id| self.shapes.get(id))
            .collect()
    }
}

// Sistema de animación de transición entre niveles
impl C4TransitionAnimator {
    pub fn animate_to_level(
        &mut self,
        target_level: C4Level,
        duration: Duration,
    ) {
        let start_shapes = self.document.get_shapes_for_level(self.current_level);
        let end_shapes = self.document.get_shapes_for_level(target_level);
        
        // Calcular transformaciones para la animación
        let start_transform = self.calculate_level_transform(self.current_level);
        let end_transform = self.calculate_level_transform(target_level);
        
        self.animation = Animation::new()
            .with_keyframe(0.0, start_transform)
            .with_keyframe(1.0, end_transform)
            .with_duration(duration)
            .with_easing(EaseInOutCubic);
        
        self.current_level = target_level;
    }
}
```

```typescript
// JavaScript - API de control de capas
export type C4Level = 'context' | 'container' | 'component' | 'code';

export interface LayerConfig {
    id: string;
    name: string;
    visible: boolean;
    locked: boolean;
    opacity: number;
}

export class LayerManager {
    constructor(private editor: ArchFlowEditor) {}
    
    getLayers(): LayerConfig[] {
        return this.editor.getAllLayers();
    }
    
    setLayerVisibility(layerId: string, visible: boolean): void {
        this.editor.setLayerVisibility(layerId, visible);
    }
    
    setCurrentLevel(level: C4Level, animate: boolean = true): void {
        if (animate) {
            this.editor.animateToLevel(level, 300); // 300ms
        } else {
            this.editor.setCurrentLevel(level);
        }
    }
    
    zoomToLevel(level: C4Level): void {
        const zoom = this.getZoomForLevel(level);
        const offset = this.getOffsetForLevel(level);
        this.editor.animateViewport({ zoom, offset }, 500);
    }
    
    private getZoomForLevel(level: C4Level): number {
        const zoomLevels = {
            context: 0.15,
            container: 0.5,
            component: 1.0,
            code: 2.0,
        };
        return zoomLevels[level];
    }
}
```

### 4.5 Resumen de Decisiones de Ubicación

| Característica | Ubicación | Justificación |
|----------------|-----------|---------------|
| Canvas infinito | Rust | Consistencia en colaboración, rendimiento de culling |
| Grids y fondos | Rust (rendering), JS (configuración) | Optimización batch, personalización |
| Importación/Exportación SVG | Rust | Parsing XML eficiente, conversión de elementos |
| Renderizado interno | Rust + WebGPU | Diferenciador de rendimiento principal |
| Sistema de capas C4 | Rust | Invariantes de documento, animaciones |
| Transformaciones geométricas | Rust | Precisión, colisiones, colab |
| Event sourcing | Rust | Undo/redo, time-travel debugging |
| Protocolo CRDT | Rust | Colaboración en tiempo real |
| UI de herramientas | JS + DOM | Patrones web familiares |
| Menús contextuales | JS + DOM | Integración con browser |
| i18n | JS | Bibliotecas estándar |
| Cacheo de assets | JS | Evitar transferencias repetidas |

---

## 5. Arquitectura del SDK Final

### 5.1 Propuesta de Arquitectura de Crates

El SDK de ArchFlow se estructura en crates Rust que son compilados a WASM y expuestos a través de un paquete npm JavaScript/TypeScript.

El primer nivel es **archflow-sdk-core**, un meta-crate que re-exporta todos los tipos y traits públicos del SDK. Este crate no contiene implementación, solo declaraciones públicas. Depende de todos los crates de engine relevantes y establece las features que pueden habilitarse opcionalmente.

El segundo nivel es **archflow-sdk-wasm**, un crate dedicado al bridge WASM que gestiona la comunicación entre JavaScript y Rust. Implementa el SharedBuffer para zero-copy, los wrappers de tipos para comunicación cross-language, y los bindings de funciones exportadas.

El tercer nivel es **@archflow/sdk**, el paquete npm que contiene los tipos TypeScript, las definiciones de API, y los helpers de integración con frameworks.

```toml
# archflow-sdk-core/Cargo.toml
[package]
name = "archflow-sdk-core"
version = "0.1.0"
edition = "2021"

[features]
default = [
    "archflow-core",
    "archflow-records", 
    "archflow-collab",
    "archflow-workspace",
]
animations = ["archflow-ecs-hybrid"]
full = ["default", "animations"]

[dependencies]
archflow-core = { path = "../archflow-core", features = ["serde"] }
archflow-records = { path = "../archflow-records" }
archflow-collab = { path = "../archflow-collab" }
archflow-workspace = { path = "../archflow-workspace" }
archflow-ecs-hybrid = { path = "../archflow-ecs-hybrid", optional = true }
```

### 5.2 API TypeScript Pública

El SDK expone una API TypeScript cuidadosamente diseñada que balancea expresividad con simplicidad. La API sigue patrones establecidos en el ecosistema JavaScript mientras preserva el acceso a la funcionalidad del motor Rust.

```typescript
// @archflow/sdk - API principal

export interface EditorOptions {
    canvas: HTMLCanvasElement;
    width?: number;
    height?: number;
    background?: BackgroundOptions;
    grid?: GridOptions;
    c4?: C4Options;
}

export class ArchFlowEditor {
    constructor(options: EditorOptions) {
        this.wasm = initArchFlowWasm();
        this.document = this.wasm.createDocument();
        this.setupEventListeners();
    }
    
    // === Shape Operations ===
    createRectangle(x: number, y: number, width: number, height: number): string {
        return this.wasm.createShape(this.document, {
            type: 'rectangle',
            x, y, width, height
        });
    }
    
    createEllipse(x: number, y: number, radiusX: number, radiusY: number): string {
        return this.wasm.createShape(this.document, {
            type: 'ellipse',
            x: x - radiusX,
            y: y - radiusY,
            width: radiusX * 2,
            height: radiusY * 2
        });
    }
    
    createPath(points: Vec2[]): string {
        return this.wasm.createShape(this.document, {
            type: 'path',
            points: points.flatMap(p => [p.x, p.y])
        });
    }
    
    getShape(id: string): ShapeData | null {
        return this.wasm.getShape(this.document, id);
    }
    
    updateShape(id: string, changes: Partial<ShapeData>): void {
        this.wasm.updateShape(this.document, id, changes);
    }
    
    deleteShape(id: string): void {
        this.wasm.deleteShape(this.document, id);
    }
    
    // === Selection Operations ===
    getSelection(): Selection {
        return this.wasm.getSelection(this.document);
    }
    
    select(id: string): void {
        this.wasm.select(this.document, id);
    }
    
    selectMultiple(ids: string[]): void {
        this.wasm.selectMultiple(this.document, ids);
    }
    
    selectAll(): void {
        this.wasm.selectAll(this.document);
    }
    
    clearSelection(): void {
        this.wasm.clearSelection(this.document);
    }
    
    // === Viewport Operations ===
    getViewport(): Viewport {
        return this.wasm.getViewport(this.document);
    }
    
    setViewport(viewport: Partial<Viewport>): void {
        this.wasm.setViewport(this.document, viewport);
    }
    
    zoomIn(factor?: number): void {
        const vp = this.getViewport();
        this.setViewport({ zoom: vp.zoom * (factor || 1.2) });
    }
    
    zoomOut(factor?: number): void {
        const vp = this.getViewport();
        this.setViewport({ zoom: vp.zoom / (factor || 1.2) });
    }
    
    zoomToFit(): void {
        this.wasm.zoomToFit(this.document);
    }
    
    zoomToSelection(): void {
        this.wasm.zoomToSelection(this.document);
    }
    
    // === Layer Operations ===
    getLayers(): LayerConfig[] {
        return this.wasm.getLayers(this.document);
    }
    
    setLayerVisibility(layerId: string, visible: boolean): void {
        this.wasm.setLayerVisibility(this.document, layerId, visible);
    }
    
    setC4Level(level: C4Level, animate?: boolean): void {
        this.wasm.setC4Level(this.document, level, animate ?? true);
    }
    
    // === Collaboration Operations ===
    async connect(url: string): Promise<void> {
        await this.wasm.connect(this.document, url);
    }
    
    async disconnect(): Promise<void> {
        await this.wasm.disconnect(this.document);
    }
    
    getConnectedUsers(): User[] {
        return this.wasm.getConnectedUsers(this.document);
    }
    
    // === History Operations ===
    undo(): boolean {
        return this.wasm.undo(this.document);
    }
    
    redo(): boolean {
        return this.wasm.redo(this.document);
    }
    
    canUndo(): boolean {
        return this.wasm.canUndo(this.document);
    }
    
    canRedo(): boolean {
        return this.wasm.canRedo(this.document);
    }
    
    // === Event System ===
    on<K extends EventType>(event: K, callback: EditorEventMap[K]): Unsubscribe {
        return this.eventEmitter.on(event, callback);
    }
    
    off<K extends EventType>(event: K, callback?: EditorEventMap[K]): void {
        this.eventEmitter.off(event, callback);
    }
    
    // === Render Control ===
    render(): void {
        this.wasm.render(this.document);
    }
    
    destroy(): void {
        this.wasm.destroyDocument(this.document);
        this.removeEventListeners();
    }
    
    private wasm: ArchFlowWasm;
    private document: DocumentHandle;
    private eventEmitter = new EventEmitter<EditorEventMap>();
}
```

### 5.3 Integración con React

El SDK proporciona componentes y hooks de React para facilitar la integración con aplicaciones modernas.

```typescript
// @archflow/sdk/react

import React, { useEffect, useRef, useState, useCallback } from 'react';
import { ArchFlowEditor, EditorOptions, ShapeData } from '@archflow/sdk';

interface ArchFlowCanvasProps extends Omit<EditorOptions, 'canvas'> {
    width?: number | string;
    height?: number | string;
    onShapeCreate?: (shape: ShapeData) => void;
    onSelectionChange?: (selection: string[]) => void;
    onViewportChange?: (viewport: Viewport) => void;
}

export const ArchFlowCanvas: React.FC<ArchFlowCanvasProps> = ({
    width = '100%',
    height = '100%',
    background,
    grid,
    c4,
    onShapeCreate,
    onSelectionChange,
    onViewportChange,
    children,
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const [editor, setEditor] = useState<ArchFlowEditor | null>(null);
    
    // Inicializar editor
    useEffect(() => {
        if (!canvasRef.current) return;
        
        const newEditor = new ArchFlowEditor({
            canvas: canvasRef.current,
            width: typeof width === 'number' ? width : undefined,
            height: typeof height === 'number' ? height : undefined,
            background,
            grid,
            c4,
        });
        
        // Suscribir a eventos
        if (onShapeCreate) {
            newEditor.on('shapecreate', onShapeCreate);
        }
        if (onSelectionChange) {
            newEditor.on('selectionchange', (sel) => onSelectionChange(sel.shapes));
        }
        if (onViewportChange) {
            newEditor.on('viewportchange', onViewportChange);
        }
        
        setEditor(newEditor);
        
        return () => {
            newEditor.destroy();
        };
    }, []);
    
    // Callback para crear rectángulo (ejemplo de API)
    const createRectangle = useCallback((x: number, y: number, w: number, h: number) => {
        editor?.createRectangle(x, y, w, h);
    }, [editor]);
    
    // Exponer editor a través de contexto o ref si es necesario
    return (
        <div className="archflow-canvas-container" style={{ width, height }}>
            <canvas ref={canvasRef} style={{ width: '100%', height: '100%' }} />
            {/* Controles UI adicionales */}
            {children}
        </div>
    );
};

// Hook useArchFlowEditor
export function useArchFlowEditor(canvas: HTMLCanvasElement | null, options?: EditorOptions) {
    const [editor, setEditor] = useState<ArchFlowEditor | null>(null);
    
    useEffect(() => {
        if (!canvas) return;
        
        const newEditor = new ArchFlowEditor({ canvas, ...options });
        setEditor(newEditor);
        
        return () => {
            newEditor.destroy();
        };
    }, [canvas]);
    
    return editor;
}

// Hook useSelection
export function useSelection(editor: ArchFlowEditor | null) {
    const [selection, setSelection] = useState<string[]>([]);
    
    useEffect(() => {
        if (!editor) return;
        
        const unsubscribe = editor.on('selectionchange', (sel) => {
            setSelection(sel.shapes);
        });
        
        return unsubscribe;
    }, [editor]);
    
    return selection;
}

// Hook useViewport
export function useViewport(editor: ArchFlowEditor | null) {
    const [viewport, setViewport] = useState<Viewport | null>(null);
    
    useEffect(() => {
        if (!editor) return;
        
        const updateViewport = () => {
            setViewport(editor.getViewport());
        };
        
        const unsubscribe = editor.on('viewportchange', updateViewport);
        updateViewport(); // Estado inicial
        
        return unsubscribe;
    }, [editor]);
    
    return viewport;
}
```

### 5.4 Sistema de Plugins

El SDK expone un sistema de plugins que permite extender la funcionalidad sin modificar el código core.

```rust
// Rust - Sistema de plugins
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> Version;
    fn init(&mut self, editor: &mut dyn Editor) -> Result<(), PluginError>;
    fn shutdown(&self);
    
    // Hooks opcionales
    fn on_shape_create(&self, _editor: &dyn Editor, _shape: &dyn Shape) {}
    fn on_shape_update(&self, _editor: &dyn Editor, _shape: &dyn Shape, _changes: &UpdateParams) {}
    fn on_selection_change(&self, _editor: &dyn Editor, _added: &[EntityId], _removed: &[EntityId]) {}
    fn on_render(&self, _editor: &dyn Editor, _render_ctx: &mut RenderContext) {}
}

pub struct PluginManager {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
    editor: *mut dyn Editor,
}

impl PluginManager {
    pub fn install<P: Plugin + 'static>(&mut self, plugin: P) -> Result<PluginId, PluginError> {
        let mut boxed = Box::new(plugin);
        boxed.init(unsafe { &mut *self.editor })?;
        
        let id = PluginId::new();
        self.plugins.insert(id, boxed);
        Ok(id)
    }
    
    pub fn uninstall(&mut self, id: PluginId) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.remove(&id) {
            plugin.shutdown();
            Ok(())
        } else {
            Err(PluginError::NotFound)
        }
    }
}
```

```typescript
// JavaScript - API de plugins
export interface Plugin {
    name: string;
    version: string;
    init(editor: ArchFlowEditor): void;
    destroy(): void;
}

export class PluginManager {
    private plugins: Map<string, Plugin> = new Map();
    
    constructor(private editor: ArchFlowEditor) {}
    
    register(name: string, pluginFactory: () => Plugin): void {
        if (this.plugins.has(name)) {
            console.warn(`Plugin ${name} already registered`);
            return;
        }
        
        const plugin = pluginFactory();
        plugin.init(this.editor);
        this.plugins.set(name, plugin);
    }
    
    unregister(name: string): void {
        const plugin = this.plugins.get(name);
        if (plugin) {
            plugin.destroy();
            this.plugins.delete(name);
        }
    }
    
    getPlugin(name: string): Plugin | undefined {
        return this.plugins.get(name);
    }
    
    listPlugins(): string[] {
        return Array.from(this.plugins.keys());
    }
}

// Ejemplo de plugin de validación de arquitectura
const architectureValidatorPlugin: Plugin = {
    name: 'architecture-validator',
    version: '1.0.0',
    init(editor) {
        editor.on('shapecreate', (shape) => {
            if (shape.type === 'database' && !hasConnection(shape)) {
                console.warn('Database without connection detected');
                // Mostrar warning al usuario
            }
        });
    },
    destroy() {
        // Cleanup
    },
};
```

---

## 6. Estrategia de Renderizado

### 6.1 Arquitectura de Renderizado Híbrida

El sistema de renderizado de ArchFlow utiliza un enfoque híbrido que combina WebGPU para el contenido del canvas con Canvas 2D para elementos de UI y debugging. Esta decisión maximiza el rendimiento para el contenido principal mientras mantiene flexibilidad para características específicas.

El primer nivel es **WebGPU Batch Rendering**, utilizado para shapes, líneas, conexiones, grids y todo el contenido del diagrama. Este nivel aprovecha el batching para minimizar draw calls y el instancing para renderizar miles de elementos idénticos eficientemente.

El segundo nivel es **Canvas 2D Overlay**, utilizado para cursores de colaboración en tiempo real, feedback de dragging, handles de selección, y overlays de debugging. Estos elementos cambian frecuentemente y requieren integración nativa con el DOM.

```rust
// Rust - Sistema de renderizado batch
pub struct BatchRenderer2D {
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: RenderPipeline,
    vertex_buffer: DynamicVertexBuffer,
    instance_buffer: DynamicInstanceBuffer,
    
    // Batches organizados por material
    batches: HashMap<MaterialId, Vec<BatchItem>>,
}

impl BatchRenderer2D {
    pub fn begin_frame(&mut self, viewport: Viewport) {
        self.batches.clear();
        self.viewport = viewport;
    }
    
    pub fn add_shape(&mut self, shape: &dyn Renderable, transform: Mat3) {
        let material = shape.material();
        let batch = self.batches.entry(material).or_default();
        
        // Extraer datos para batching
        let (vertices, indices) = shape.geometry();
        let instances = shape.instances(transform);
        
        batch.push(BatchItem {
            vertices,
            indices,
            instances,
            shader_params: shape.shader_params(),
        });
    }
    
    pub fn end_frame(&mut self, render_ctx: &mut RenderContext) {
        // Ejecutar todos los batches
        for (material, items) in self.batches.drain() {
            self.render_batch(material, &items, render_ctx);
        }
    }
    
    fn render_batch(
        &self,
        material: MaterialId,
        items: &[BatchItem],
        render_ctx: &mut RenderContext,
    ) {
        // Configurar pipeline para el material
        render_ctx.set_pipeline(&self.pipelines[material]);
        
        // Upload de geometry data (compartido entre todos los items)
        let geometry_buffer = self.upload_geometry(items);
        
        // Upload de instance data (único por item)
        for item in items {
            let instance_buffer = self.upload_instances(item);
            render_ctx.draw_indexed(
                geometry_buffer,
                instance_buffer,
                item.vertex_count,
                item.instance_count,
            );
        }
    }
}
```

### 6.2 Zero-Copy Buffer Sharing

Para maximizar el rendimiento de la comunicación entre Rust y JavaScript, el SDK utiliza SharedArrayBuffer para transferir datos de renderizado sin copiar.

```rust
// Rust - SharedBuffer para zero-copy
pub struct SharedBuffer {
    buffer: NonNull<u8>,
    size: usize,
    file_descriptor: FileDescriptor,
}

impl SharedBuffer {
    pub fn new(size: usize) -> Result<Self, SharedBufferError> {
        // Crear shared buffer con protección de lectura/escritura
        let buffer = mmap(
            None,
            size,
            PROT_READ | PROT_WRITE,
            MAP_SHARED | MAP_ANONYMOUS,
            -1,
            0,
        )?;
        
        Ok(Self {
            buffer: NonNull::dangling(),
            size,
        })
    }
    
    /// Escribe datos de renderizado al buffer
    pub fn write_render_data(&mut self, data: &RenderData) -> usize {
        let encoded = postcard::to_vec(data).unwrap();
        let offset = self.write_header(FrameHeader {
            data_size: encoded.len(),
            timestamp: Instant::now(),
        });
        
        unsafe {
            std::ptr::copy(encoded.as_ptr(), self.buffer.as_ptr().add(offset), encoded.len());
        }
        
        offset + encoded.len()
    }
    
    /// Obtiene el file descriptor para compartir con JavaScript
    pub fn as_js_transferable(&self) -> JSValue {
        // Wasm-bindgen helpers para transferir el buffer
        js_sys::Reflect::get(
            &self.memory_buffer,
            &JsValue::from_str("buffer"),
        ).unwrap()
    }
}

// Estructura de datos compartida
#[derive(Serialize, Deserialize)]
pub struct RenderData {
    pub viewport: Viewport,
    pub shapes: Vec<RenderableShape>,
    pub cursors: Vec<RemoteCursor>,
    pub selection: SelectionHighlight,
    pub grid: Option<GridRenderData>,
    pub frame_number: u64,
}
```

```typescript
// JavaScript - Consumo del shared buffer
class RenderAdapter {
    private sharedBuffer: SharedArrayBuffer;
    private sharedMemory: Uint8Array;
    
    constructor(wasmModule: ArchFlowWasm) {
        // Obtener SharedArrayBuffer del módulo WASM
        this.sharedBuffer = wasmModule.getSharedBuffer();
        this.sharedMemory = new Uint8Array(this.sharedBuffer);
    }
    
    render(): void {
        // Leer header del buffer
        const header = this.readHeader();
        
        if (header.data_size === 0) {
            // No hay datos nuevos, usar frame anterior
            return;
        }
        
        // Parsear datos de renderizado
        const data = this.parseRenderData(header.offset);
        
        // Ejecutar WebGPU draw calls
        this.executeRenderCommands(data);
    }
    
    private executeRenderCommands(data: RenderData): void {
        // Batch rendering de shapes
        for (const shape of data.shapes) {
            this.webgpuRenderer.drawShape(shape);
        }
        
        // Renderizar cursores remotos (Canvas 2D overlay)
        for (const cursor of data.cursors) {
            this.canvas2dOverlay.drawCursor(cursor);
        }
        
        // Renderizar selección
        if (data.selection) {
            this.canvas2dOverlay.drawSelection(data.selection);
        }
        
        // Renderizar grid si está visible
        if (data.grid) {
            this.webgpuRenderer.drawGrid(data.grid);
        }
    }
}
```

---

## 7. Colaboración en Tiempo Real

### 7.1 Arquitectura CRDT

El sistema de colaboración de ArchFlow utiliza CRDTs (Conflict-free Replicated Data Types) para garantizar consistencia eventual entre todos los clientes sin requerir un coordinador centralizado para resolver conflictos.

```rust
// Rust - Sistema CRDT básico
pub struct CRDT<T: Clone> {
    site_id: SiteId,
    vector_clock: VectorClock,
    state: HashMap<EntityId, Versioned<T>>,
    pending_changes: Vec<Change<T>>,
}

impl<T: Clone> CRDT<T> {
    pub fn new(site_id: SiteId) -> Self {
        Self {
            site_id,
            vector_clock: VectorClock::new(),
            state: HashMap::new(),
            pending_changes: Vec::new(),
        }
    }
    
    pub fn apply(&mut self, change: Change<T>) -> Result<(), ConflictError> {
        // Verificar causalidad
        if !self.vector_clock.causally_before(&change.vector_clock) {
            return Err(ConflictError::CausalViolation);
        }
        
        // Verificar versión para Last-Writer-Wins
        if let Some(existing) = self.state.get(&change.entity_id) {
            if existing.version > change.version {
                return Err(ConflictError::VersionConflict);
            }
        }
        
        // Aplicar cambio
        self.state.insert(
            change.entity_id,
            Versioned {
                value: change.value,
                version: change.version,
                origin: change.site_id,
            },
        );
        
        self.vector_clock.merge(&change.vector_clock);
        Ok(())
    }
    
    pub fn get_pending_changes(&mut self) -> Vec<Change<T>> {
        self.pending_changes.drain(..).collect()
    }
    
    pub fn merge(&mut self, other: &CRDT<T>) -> Result<(), ConflictError> {
        for change in other.get_all_changes() {
            self.apply(change.clone())?;
        }
        Ok(())
    }
}

pub struct Change<T> {
    pub entity_id: EntityId,
    pub value: T,
    pub version: u64,
    pub vector_clock: VectorClock,
    pub site_id: SiteId,
    pub timestamp: Timestamp,
}

pub struct VectorClock {
    clocks: HashMap<SiteId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { clocks: HashMap::new() }
    }
    
    pub fn increment(&mut self, site_id: SiteId) {
        *self.clocks.entry(site_id).or_insert(0) += 1;
    }
    
    pub fn merge(&mut self, other: &VectorClock) {
        for (site, &time) in &other.clocks {
            let current = self.clocks.entry(site).or_insert(0);
            *current = (*current).max(time);
        }
    }
    
    pub fn causally_before(&self, other: &VectorClock) -> bool {
        for (site, &self_time) in &self.clocks {
            if let Some(&other_time) = other.clocks.get(site) {
                if self_time > other_time {
                    return false;
                }
            }
        }
        true
    }
}
```

### 7.2 Sincronización de Cursors y Selección

```rust
// Rust - Sincronización de presencia
pub struct PresenceManager {
    local_site: SiteId,
    remote_sites: HashMap<SiteId, RemotePresence>,
    broadcast_channel: BroadcastChannel,
}

impl PresenceManager {
    pub fn update_cursor(&mut self, position: Vec2) {
        self.local_cursor = Some(CursorState {
            position,
            timestamp: Instant::now(),
        });
        
        // Broadcast a otros clientes
        self.broadcast_channel.send(P presenceUpdate {
            site_id: self.local_site,
            cursor: self.local_cursor,
        });
    }
    
    pub fn update_selection(&mut self, selection: Vec<EntityId>) {
        self.local_selection = selection.clone();
        
        self.broadcast_channel.send(SelectionUpdate {
            site_id: self.local_site,
            selection,
        });
    }
    
    pub fn apply_remote_presence(&mut self, update: PresenceUpdate) {
        let presence = self.remote_sites.entry(update.site_id).or_default();
        presence.cursor = update.cursor;
        presence.last_seen = Instant::now();
    }
}
```

```typescript
// JavaScript - API de colaboración
export interface CollaborationOptions {
    roomId: string;
    userId: string;
    userName: string;
    userColor?: string;
}

export class CollaborationManager {
    private ws: WebSocket | null = null;
    private presenceManager: PresenceManager;
    private cursors: Map<string, CursorState> = new Map();
    
    constructor(
        private editor: ArchFlowEditor,
        private options: CollaborationOptions,
    ) {}
    
    async connect(): Promise<void> {
        this.ws = new WebSocket(this.getServerUrl());
        
        this.ws.onmessage = (event) => {
            const message = JSON.parse(event.data);
            this.handleMessage(message);
        };
        
        // Enviar join
        this.send({
            type: 'join',
            roomId: this.options.roomId,
            userId: this.options.userId,
            userName: this.options.userName,
            userColor: this.options.userColor,
        });
    }
    
    private handleMessage(message: ServerMessage): void {
        switch (message.type) {
            case 'presence':
                this.updateRemoteCursor(message);
                break;
            case 'selection':
                this.updateRemoteSelection(message);
                break;
            case 'change':
                this.applyRemoteChange(message);
                break;
            case 'user_joined':
                this.onUserJoined(message);
                break;
            case 'user_left':
                this.onUserLeft(message);
                break;
        }
    }
    
    private updateRemoteCursor(message: PresenceMessage): void {
        this.cursors.set(message.userId, {
            x: message.x,
            y: message.y,
            name: message.userName,
            color: message.userColor,
        });
        
        // Invalidar área de cursor para redibujar
        this.editor.invalidateCursorArea(message.userId);
    }
    
    private updateRemoteSelection(message: SelectionMessage): void {
        // Notificar al editor para mostrar selección remota
        this.editor.setRemoteSelection(message.userId, message.selection);
    }
    
    private applyRemoteChange(message: ChangeMessage): void {
        this.editor.applyChange(message.change);
    }
    
    private onUserJoined(message: UserEventMessage): void {
        this.editor.showNotification(`${message.userName} joined`);
    }
    
    private onUserLeft(message: UserEventMessage): void {
        this.cursors.delete(message.userId);
        this.editor.clearRemoteSelection(message.userId);
        this.editor.showNotification(`${message.userName} left`);
    }
    
    // === API pública ===
    
    getConnectedUsers(): ConnectedUser[] {
        return Array.from(this.cursors.keys()).map(userId => ({
            userId,
            name: this.cursors.get(userId)?.name,
            cursor: this.cursors.get(userId),
        }));
    }
    
    async disconnect(): Promise<void> {
        if (this.ws) {
            this.send({ type: 'leave', roomId: this.options.roomId, userId: this.options.userId });
            this.ws.close();
            this.ws = null;
        }
    }
}
```

---

## 8. Gestión de Estado y Undo/Redo

### 8.1 Event Sourcing

El sistema de estado de ArchFlow utiliza event sourcing, donde cada cambio al documento se registra como un evento inmutable en lugar de modificar directamente el estado.

```rust
// Rust - Sistema de eventos
pub struct EventSourcingStore {
    events: Vec<DomainEvent>,
    snapshot: Option<DocumentSnapshot>,
    event_handlers: HashMap<EventType, Vec<EventHandler>>,
}

impl EventSourcingStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshot: None,
            event_handlers: HashMap::new(),
        }
    }
    
    pub fn apply_event(&mut self, event: DomainEvent) -> Result<(), ApplyError> {
        // Validar evento
        self.validate_event(&event)?;
        
        // Aplicar al estado
        self.apply_to_state(&event)?;
        
        // Registrar evento
        self.events.push(event.clone());
        
        // Notificar handlers
        self.notify_handlers(&event);
        
        // Crear snapshot periódicamente
        if self.events.len() % SNAPSHOT_INTERVAL == 0 {
            self.create_snapshot();
        }
        
        Ok(())
    }
    
    fn apply_to_state(&mut self, event: &DomainEvent) -> Result<(), ApplyError> {
        match event {
            DomainEvent::ShapeCreated(id, data) => {
                self.state.shapes.insert(*id, data.clone());
            }
            DomainEvent::ShapeUpdated(id, changes) => {
                if let Some(shape) = self.state.shapes.get_mut(id) {
                    shape.apply_changes(changes);
                }
            }
            DomainEvent::ShapeDeleted(id) => {
                self.state.shapes.remove(id);
            }
            DomainEvent::SelectionChanged(selection) => {
                self.state.selection = selection.clone();
            }
            // ... otros eventos
        }
        Ok(())
    }
    
    pub fn undo(&mut self) -> Result<(), UndoError> {
        // Encontrar último evento reversible
        let event_index = self.events.iter()
            .rposition(|e| e.is_reversible())
            .ok_or(UndoError::NoEventsToUndo)?;
        
        let event = self.events[event_index].clone();
        
        // Aplicar evento inverso
        let inverse = event.create_inverse();
        self.apply_event(inverse)?;
        
        // Marcar como deshecho
        self.events[event_index].mark_undone();
        
        Ok(())
    }
    
    pub fn redo(&mut self) -> Result<(), RedoError> {
        // Encontrar último evento deshecho
        let event_index = self.events.iter()
            .rposition(|e| e.is_undone())
            .ok_or(RedoError::NoEventsToRedo)?;
        
        let event = self.events[event_index].clone();
        
        // Re-aplicar evento original
        let original = event.create_original();
        self.apply_event(original)?;
        
        // Desmarcar como deshecho
        self.events[event_index].mark_redone();
        
        Ok(())
    }
    
    pub fn get_history(&self) -> Vec<HistoryEntry> {
        self.events.iter()
            .filter(|e| !e.is_draft())
            .enumerate()
            .map(|(index, event)| HistoryEntry {
                index,
                event_type: event.event_type(),
                description: event.description(),
                timestamp: event.timestamp(),
                is_undone: event.is_undone(),
            })
            .collect()
    }
}

pub enum DomainEvent {
    ShapeCreated(EntityId, ShapeData),
    ShapeUpdated(EntityId, ShapeChanges),
    ShapeDeleted(EntityId),
    SelectionChanged(Vec<EntityId>),
    ViewportChanged(Viewport),
    LayerVisibilityChanged(LayerId, bool),
    // ... otros eventos
}
```

```typescript
// JavaScript - API de historia
export class HistoryManager {
    constructor(private editor: ArchFlowEditor) {}
    
    undo(): boolean {
        return this.editor.undo();
    }
    
    redo(): boolean {
        return this.editor.redo();
    }
    
    canUndo(): boolean {
        return this.editor.canUndo();
    }
    
    canRedo(): boolean {
        return this.editor.canRedo();
    }
    
    getHistory(): HistoryEntry[] {
        return this.editor.getHistory();
    }
    
    goToState(stateId: string): void {
        this.editor.jumpToState(stateId);
    }
    
    clearHistory(): void {
        this.editor.clearHistory();
    }
    
    // Suscribirse a cambios de historia
    onUndoStateChange(callback: (canUndo: boolean, canRedo: boolean) => void): Unsubscribe {
        return this.editor.on('historychange', () => {
            callback(this.canUndo(), this.canRedo());
        });
    }
}
```

---

## 9. Rendimiento y Optimización

### 9.1 Benchmarking y Métricas

El SDK establece targets de rendimiento que guían las decisiones de implementación.

| Métrica | Target | Condición |
|---------|--------|-----------|
| Tiempo de inicialización | < 500ms | Carga fría del WASM |
| FPS del canvas | 60 fps | Con 10,000 shapes |
| Latencia de input | < 16ms | Del evento DOM al render |
| Memoria | < 100MB | Con 10,000 shapes |
| Tamaño WASM | < 500KB | Gzipado |
| Tamaño bundle JS | < 50KB | Gzipado |

### 9.2 Estrategias de Optimización

La primera estrategia es **Renderizado por Lotes (Batching)**. En lugar de emitir un draw call por shape, el sistema agrupa shapes por material y tipo, emitiendo un solo draw call por grupo. Esto reduce drásticamente el overhead de la GPU.

La segunda estrategia es **Instancing**. Shapes idénticos (como rectángulos de grid o íconos) se renderizan mediante instancing, donde un solo draw call dibuja miles de instancias con diferentes transformaciones.

La tercera estrategia es **Culling Inteligente**. Antes de renderizar, el sistema determina qué shapes están visibles en el viewport actual y solo procesa esos. Los shapes fuera del viewport no se consideran para renderizado ni para hit-testing.

La cuarta estrategia es **Niveles de Detalle (LOD)**. Cuando el zoom es muy bajo (el documento completo es pequeño en pantalla), los shapes complejos se simplifican a representaciones más simples que requieren menos recursos para renderizar.

La quinta estrategia es **Delta Updates**. En lugar de re-renderizar todo el documento en cada frame, el sistema solo actualiza las áreas que cambiaron. Esto es especialmente importante para colaboración donde los cambios remotos son frecuentes.

```rust
// Rust - Sistema de culling
pub struct ViewportCuller {
    rtree: RTree<ShapeId, AABB>,
    viewport: Viewport,
}

impl ViewportCuller {
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }
    
    pub fn add_shape(&mut self, id: ShapeId, bounds: AABB) {
        self.rtree.insert(id, bounds);
    }
    
    pub fn remove_shape(&mut self, id: ShapeId) {
        self.rtree.remove(&id);
    }
    
    pub fn get_visible_shapes(&self) -> Vec<ShapeId> {
        let viewport_aabb = AABB {
            min: self.viewport.screen_to_canvas(Vec2::ZERO),
            max: self.viewport.screen_to_canvas(Vec2::new(
                self.viewport.width,
                self.viewport.height,
            )),
        };
        
        // Query al R-tree para shapes que intersectan el viewport
        self.rtree.intersection_query(&viewport_aabb)
    }
    
    pub fn get_shapes_in_rect(&self, rect: Rect) -> Vec<ShapeId> {
        let rect_aabb = AABB {
            min: rect.min(),
            max: rect.max(),
        };
        self.rtree.intersection_query(&rect_aabb)
    }
}
```

---

## 10. Roadmap de Implementación

### 10.1 Fase 1: Fundamentos (Semanas 1-4) ✅ COMPLETADA

| Semana | Objetivo | Entregables | Estado |
|--------|----------|-------------|--------|
| 1 | Configuración del workspace SDK | Cargo.toml correcto, WASM build pipeline | ✅ |
| 2 | Bridge WASM básico | SharedBuffer, funciones exportadas | ✅ |
| 3 | Tipos TypeScript públicos | Definiciones completas, tipos compartidos | ✅ |
| 4 | Editor básico con shapes | Rect, Ellipse, Line, Path | ✅ |

**Commits:**
- `accfd19` feat(sdk): add ArchFlow SDK crate with core modules
- `5848e18` feat(sdk): add TypeScript bindings and React integration
- `585be43` chore: add archflow-sdk to workspace

### 10.2 Fase 2: Interacción y Renderizado (Semanas 5-8) 🚧 EN PROGRESO

| Semana | Objetivo | Entregables | Estado |
|--------|----------|-------------|--------|
| 5 | Sistema de selección | Box select, multi-select, hit-testing | ✅ Parcial |
| 6 | Transformaciones | Resize, rotate, handles | ⏳ Pendiente |
| 7 | Grid y fondos | Grid configurable, fondos | ✅ |
| 8 | Viewport y navegación | Pan, zoom, zoom to fit | ✅ |

### 10.3 Fase 3: Historia y Colaboración (Semanas 9-12)

| Semana | Objetivo | Entregables | Estado |
|--------|----------|-------------|--------|
| 9 | Sistema de undo/redo | Event sourcing, history UI | ⏳ Pendiente |
| 10 | CRDT básico | Estado replicado, merge | ⏳ Pendiente |
| 11 | Colaboración WebSocket | Cursors remotos, presencia | ⏳ Pendiente |
| 12 | Sincronización de selección | Selección multiusuario | ⏳ Pendiente |

### 10.4 Fase 4: Capas y Exportación (Semanas 13-16)

| Semana | Objetivo | Entregables | Estado |
|--------|----------|-------------|--------|
| 13 | Sistema de capas C4 | Capas, niveles, transiciones | ✅ |
| 14 | Importación SVG | Parser SVG básico | ⏳ Pendiente |
| 15 | Exportación SVG | Generador SVG, download | ⏳ Pendiente |
| 16 | Optimización | Benchmarks, profiling | ⏳ Pendiente |

### 10.5 Fase 5: Ecosistema (Semanas 17-20)

| Semana | Objetivo | Entregables | Estado |
|--------|----------|-------------|--------|
| 17 | React integration | Componentes, hooks | ✅ |
| 18 | Sistema de plugins | API de plugins, ejemplos | ⏳ Pendiente |
| 19 | Documentación | Docs completos, ejemplos | ⏳ Pendiente |
| 20 | Lanzamiento beta | npm package público | ⏳ Pendiente |

---

## Estado Actual del SDK (v0.12.1)

### ✅ Implementado

**Módulos Rust:**
- `viewport`: Viewport, pan, zoom, zoom_at, zoom_to_fit, screen/canvas conversion
- `canvas`: Canvas infinito, shapes (Rect, Ellipse, Line, Path), selección
- `background`: Grid configurable (dots, lines, isometric), background color
- `layers`: Sistema C4 (Context, Container, Component, Code), LayerManager

**Bindings TypeScript:**
- `ArchFlowEditor`: API pública completa con tipos
- `useSelection`: Hook de React para selección
- `useViewport`: Hook de React para viewport
- `useC4Level`: Hook de React para nivel C4
- `ArchFlowCanvas`: Componente React principal
- `C4LevelSelector`: Selector de nivel C4
- `ViewportControls`: Controles de zoom
- `GridControls`: Controles de grid

### ⏳ Por Implementar

- SVG Import/Export
- Sistema de undo/redo (Event Sourcing)
- Plugin System
- WASM bindings reales
- Colaboración CRDT
- Accesibilidad (Shadow DOM semántico)
- Error bridging (Panics → JavaScript)

---

## 11. Crítica Constructiva y Mejoras de Producción

Esta sección incorpora mejoras propuestas tras una revisión técnica detallada, elevando el SDK de "funcional" a "listo para producción profesional".

### 11.1 Accessibility: Shadow DOM Semántico

Un canvas WebGPU es invisible para lectores de pantalla. Para解决这个问题, el SDK expone la **jerarquía semántica** del documento para que JavaScript genere un árbol DOM accesible paralelo.

```rust
// Rust - Exposición de estructura semántica (mínimo cambio)
impl Document {
    /// Exporta la estructura jerárquica para accesibilidad
    pub fn export_semantic_tree(&self) -> SemanticTree {
        SemanticTree {
            root: self.shapes.values().map(|s| s.to_semantic_node()).collect(),
            viewport: self.viewport.clone(),
            selection: self.selection.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct SemanticNode {
    pub id: EntityId,
    pub type_: String,        // "rectangle", "ellipse", "container", etc.
    pub label: String,        // Etiqueta accesible
    pub role: String,         // ARIA role: "img", "group", "region"
    pub bounds: Rect,
    pub children: Vec<SemanticNode>,
    pub expanded: bool,       // Para containers colapsables
}
```

```typescript
// JavaScript - Generador de DOM accesible paralelo
export class AccessibilityBridge {
    constructor(private editor: ArchFlowEditor) {
        this.container = document.createElement('div');
        this.container.setAttribute('aria-hidden', 'true'); // Oculto visualmente
        this.container.style.cssText = `
            position: absolute;
            pointer-events: none;
            width: 0;
            height: 0;
            overflow: hidden;
        `;
        document.body.appendChild(this.container);
    }
    
    updateSemanticTree(): void {
        const tree = this.editor.exportSemanticTree();
        this.container.innerHTML = this.renderA11yTree(tree);
    }
    
    private renderA11yTree(node: SemanticNode): string {
        return `
            <div role="${node.role}"
                 aria-label="${node.label}"
                 aria-expanded="${node.expanded}"
                 data-archflow-id="${node.id}">
                ${node.children.map(c => this.renderA11yTree(c)).join('')}
            </div>
        `;
    }
}
```

**Justificación**: El motor Rust ya conoce toda la estructura. Solo necesitamos exponerla como JSON serializable. JavaScript genera el DOM accesible sin overhead en el rendering visual.

### 11.2 Error Bridging: Panics → JavaScript

Los panics de Rust son crípticos en la consola del navegador. Implementamos un sistema de **error bridging** robusto.

```rust
// Rust - Panic hook expandido
use std::panic;
use wasm_bindgen::prelude::*;

static mut PANIC_CALLBACK: Option<Box<dyn Fn(String)>> = None;

#[wasm_bindgen]
pub fn set_panic_callback(callback: &JsValue) {
    let callback = callback.dyn_into::<js_sys::Function>()
        .expect("Expected Function");
    
    unsafe {
        PANIC_CALLBACK = Some(Box::new(move |msg: String| {
            let this = JsValue::NULL;
            let args = [JsValue::from(msg)];
            callback.call1(&this, &args[0]);
        }));
    }
}

pub fn init_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        eprintln!("ArchFlow panic: {}", msg);
        
        unsafe {
            if let Some(ref callback) = PANIC_CALLBACK {
                callback(format!(
                    "ArchFlow Error: {}\n\nPlease report this at https://github.com/archflow/archflow/issues",
                    msg
                ));
            }
        }
    }));
}
```

```typescript
// JavaScript - Manejo de errores robusto
export class ErrorManager {
    constructor(private editor: ArchFlowEditor) {
        // Configurar callback de panic
        archflowWasm.set_panic_callback((message: string) => {
            this.handleRustPanic(message);
        });
        
        // Configurar error handler global
        window.addEventListener('error', (e) => {
            if (e.message?.includes('ArchFlow')) {
                this.showCrashScreen(e.message);
            }
        });
    }
    
    private handleRustPanic(message: string): void {
        console.error('[ArchFlow Panic]', message);
        
        // Mostrar crash screen elegante
        this.showCrashScreen(message);
        
        // Opcional: Reporte automático de telemetría
        this.reportTelemetry('panic', { message });
    }
    
    private showCrashScreen(message: string): void {
        const crashScreen = document.createElement('div');
        crashScreen.innerHTML = `
            <div style="
                position: fixed;
                top: 0; left: 0; right: 0; bottom: 0;
                background: rgba(0,0,0,0.9);
                color: white;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                z-index: 999999;
                font-family: system-ui, sans-serif;
                padding: 2rem;
            ">
                <h1>⚠️ ArchFlow encountered an error</h1>
                <p style="max-width: 600px; text-align: center; margin: 1rem 0;">
                    ${message.split('\n')[0]}
                </p>
                <button onclick="location.reload()">Reload Page</button>
            </div>
        `;
        document.body.appendChild(crashScreen);
    }
}
```

### 11.3 Optimización de Reactividad: useSyncExternalStore

React 18+ incluye `useSyncExternalStore` para fuentes de datos externas, eliminando el overhead de `useEffect` + `useState`.

```typescript
// JavaScript - Hook optimizado con useSyncExternalStore
import { useSyncExternalStore } from 'react';

function createArchFlowStore(editor: ArchFlowEditor) {
    let listeners: Set<(selection: string[]) => void> = new Set();
    let currentSelection: string[] = [];
    
    return {
        getSelection() {
            return editor.getSelection().shapes;
        },
        
        subscribe(listener: (selection: string[]) => void) {
            const unsubscribe = editor.on('selectionchange', (sel) => {
                currentSelection = sel.shapes;
                listeners.forEach(l => l(currentSelection));
            });
            listeners.add(listener);
            return () => {
                listeners.delete(listener);
                unsubscribe();
            };
        },
        
        // Snapshot para hydration
        getServerSnapshot() {
            return editor.getSelection().shapes;
        }
    };
}

function useArchFlowSelection(editor: ArchFlowEditor | null) {
    const store = React.useMemo(() => {
        if (!editor) return null;
        return createArchFlowStore(editor);
    }, [editor]);
    
    if (!store) return [];
    
    return useSyncExternalStore(
        store.subscribe.bind(store),
        store.getSelection.bind(store),
        store.getServerSnapshot.bind(store)
    );
}

// Uso - SIN re-renders excesivos
function SelectionToolbar() {
    const editor = useArchFlowEditor(canvasRef.current);
    const selection = useArchFlowSelection(editor);
    
    return (
        <div className="toolbar">
            {selection.length} selected
        </div>
    );
}
```

### 11.4 Suscripciones a Entidades Específicas

Para evitar filtrar en JavaScript, el SDK permite suscribirse a cambios de entidades específicas.

```rust
// Rust - Sistema de suscripciones filtradas (mínimo cambio)
impl EventEmitter {
    pub fn subscribe_entity(
        &mut self,
        entity_id: EntityId,
        callback: EventCallback,
    ) -> SubscriptionId {
        let id = SubscriptionId::new();
        self.entity_subscriptions
            .entry(entity_id)
            .or_default()
            .insert(id, callback);
        id
    }
    
    pub fn emit_shape_update(&self, entity_id: EntityId, changes: &ShapeChanges) {
        // Emitir evento general
        self.emit(EventType::ShapeUpdate, shape_update_event(entity_id, changes));
        
        // Emitir solo a suscriptores de esta entidad
        if let Some(subs) = self.entity_subscriptions.get(&entity_id) {
            for callback in subs.values() {
                callback(shape_update_event(entity_id, changes));
            }
        }
    }
}
```

```typescript
// JavaScript - API filtrada
export class EntitySubscriptionManager {
    constructor(private editor: ArchFlowEditor) {}
    
    /**
     * Suscribirse a cambios de una entidad específica.
     * El filtering ocurre en Rust, reduciendo tráfico WASM.
     */
    onEntityChange(
        entityId: string,
        callback: (change: ShapeChangeEvent) => void
    ): Unsubscribe {
        return this.editor.subscribeToEntity(entityId, callback);
    }
    
    /**
     * Crear watcher temporal para una entidad.
     * Se limpia automáticamente después del timeout.
     */
    watchEntity(
        entityId: string,
        callback: (change: ShapeChangeEvent) => void,
        timeoutMs: number = 5000
    ): void {
        const unsubscribe = this.onEntityChange(entityId, callback);
        
        setTimeout(() => {
            unsubscribe();
        }, timeoutMs);
    }
}
```

### 11.5 Generación Automática de Tipos TypeScript

En lugar de mantener tipos sincronizados manualmente, generamos desde Rust.

```rust
// build_rs - Generación automática de tipos
fn main() -> std::io::Result<()> {
    // Usar ts-rs o quick-protobuf para generar tipos TS
    ts_rs::export![
        EntityId,
        Vec2,
        Rect,
        Color,
        ShapeData,
        Viewport,
        Selection,
        LayerConfig,
    ]
    .with_config(ts_rs::Config::default()
        .space_after_type(true)
        .declare_modules(false))
    .to_folder("sdk-types/")?;
    
    Ok(())
}
```

```toml
# Cargo.toml del SDK
[build-dependencies]
ts-rs = "8"

[package.metadata.ts-rs]
default = ["serde", "wasm-bindgen"]
compatibility = "es2020"
```

```typescript
// Generado automáticamente - sdk-types/ShapeData.ts
/**
 * AUTO-GENERATED by ts-rs - DO NOT EDIT
 */
export interface ShapeData {
  id: string;
  type: "rectangle" | "ellipse" | "line" | "path" | "text" | "image" | "group";
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  fillColor: string;
  strokeColor?: string;
  opacity: number;
}

/**
 * AUTO-GENERATED by ts-rs - DO NOT EDIT
 */
export interface Viewport {
  offset: { x: number; y: number };
  zoom: number;
  minZoom: number;
  maxZoom: number;
}
```

### 11.6 Streaming Instantiation de WASM

Mejorar la experiencia de carga con feedback progresivo.

```typescript
// JavaScript - Carga progresiva con feedback
export async function initArchFlow(options: {
    container: HTMLElement;
    onProgress?: (progress: number) => void;
}): Promise<ArchFlowEditor> {
    const { default: initWasm, ArchFlowWasm } = await import('@archflow/sdk/wasm');
    
    // Mostrar skeleton inicial
    options.container.innerHTML = `
        <div class="archflow-loading">
            <div class="progress-bar" style="width: 0%"></div>
            <span class="status">Initializing engine...</span>
        </div>
    `;
    
    // Inicialización con streaming
    const wasm = await initWasm({
        module: await WebAssembly.compileStreaming(
            fetch('archflow_sdk_bg.wasm')
        ),
        onProgress: (loaded, total) => {
            const percent = Math.round((loaded / total) * 100);
            options.container.querySelector('.progress-bar').style.width = `${percent}%`;
            options.container.querySelector('.status').textContent = 
                `Loading engine... ${percent}%`;
            options.onProgress?.(percent);
        }
    });
    
    // Completado
    options.container.innerHTML = '';
    
    return new ArchFlowEditor({ wasm });
}
```

### 11.7 Matriz de Mejoras Implementadas

| Característica | Propuesta Original | Mejora Implementada | Impacto |
|----------------|-------------------|---------------------|---------|
| **Suscripciones** | `useEffect` + `on('change')` | `useSyncExternalStore` + filtros en Rust | **Rendimiento & DX** |
| **Tipado** | Manual en TS | ts-rs auto-generation | **Mantenibilidad** |
| **Accesibilidad** | No considerada | Shadow DOM semántico | **A11y** |
| **Debugging** | No especificado | Panic callback → Crash Screen | **DX** |
| **Carga WASM** | Básica | Streaming + progress bar | **UX** |
| **Eventos** | BroadCast | Suscripciones por EntityId | **Rendimiento** |

---

## 12. Conclusiones y Recomendaciones

### Síntesis de Decisiones

Este estudio ha establecido las siguientes decisiones arquitectónicas fundamentales para el SDK de ArchFlow:

**Principio de Delegation Completa al Motor Rust.** Toda la lógica de negocio, estado, transformaciones, colisiones, renderizado y colaboración reside en Rust. JavaScript actúa únicamente como adapter de presentación y eventos. Esta decisión elimina problemas de sincronización, maximiza el rendimiento, y facilita la colaboración en tiempo real.

**Sistema de Capas C4 en Rust.** Las transiciones entre niveles de zoom semánticos, la visibilidad de elementos por capa, y las animaciones asociadas se implementan completamente en Rust para garantizar consistencia entre usuarios.

**Grids y Fondos Renderizados en Rust.** El rendering de grids y fondos se realiza en el motor Rust para beneficiarse del batching y las optimizaciones WebGPU, mientras que la configuración (tipo, colores, espaciado) se expone a JavaScript.

**Importación/Exportación SVG en Rust.** El parsing de SVG y la generación de SVG se realizan en Rust para máxima eficiencia, con APIs de alto nivel expuestas a JavaScript.

**SharedBuffer para Zero-Copy.** La comunicación entre Rust y JavaScript utiliza SharedArrayBuffer para transferir datos de renderizado sin copiar, eliminando el overhead de serialización.

**Shadow DOM Semántico para Accesibilidad.** El SDK expone la estructura jerárquica del documento para generar un árbol DOM accesible paralelo, permitiendo que lectores de pantalla naveguen el contenido.

**Event Bridging Robusto.** Los panics de Rust se capturan y propagan a JavaScript para mostrar crash screens informativos en lugar de errores crípticos.

**Generación Automática de Tipos.** El SDK genera automáticamente las interfaces TypeScript desde las estructuras Rust usando ts-rs, garantizando sincronización perfecta.

**Reactividad Optimizada.** El SDK usa `useSyncExternalStore` de React 18+ para evitar re-renders excesivos y expone suscripciones filtradas por entidad para reducir tráfico WASM.

### 11.2 Beneficios de la Arquitectura

Los beneficios de esta arquitectura son múltiples. El primer beneficio es la **consistencia**: el estado vive en un solo lugar (Rust), eliminando bugs de sincronización entre capas.

El segundo beneficio es el **rendimiento**: Rust+WASM+WebGPU proporciona un diferenciador competitivo significativo frente a soluciones JavaScript puro.

El tercer beneficio es la **colaboración**: el sistema CRDT opera sobre el estado en Rust, simplificando la sincronización multiusuario.

El cuarto beneficio es la **experiencia de desarrollador**: TypeScript proporciona tipo-seguridad en el boundary, mientras que Rust garantiza correctitud en el core.

El quinto beneficio es la **extensibilidad**: el sistema de plugins permite extensión sin modificar el código core.

### 11.3 Riesgos y Mitigaciones

El primer riesgo es la **curva de aprendizaje del bridge WASM**. La comunicación entre Rust y JavaScript añade complejidad. La mitigación es invertir en abstracciones de alto nivel que oculten la complejidad.

El segundo riesgo es el **tamaño del bundle WASM**. El código Rust compilado puede ser grande. La mitigación es usar link-time optimization, tree shaking, y feature flags para excluir funcionalidad no usada.

El tercer riesgo es la **compatibilidad de navegadores**. WebGPU no está disponible en todos los navegadores. La mitigación es implementar un fallback a Canvas 2D para navegadores sin WebGPU.

### 11.4 Recomendaciones Finales

Las recomendaciones finales para el equipo de desarrollo son las siguientes.

**Priorizar la calidad del bridge WASM.** El éxito del SDK depende de que la comunicación Rust-JavaScript sea transparente y eficiente. Invertir tiempo en abstractions de alto nivel desde el principio.

**Diseñar APIs centradas en el desarrollador.** No exponer la complejidad de Rust directamente. Crear APIs TypeScript que sean intuitivas y familiares para desarrolladores web.

**Implementar progresivamente.** Comenzar con la funcionalidad core (shapes, selección, renderizado) antes de añadir capas C4 y colaboración. Cada etapa debe ser funcional y testeable.

**Invertir en testing.** El SDK debe tener coverage de tests del 100% en las APIs públicas. Los tests deben ejecutarse tanto en Rust como en JavaScript.

**Documentar extensivamente.** La documentación debe incluir ejemplos de código, guías de integración con frameworks, y troubleshooting guides.

---

## Anexo A: Glosario de Términos

| Término | Definición |
|---------|------------|
| **CRDT** | Conflict-free Replicated Data Type - tipo de dato que puede replicarse en múltiples nodos y sincronizarse sin coordinación centralizada |
| **WASM** | WebAssembly - formato de código binario portable que ejecuta en navegadores con rendimiento cercano a nativo |
| **SharedArrayBuffer** | API de JavaScript que permite compartir memoria entre threads |
| **LOD** | Level of Detail - técnica de optimización que reduce la complejidad de renderizado según la distancia o tamaño |
| **Batching** | Técnica de optimización que agrupa múltiples operaciones similares para reducir overhead |
| **C4 Model** | Modelo de documentación de arquitectura con 4 niveles: Context, Container, Component, Code |
| **Event Sourcing** | Patrón donde el estado se deriva de una secuencia de eventos en lugar de almacenar el estado actual |

---

## Anexo B: Referencias Técnicas

- **tldraw SDK**: https://github.com/tldraw/tldraw
- **Figma Plugin API**: https://www.figma.com/plugin-docs/
- **WebGPU Specification**: https://www.w3.org/TR/webgpu/
- **Yjs CRDT**: https://github.com/yjs/yjs
- **Rust WASM Book**: https://rustwasm.github.io/docs/book/
- **Kurbo Geometry**: https://github.com/linebender/kurbo

---

*Documento creado: 2024*
*Versión: 1.0.0*
*Estado: Final para revisión*
