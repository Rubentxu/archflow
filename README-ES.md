# ArchFlow

<p align="center">
  <strong>La Plataforma de Arquitectura Viva</strong>
</p>

<p align="center">
  <em>Diseña, Simula, Despliega y Evoluciona tu Arquitectura Cloud</em>
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

---

## 🎯 ¿Qué es ArchFlow?

ArchFlow es una **Plataforma de Arquitectura Viva** que transforma cómo las organizaciones diseñan, colaboran, simulan y despliegan arquitecturas cloud-native e híbridas. Conectamos la brecha entre herramientas de diseño visual (Figma, draw.io) e infraestructura como código (Terraform, Pulumi) haciendo que el diagrama de arquitectura sea la **única fuente de verdad** que es tanto visual como ejecutable.

### El Problema que Resolvemos

| Desafío | Impacto |
|---------|---------|
| **Deriva de Arquitectura** | Los diagramas se quedan obsoletos en cuanto se crean |
| **Fragmentación de Herramientas** | Los arquitectos usan 5+ herramientas (diagramación, IaC, documentación, colaboración) |
| **Brechas de Implementación** | Los diagramas hermosos no se traducen en infraestructura desplegable |
| **Sorpresas de Costos** | Decisiones de arquitectura tomadas sin visibilidad de costos |

### La Solución ArchFlow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Plataforma ArchFlow                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   🎨 DISEÑO          👥 COLABORAR         🔬 SIMULAR          │
│   Editor Visual      Edición en Tiempo    Costos y Rendimiento │
│   Biblioteca de Comp. Control de Versión  Escenarios de Falla  │
│   Conexiones Smart   Comentarios Context. Análisis de Seguridad │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   🚀 DESPLIEGUE      📊 ANALIZAR          🤖 IA ASISTIDO       │
│   Generación IaC     Informes Cumplim.    Sugerencias Smart    │
│   Sincronización Cloud Optimización Costos  Detección Patrones │
│   Detección Drift    Evolución Histórica  Auto-Documentación   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Capacidades Principales

### 1. Diseño Visual de Arquitectura

La base de ArchFlow es un motor de renderizado de alto rendimiento, impulsado por Rust, que permite:

- **Canvas Infinito** con marcos arquitectónicos anidados
- **Integración C4**: Transiciones fluidas entre Context, Container, Component y Code
- **Biblioteca de Componentes**: AWS, Azure, GCP y componentes empresariales personalizados
- **Conexiones Semánticas**: Puertos y relaciones que mantienen el significado arquitectónico

### 2. Arquitectura como Código (AaC)

Define tu arquitectura en código, expórtalo a cualquier formato IaC:

| Destino de Exportación | Estado | Caso de Uso |
|------------------------|--------|-------------|
| Terraform | ✅ MVP | Aprovisionamiento de infraestructura |
| Kubernetes | ✅ MVP | Orquestación de contenedores |
| Pulumi | 🔜 Fase 2 | IaC multi-lenguaje |
| AWS CDK | 🔜 Fase 2 | Constructs específicos de AWS |
| CloudFormation | 🔜 Fase 2 | Plantillas AWS |
| Crossplane | 🔜 Fase 3 | IaC nativo de Kubernetes |

### 3. Colaboración en Tiempo Real

- **Edición Multi-usuario**: Ve los cambios mientras tu equipo trabaja
- **Flujo Git-Nativo**: Architecture Pull Requests (APRs) con diffs visuales
- **Comentarios Contextuales**: Discute componentes directamente en el diagrama
- **Flujos de Aprobación**: Aprobación visual con auditoría completa

### 4. Simulación y Análisis What-If

Antes de desplegar, valida tu arquitectura:

- **Simulación de Costos**: Estimación de costos en tiempo real con integración Infracost
- **Análisis de Rendimiento**: Modelado de latencia, planificación de throughput, identificación de cuellos de botella
- **Simulación de Falla**: Escenarios de ingeniería del caos, análisis de impacto de dependencias
- **Escaneo de Seguridad**: Análisis de rutas de ataque, detección de brechas de cumplimiento

### 5. Diseño Asistido por IA

Genera, optimiza y documenta tu arquitectura con IA:

- **Generación de Arquitectura**: "Crea una API serverless con auth y base de datos"
- **Sugerencias de Optimización**: "Reduce costos en 40% usando instancias spot"
- **Reconocimiento de Patrones**: Identifica anti-patrones y recomienda mejoras
- **Documentación**: Genera automáticamente Architecture Decision Records (ADRs)

---

## 🛠️ Arquitectura Técnica

### La Base Rust + WebAssembly

ArchFlow está construido sobre un motor Rust de alto rendimiento que compila a WebAssembly:

```
┌────────────────────────────────────────────────────────────┐
│                    Frontend (Navegador)                     │
├────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Motor Core Rust/WASM                          │  │
│  │  ┌─────────┐ ┌──────────┐ ┌─────────────────────┐   │  │
│  │  │Gráficos │ │Motor de  │ │Motor de             │   │  │
│  │  │(WebGPU) │ │Geometría │ │Colaboración         │   │  │
│  │  └─────────┘ └──────────┘ └─────────────────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│           ↑                    ↑                           │
│     archflow-geometry     archflow-primitives              │
│           ↑                                              │
│     archflow-core                                        │
└────────────────────────────────────────────────────────────┘
              ↓ (opcional)
┌────────────────────────────────────────────────────────────┐
│                  Servicios Backend                          │
│  Servicio Sync │ Motor IA │ Servicio Agente │ Almacenamiento│
└────────────────────────────────────────────────────────────┘
```

