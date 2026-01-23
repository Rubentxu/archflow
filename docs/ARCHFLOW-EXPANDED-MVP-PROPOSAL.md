# ArchFlow: Critica y Propuesta de Expansión (Figma for Solutions Architects)

## 1. Análisis y Crítica del Estado Actual

Tras revisar `EPICS-ARCHFLOW-V2.md` (el motor gráfico actual), `ARCHFLOW-MVP-IMPLEMENTATION.md` (la visión de producto anterior) y el análisis de crates, presento las siguientes conclusiones críticas:

### 1.1 Fortalezas (Lo que está bien)
*   **Motor Gráfico Sólido**: El núcleo (Core + ECS + Renderer) inspirado en tldraw y bevy_ecs es una base excelente. La separación `Store` (datos) vs `Renderer` (vista) es correcta.
*   **Decisiones de Crates**: El uso de `euclid`, `glam` y `rstar` es acertado para un rendimiento alto en geometría.
*   **Historial Delta-based**: Fundamental para una UX tipo Figma (undo/redo robusto).

### 1.2 Debilidades y Brechas (Para la visión "Figma for Architects")
*   **Renderizado (WGPU vs Canvas2D)**: El documento `MVP-IMPLEMENTATION` propone WGPU agresivamente. Sin embargo, la demo actual demuestra que **Canvas 2D (vía WASM) es suficiente y mucho más rápido de iterar** para un MVP. WGPU añade una complejidad inmensa (shaders, pipelines) que puede retrasar el lanzamiento meses sin aportar valor crítico inicial (a menos que tengamos 50k+ nodos).
    *   *Recomendación*: Mantener Canvas 2D optimizado para el MVP. Migrar a WGPU solo si el profiling lo exige en Fase 3.
*   **Modelo de Datos Anémico**: El ECS actual tiene componentes visuales (`Transform`, `Shape`, `Color`) pero carece de **Semántica Arquitectónica**. No hay distinción entre "un rectángulo rojo" y "una instancia EC2".
*   **Falta de Jerarquía (C4)**: El Spatial Index es plano. El modelo C4 requiere anidamiento estricto (System Context -> Container -> Component). El sistema actual no soporta "entrar" en un nodo para ver su detalle interior de forma nativa.
*   **Ausencia del Motor IaC**: La conversión a Terraform/K8s está mencionada como "Export", pero requiere un motor de compilación que valide reglas (ej: "No puedes conectar una Lambda a una Subnet sin VPC").

---

## 2. Propuesta de Expansión: "ArchFlow Architect Edition"

Para transformar el "Editor de Diagramas" actual en un "Figma para Arquitectos de Soluciones", propongo expandir el alcance con los siguientes módulos y conceptos:

### 2.1 Nuevo Modelo de Dominio: "Semantic Layer"

Debemos separar la **Representación Visual** de la **Definición Arquitectónica**.

```rust
// Visual Layer (Ya existente en ECS)
struct VisualComponent {
    transform: Transform, // x, y, rotation
    shape: Shape,         // Rect, Circle
    style: Style,         // Color, Stroke
}

// NUEVO: Logical Layer (Architecture)
struct ArchitectureComponent {
    id: RecordId,
    name: String,
    
    // Semántica C4
    level: C4Level,       // Context, Container, Component
    parent_id: Option<RecordId>, // Jerarquía
    
    // Semántica Cloud
    provider: CloudProvider, // AWS, Azure, K8s, OnPrem
    resource_type: ResourceType, // "aws_instance", "k8s_pod"
    metadata: HashMap<String, Value>, // { "instance_type": "t3.micro", "replicas": 3 }
    
    // Estado Lógico
    status: ResourceStatus, // Active, Failed (para visualización en vivo)
}
```

### 2.2 Funcionalidades Clave Expandidas

#### A. Soporte Nativo C4 (Zoom Semántico)
En lugar de un simple zoom óptico, implementaremos **Semantic Zoom**:
1.  **Nivel Contexto**: Ver cajas grandes ("Sistema de Pagos", "Banco").
2.  **Zoom In**: El nodo "Sistema de Pagos" se vuelve transparente y revela sus Contenedores internos (API, DB, Worker).
3.  **Zoom In Profundo**: Revela Componentes (Clases, tablas, controladores).

#### B. "Smart Flows" (Animaciones de Red)
Reemplazar líneas estáticas con **Flujos Activos**:
*   **Visual**: Animación de partículas sobre curvas de Bézier (`stroke_dash_offset` animation) para indicar dirección del tráfico.
*   **Lógico**: El flujo define protocolo (HTTP/gRPC), puerto (443) y autenticación.
*   **Validación**: Si conectas un componente "Public Internet" a "Private DB" directamente, el la conexión se marca en rojo (violación de regla de seguridad).

#### C. Motor de "Arquitectura Viva" (Live Validation)
Utilizar `petgraph` para correr validaciones en tiempo real (mientras diseñas):
*   *Cycle Detection*: "Cuidado, has creado una dependencia circular".
*   *Security Scan*: "El Security Group permite 0.0.0.0/0 al puerto 22".
*   *Cost Estimation*: Calcular coste aproximado basado en los recursos definidos en el lienzo.

#### D. Generador de IaC (El "Compiler")
No un simple exportador, sino un compilador de grafo:
1.  **Parse**: Convertir Gráfico Visual -> Gráfico Lógico.
2.  **Validate**: Aplicar reglas (ej: política organizacional).
3.  **Synthesize**: Generar HCL (Terraform) o YAML (K8s/Helm).
    *   *Propuesta*: Usar templates `handlebars` o `tera` para que los usuarios puedan personalizar cómo se genera el código de cada nodo (ej: "Mi empresa usa un módulo custom de Terraform para S3").

---

## 3. Hoja de Ruta Actualizada (Expanded MVP)

### Fase 1: Core Engine & Visuals (Completado/En Progreso)
*   ECS, Rendering 2D (Canvas), Spatial Indexing.
*   *Ajuste*: Priorizar "Network Animations" en el renderer (soporte para líneas punteadas animadas).

### Fase 2: Architecture Semantics (NUEVO)
*   **Crate `archflow-model`**: Definir structs para Cloud Resources y C4.
*   **Catalog System**: Registro de tipos (AWS Icons, Azure Icons) con metadatos por defecto.
*   **Property Editor UI**: Panel lateral (tipo Figma) para editar `metadata` del recurso seleccionado (no solo color/tamaño).

### Fase 3: The "Architect" Logic (NUEVO)
*   **Graph Validation**: Integrar `petgraph` para validación de reglas de topología.
*   **IaC Compiler**: Generador básico de Terraform (HCL) para AWS (ej: VPC + EC2 + RDS).
*   **Hierarchical View**: Implementar la lógica de "entrar" en un grupo/contenedor.

---

## 4. Crates Adicionales Necesarios

| Crate | Uso Propuesto |
|-------|---------------|
| `schemars` | Generar JSON Schemas para los metadatos de recursos cloud |
| `tera` / `handlebars` | Templating para generar código IaC (Terraform/Helm) |
| `rust-embed` | Embeber iconos (SVG) y definiciones de recursos en el binario |
| `validator` | Validación de campos en metadatos (ej: validar CIDR blocks) |

---

Este documento redefine el alcance para atacar directamente el nicho de "Solutions Architects", diferenciándonos de herramientas de dibujo genérico como draw.io al "entender" la nube.
