# ArchFlow

<p align="center">
  <strong>Un Motor de Gráficos 2D Producción-Ready Construido en Rust</strong>
</p>

<p align="center">
  <a href="https://github.com/Rubentxu/archflow/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/Rubentxu/archflow/ci.yml?branch=main" alt="Estado CI">
  </a>
  <a href="https://github.com/Rubentxu/archflow/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/Licencia-MIT-blue.svg" alt="Licencia">
  </a>
  <a href="https://github.com/Rubentxu/archflow">
    <img src="https://img.shields.io/badge/Rust-1.80+-orange.svg" alt="Versión Rust">
  </a>
</p>

<p align="center">
  <em>Motor de gráficos 2D de nivel empresarial con principios de seguridad Zero Trust</em>
</p>

---

## 🎯 Propósito

ArchFlow es un **motor de gráficos 2D producción-ready** diseñado desde cero para construir herramientas profesionales de diagramación, editores basados en nodos y aplicaciones visuales interactivas. Construido completamente en Rust, proporciona una base robusta para aplicaciones que requieren:

- **Diagramación y Flujos de Trabajo**: Crear editores basados en nodos con puertos y conexiones
- **Renderizado de Gráficos Vectoriales**: Renderizado de alta calidad de curvas de Bézier con múltiples backends
- **Aplicaciones de Canvas Interactivo**: Arrastrar, redimensionar, seleccionar y manipular elementos gráficos
- **Despliegue WebAssembly**: Ejecutar aplicaciones gráficas en navegadores con rendimiento nativo

### Principios de Diseño Clave

1. **Seguridad Zero Trust**: Cada componente está diseñado considerando la seguridad, desde validación de entrada hasta valores predeterminados seguros
2. **Producción Ready**: Pruebas exhaustivas, documentación y prácticas de código limpio
3. **Arquitectura Extensible**: Arquitectura hexagonal (Ports & Adapters) para máxima flexibilidad
4. **Seguridad de Tipos**: Aprovechar el sistema de tipos de Rust para prevenir errores en tiempo de compilación

---

## 🚀 Características

### Infraestructura Central

- **Sistema de Tipos Personalizado**: `Vec2`, `Mat3`, `Rect`, `Color`, `EntityId` con soporte completo de serialización
- **Sistema de Entidades y Componentes**: Abstracción limpia para gestionar entidades gráficas
- **Transformaciones**: Operaciones de traslación, rotación y escala con soporte matricial

### Sistema de Primitivas

- **Primitivas de Formas**: Rectángulo, Elipse, Línea, Polilínea con propiedades geométricas completas
- **Sistema de Estilos**: Estilos completos con Soporte de Relleno, Trazo, Texto y Efectos
- **Puertos y Conexiones**: Sistema completo de conectividad para diagramas basados en nodos con enrutamiento inteligente

### Motor de Geometría

- **Curvas de Bézier**: Soporte para curvas de Bézier cuadráticas y cúbicas via kurbo
- **Operaciones de Paths**: Creación, transformación y simplificación de paths (Ramer-Douglas-Peucker)
- **Detección de Intersecciones**: Algoritmo SAT, ray casting y pruebas de hits precisas
- **Indexación Espacial**: Consultas espaciales eficientes para escenas grandes

### Renderizador

- **Trait Renderizador Abstracto**: Interfaz de renderizado agnóstica al backend
- **Backend Canvas 2D**: Renderizado web-compatible via web-sys
- **Renderizador Rough**: Renderizado estilo dibujo a mano para bocetos

---

## 📦 Estructura de Crates

```
crates/
├── archflow-core/           # Tipos centrales y primitivas de dominio
├── archflow-ecs/            # Sistema de Entidades y Componentes
├── archflow-geometry/       # Motor de geometría con kurbo
├── archflow-primitives/     # Formas, estilos, puertos y conexiones
├── archflow-renderer/       # Traits de renderizador abstractos
├── archflow-renderer-canvas/ # Implementación Canvas 2D
├── archflow-renderer-rough/  # Renderizador estilo boceto/dibujado a mano
├── archflow-workspace/      # Gestión de documentos y espacio de trabajo
└── archflow-wasm/           # Bindings WebAssembly
```

---

## 🛠️ Primeros Pasos

### Requisitos Previos

- **Rust**: 1.80 o posterior
- **Cargo**: Última versión estable
- **Git**: Para control de versiones

### Instalación

```bash
# Clonar el repositorio
git clone https://github.com/Rubentxu/archflow.git
cd archflow

# Construir el workspace
cargo build --workspace

# Ejecutar pruebas
cargo test --workspace

# Ejecutar benchmarks
cargo bench --workspace
```

### Ejemplo de Uso

```rust
use archflow_core::{Vec2, EntityId};
use archflow_primitives::{Rectangle, FillStyle, StrokeStyle};
use archflow_geometry::GeometryEngine;

// Crear una primitiva rectángulo
let rect = Rectangle::new(
    Vec2::new(0.0, 0.0),
    Vec2::new(100.0, 50.0),
);

// Aplicar estilos
let styled_rect = rect
    .with_fill(FillStyle::solid("#FF5733"))
    .with_stroke(StrokeStyle::new("#333333", 2.0));

// Usar el motor de geometría para cálculos
let engine = GeometryEngine::default();
let center = engine.rect_center(rect.global_bounds());
let area = rect.local_bounds().area();
```

---

## 📚 Documentación

- **Arquitectura**: Ver `docs/ARCHITECTURE-DESIGN.md`
- **EPICS e Historias de Usuario**: Ver `docs/EPICS-ENGINE-2D.md`
- **Documentación API**: Ejecutar `cargo doc --open` para generar documentación local

---

## 🧪 Pruebas

```bash
# Ejecutar todas las pruebas
cargo test --workspace

# Ejecutar pruebas de un crate específico
cargo test -p archflow-geometry

# Ejecutar pruebas con cobertura
cargo tarpaulin --workspace
```

---

## 📦 Dependencias

ArchFlow utiliza dependencias cuidadosamente seleccionadas:

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `kurbo` | 0.13 | Geometría 2D y curvas de Bézier |
| `glam` | 0.31 | Matemáticas aceleradas por SIMD |
| `serde` | 1.0 | Serialización |
| `uuid` | 1.11 | Identificación de entidades |
| `web-sys` | 0.3 | Bindings DOM para WebAssembly |

---

## 🤝 Contribuciones

¡Las contribuciones son bienvenidas! Por favor lee nuestras guías de contribución antes de enviar PRs.

1. Haz fork del repositorio
2. Crea una rama de característica (`git checkout -b feature/caracteristica-increible`)
3. Confirma tus cambios (`git commit -m 'feat: añadir característica increíble'`)
4. Haz push a la rama (`git push origin feature/caracteristica-increible`)
5. Abre un Pull Request

---

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

---

## 🙏 Reconocimientos

- [kurbo](https://github.com/linebender/kurbo) - Excelente biblioteca de geometría 2D
- [glam](https://github.com/bitshifter/glam-rs) - Biblioteca matemática de alto rendimiento
- [tldraw](https://github.com/tldraw/tldraw) - Inspiración para el diseño de primitivas
- [React Flow](https://github.com/xyflow/react-flow) - Referencia para diagramas basados en nodos

---

<p align="center">
  Construido con ❤️ usando Rust
</p>