### Implementación Actual (Capa de Fundación)

Nuestra implementación actual proporciona la **base fundamental** para la plataforma completa:

| Crate | Propósito | Estado |
|-------|-----------|--------|
| `archflow-core` | Tipos centrales (Vec2, Mat3, Rect, Color, EntityId) | ✅ Completo |
| `archflow-geometry` | Motor de geometría con kurbo, curvas de Bézier, detección de intersecciones | ✅ Completo |
| `archflow-primitives` | Formas, estilos, puertos y conexiones | ✅ Completo |
| `archflow-renderer` | Traits de renderizador abstractos | ✅ Completo |
| `archflow-renderer-canvas` | Backend Canvas 2D | ✅ Completo |
| `archflow-ecs` | Sistema de Entidades y Componentes | ✅ Completo |
| `archflow-workspace` | Gestión de documentos y espacio de trabajo | ✅ Completo |
| `archflow-wasm` | Bindings WebAssembly | ✅ Completo |

### Objetivos de Rendimiento

| Métrica | Objetivo | Implementación |
|---------|----------|----------------|
| Carga 10k nodos | <2s | Rust/WASM |
| 60fps pan/zoom | 1k elementos animados | WebGPU |
| Latencia colaboración | <100ms | Sincronización CRDT |

---

## 📦 Primeros Pasos

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
```

### Ejemplo Rápido

```rust
use archflow_core::{Vec2, EntityId};
use archflow_primitives::{Rectangle, FillStyle, StrokeStyle, Port};
use archflow_geometry::GeometryEngine;

// Crear un componente cloud con puertos
let component = Rectangle::new(
    Vec2::new(0.0, 0.0),
    Vec2::new(200.0, 150.0),
).with_fill(FillStyle::solid("#FF5733"))
 .with_stroke(StrokeStyle::new("#333333", 2.0))
 .with_port(Port::output("api", Vec2::new(200.0, 75.0)))
 .with_port(Port::input("data", Vec2::new(0.0, 75.0)));

// Calcular propiedades geométricas
let engine = GeometryEngine::default();
let bounds = component.global_bounds();
let center = engine.rect_center(bounds);

// Exportar a Infraestructura como Código
let terraform = component.to_terraform();
let kubernetes = component.to_kubernetes();
```

---

## 🗺️ Hoja de Ruta

```
Fase 1: Fundación (Meses 1-6) - ESTÁS AQUÍ
├── ✅ Motor Core (Geometría, Renderizado, Primitivas)
├── 🔄 Diseño del Sistema de Componentes
└── ⏳ Especificación AUF (Architecture Universal Format)

Fase 2: MVP (Meses 7-12)
├── Editor Visual con arrastrar y soltar
├── Exportación Terraform & Kubernetes
├── Simulación de Costos Básica
└── Biblioteca de Componentes (AWS, Azure, GCP)

Fase 3: Colaboración (Meses 13-18)
├── Edición multi-usuario en tiempo real
├── Integración Git (commit/pull/push)
├── Architecture Pull Requests (APRs)
└── Flujos de trabajo de comentarios y revisión

Fase 4: Inteligencia (Meses 19-24)
├── Diseño asistido por IA
├── Simulaciones avanzadas (rendimiento, seguridad)
├── Recomendaciones de optimización
└── SDK de Plugins

Fase 5: Plataforma (Meses 25-30)
├── Marketplace de Componentes
├── Funciones empresariales (SSO, auditoría, on-prem)
├── Integraciones de socios
└── Ecosistema comunitario
```

---

## 📚 Documentación

| Documento | Propósito |
|-----------|-----------|
| [PRD](docs/prd.md) | Documento de Requisitos del Producto - Visión y especificación completa |
| [Diseño de Arquitectura](docs/ARCHITECTURE-DESIGN.md) | Decisiones de arquitectura técnica |
| [EPICS-ENGINE-2D](docs/EPICS-ENGINE-2D.md) | Hoja de ruta de implementación e historias de usuario |
| [Documentación API](docs/) | Documentación Rust generada |

---

## 🤝 Contribuciones

ArchFlow está en sus primeras etapas, ¡y necesitamos contribuidores para construir la base!

### Cómo Contribuir

1. **Explora los Crates**: Comienza con `archflow-core` para entender el sistema de tipos
2. **Elige una Feature**: Revisa los [EPICS](docs/EPICS-ENGINE-2D.md) para trabajo disponible
3. **Sigue TDD**: Escribe pruebas primero, luego implementa
4. **Envía PR**: Abre un pull request con descripción clara

### Prioridades Actuales

- Diseño e implementación del sistema de componentes
- Especificación de AUF (Architecture Universal Format)
- Optimizaciones de renderizado (soporte WebGPU)
- Bibliotecas de componentes para proveedores cloud
- Infraestructura de CI/CD y pruebas

---

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](LICENSE) para más detalles.

---

## 🙏 Reconocimientos

- [kurbo](https://github.com/linebender/kurbo) - Biblioteca de geometría 2D
- [glam](https://github.com/bitshifter/glam-rs) - Biblioteca matemática SIMD
- [tldraw](https://github.com/tldraw/tldraw) - Inspiración para interacción de canvas
- [Terraform](https://www.terraform.io/) - Inspiración de formato IaC
- [Modelo C4](https://c4model.com/) - Metodología de visualización de arquitectura

---

<p align="center">
  <strong>Arquitectura como Sistema Vivo</strong><br>
  Donde los diagramas se convierten en infraestructura desplegable.
</p>
