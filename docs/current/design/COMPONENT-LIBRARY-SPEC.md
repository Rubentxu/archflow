# Component Library System - ArchFlow

## 📋 Especificación del Sistema de Librerías de Componentes

Sistema de librerías de componentes tipo draw.io/excalidraw para arrastrar y soltar elementos predefinidos en el canvas.

---

## 🎯 Objetivos

1. **Reusabilidad**: Componentes predefinidos reutilizables
2. **Productividad**: Crear diagramas rápidamente sin dibujar desde cero
3. **Consistencia**: Elementos con estilos coherentes
4. **Extensibilidad**: Librerías personalizables y expandibles
5. **Descubribilidad**: Búsqueda y categorización intuitiva

---

## 🗂️ Estructura de Librerías

### 1. Jerarquía de Librerías

```
Libraries/
├── Built-in/                    # Librerías incluidas por defecto
│   ├── General/                 # Formas básicas
│   ├── Flowchart/               # Diagramas de flujo
│   ├── UML/                     # Diagramas UML
│   ├── AWS/                     # Iconos AWS
│   └── C4-Model/                # Diagramas C4 (especialidad ArchFlow)
├── User/                        # Librerías del usuario
│   ├── My-Components/           # Componentes personalizados
│   └── Project-Specific/        # Librerías por proyecto
└── Community/                   # Librerías de la comunidad
    └── [Descargables]           # Marketplace de librerías
```

### 2. Categorías de Componentes

#### General (Básicos)
**Iconos**: SVGs personalizados simples + Phosphor Icons

| Componente | Descripción | Icono | Source |
|------------|-------------|-------|--------|
| Rectangle | Rectángulo simple | `<svg rect>` | Custom SVG |
| Rounded Rect | Rectángulo redondeado | `<svg rect rx>` | Custom SVG |
| Circle | Círculo perfecto | `<svg circle>` | Custom SVG |
| Ellipse | Elipse | `<svg ellipse>` | Custom SVG |
| Diamond | Rombo/diamante | `<svg polygon>` | Custom SVG |
| Triangle | Triángulo | `<svg polygon>` | Custom SVG |
| Hexagon | Hexágono | `<svg polygon>` | Custom SVG |
| Cylinder | Cilindro (DB) | `ph-cylinder` | Phosphor |
| Cloud | Nube | `ph-cloud` | Phosphor |
| Document | Documento | `ph-file-text` | Phosphor |

#### Flowchart (Diagramas de Flujo)
**Iconos**: SVGs personalizados para formas + Phosphor para símbolos

| Componente | Símbolo | Uso | Icono |
|------------|---------|-----|-------|
| Start/End | ⬭ | Inicio/fin | `<svg>` ovalado |
| Process | ⬜ | Proceso | `<svg>` rectángulo |
| Decision | ◆ | Decisión | `<svg>` rombo |
| Input/Output | ⬯ | Entrada/salida | `<svg>` paralelogramo |
| Database | 🛢️ | Base de datos | `ph-database` o `<svg>` cilindro |
| Document | 📄 | Documento | `ph-file-text` |
| Connector | ● | Conector | `ph-dot` |
| Arrow | → | Flecha | `ph-arrow-right` |

**Ver**: [ICON-LIBRARIES-GUIDE.md](./ICON-LIBRARIES-GUIDE.md) para implementación SVG

#### UML (Diagramas UML)
**Iconos**: SVGs personalizados + Phosphor/Lucide

| Componente | Símbolo | Uso | Icono |
|------------|---------|-----|-------|
| Class | ⬜ + líneas | Clase | `<svg>` rect con compartimentos |
| Interface | ⬯ | Interfaz | `<svg>` círculo con línea |
| Actor | 👤 | Actor | `ph-user` |
| Use Case | ⬭ | Caso de uso | `<svg>` elipse |
| Package | 📁 | Paquete | `ph-folder` |
| Component | ⬡ | Componente | `<svg>` hexágono |
| Node | 🖥️ | Nodo | `ph-desktop` |
| Note | 📝 | Nota | `ph-note` |

#### AWS Architecture Icons
| Categoría | Ejemplos |
|-----------|----------|
| Compute | EC2, Lambda, ECS |
| Storage | S3, EBS, Glacier |
| Database | RDS, DynamoDB, Redshift |
| Networking | VPC, CloudFront, Route53 |
| Security | IAM, KMS, WAF |

#### C4 Model (Especialidad ArchFlow)
| Nivel | Componentes |
|-------|-------------|
| **Context** | Person, System, External System |
| **Container** | Web App, Mobile App, API, Database |
| **Component** | Controller, Service, Repository |
| **Code** | Class, Interface |

---

## 🎨 Diseño de UI - Library Sidebar

### Layout

```
┌──────────────────────────────────────┐
│ 🔍 Search components...        [⚙️] │  ← Header con búsqueda
├──────────────────────────────────────┤
│ 📚 General                    [v]   │  ← Categoría colapsable
│   ⬜ Rectangle        ⬭ Rounded     │
│   ● Circle           ⬭ Ellipse      │
│   ◆ Diamond          ▲ Triangle     │
├──────────────────────────────────────┤
│ 📊 Flowchart                  [v]   │  ← Categoría colapsable
│   ⬭ Start/End        ⬜ Process     │
│   ◆ Decision         🛢️ Database    │
├──────────────────────────────────────┤
│ 🏗️ UML                        [v]   │
│   ⬜ Class           👤 Actor        │
│   ⬭ Use Case        📁 Package     │
├──────────────────────────────────────┤
│ ☁️ AWS                         [v]   │
│   🖥️ EC2             💾 S3          │
│   🗄️ RDS             🌐 VPC         │
├──────────────────────────────────────┤
│ 🏛️ C4 Model                    [v]   │  ← Especial ArchFlow
│   👤 Person          🏢 System       │
│   📱 Container       ⚙️ Component    │
├──────────────────────────────────────┤
│ ⭐ My Library                  [v]   │  ← Librería personal
│   [Custom shapes...]                 │
├──────────────────────────────────────┤
│ [+] Import Library                   │  ← Footer
└──────────────────────────────────────┘
```

### Especificaciones de Estilo

```css
/* Library Sidebar Container */
.library-sidebar {
  width: 280px;
  background: var(--color-bg-sidebar);
  border-right: 1px solid var(--color-border-divider);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Search Bar */
.library-search {
  padding: 12px;
  border-bottom: 1px solid var(--color-border-divider);
}

.library-search input {
  width: 100%;
  background: var(--color-bg-panel);
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  padding: 8px 12px;
  color: var(--color-text-primary);
  font-size: 13px;
}

.library-search input:focus {
  border-color: var(--color-primary);
  outline: none;
}

/* Category Section */
.library-category {
  border-bottom: 1px solid var(--color-border-divider);
}

.category-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  cursor: pointer;
  background: var(--color-bg-toolbar);
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.category-header:hover {
  background: var(--color-bg-hover);
}

.category-toggle {
  transition: transform 0.2s ease;
}

.category-toggle.collapsed {
  transform: rotate(-90deg);
}

/* Component Grid */
.component-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  padding: 12px;
}

/* Component Item */
.component-item {
  aspect-ratio: 1;
  background: var(--color-bg-panel);
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: grab;
  padding: 8px;
  transition: all 0.15s ease;
}

.component-item:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-primary);
}

.component-item:active {
  cursor: grabbing;
}

.component-item.dragging {
  opacity: 0.5;
}

.component-preview {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  margin-bottom: 4px;
}

.component-label {
  font-size: 10px;
  color: var(--color-text-secondary);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

/* Footer Actions */
.library-footer {
  padding: 12px;
  border-top: 1px solid var(--color-border-divider);
  margin-top: auto;
}

.import-btn {
  width: 100%;
  padding: 8px;
  background: transparent;
  border: 1px dashed var(--color-border-default);
  border-radius: 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 12px;
}

.import-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}
```

---

## 🔧 Modelo de Datos

### 1. Estructura de Librería (Rust)

```rust
// crates/archflow-sdk/src/library/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use uuid::Uuid;

/// Representa una librería de componentes
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComponentLibrary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub categories: Vec<LibraryCategory>,
    pub metadata: LibraryMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryCategory {
    pub id: String,
    pub name: String,
    pub icon: String, // Emoji o icono
    pub items: Vec<LibraryItem>,
    pub collapsed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub preview: ItemPreview,
    pub data: ComponentData,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ItemPreview {
    Icon(String),      // Emoji o código de icono
    Svg(String),       // SVG string
    Path(String),      // Path de icono
    Color(String),     // Color representativo
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComponentData {
    pub shape_type: LibraryShapeType,
    pub geometry: ComponentGeometry,
    pub style: ComponentStyle,
    pub children: Vec<ComponentData>, // Para grupos complejos
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LibraryShapeType {
    Rectangle,
    RoundedRectangle { radius: f32 },
    Ellipse,
    Diamond,
    Triangle,
    Hexagon,
    Cylinder,
    Cloud,
    Document,
    Custom { path: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComponentGeometry {
    pub width: f32,
    pub height: f32,
    pub default_x: Option<f32>,
    pub default_y: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComponentStyle {
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f32>,
    pub opacity: Option<f32>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub is_builtin: bool,
    pub is_editable: bool,
    pub source: LibrarySource,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum LibrarySource {
    BuiltIn,
    UserCreated,
    Imported { path: String },
    Community { url: String },
}
```

### 2. Manager de Librerías

```rust
// crates/archflow-sdk/src/library/manager.rs

use std::collections::HashMap;

pub struct LibraryManager {
    libraries: HashMap<String, ComponentLibrary>,
    active_library_ids: Vec<String>,
    favorites: Vec<String>,
    recent_items: Vec<String>,
}

impl LibraryManager {
    pub fn new() -> Self {
        let mut manager = Self {
            libraries: HashMap::new(),
            active_library_ids: Vec::new(),
            favorites: Vec::new(),
            recent_items: Vec::new(),
        };
        
        // Cargar librerías built-in
        manager.load_builtin_libraries();
        manager
    }
    
    /// Carga librerías incluidas por defecto
    fn load_builtin_libraries(&mut self) {
        self.register_library(Self::create_general_library());
        self.register_library(Self::create_flowchart_library());
        self.register_library(Self::create_uml_library());
        self.register_library(Self::create_c4_library());
    }
    
    /// Crea la librería "General"
    fn create_general_library() -> ComponentLibrary {
        ComponentLibrary {
            id: "general".to_string(),
            name: "General".to_string(),
            description: "Formas básicas y geométricas".to_string(),
            version: "1.0.0".to_string(),
            author: "ArchFlow".to_string(),
            categories: vec![
                LibraryCategory {
                    id: "basic".to_string(),
                    name: "Básicos".to_string(),
                    icon: "⬜".to_string(),
                    collapsed: false,
                    items: vec![
                        LibraryItem {
                            id: "rect".to_string(),
                            name: "Rectangle".to_string(),
                            description: "Rectángulo simple".to_string(),
                            preview: ItemPreview::Icon("⬜".to_string()),
                            data: ComponentData {
                                shape_type: LibraryShapeType::Rectangle,
                                geometry: ComponentGeometry {
                                    width: 120.0,
                                    height: 80.0,
                                    default_x: None,
                                    default_y: None,
                                },
                                style: ComponentStyle {
                                    fill_color: Some("#3366cc".to_string()),
                                    stroke_color: Some("#ffffff".to_string()),
                                    stroke_width: Some(1.0),
                                    opacity: Some(1.0),
                                    ..Default::default()
                                },
                                children: vec![],
                            },
                            tags: vec!["basic".to_string(), "shape".to_string()],
                        },
                        LibraryItem {
                            id: "rounded-rect".to_string(),
                            name: "Rounded Rect".to_string(),
                            description: "Rectángulo con esquinas redondeadas".to_string(),
                            preview: ItemPreview::Icon("⬭".to_string()),
                            data: ComponentData {
                                shape_type: LibraryShapeType::RoundedRectangle { radius: 8.0 },
                                geometry: ComponentGeometry {
                                    width: 120.0,
                                    height: 80.0,
                                    default_x: None,
                                    default_y: None,
                                },
                                style: ComponentStyle {
                                    fill_color: Some("#33aa66".to_string()),
                                    stroke_color: Some("#ffffff".to_string()),
                                    stroke_width: Some(1.0),
                                    opacity: Some(1.0),
                                    ..Default::default()
                                },
                                children: vec![],
                            },
                            tags: vec!["basic".to_string(), "shape".to_string()],
                        },
                        // Más items...
                    ],
                },
            ],
            metadata: LibraryMetadata {
                created_at: "2025-01-01".to_string(),
                updated_at: "2025-01-01".to_string(),
                is_builtin: true,
                is_editable: false,
                source: LibrarySource::BuiltIn,
            },
        }
    }
    
    /// Registra una nueva librería
    pub fn register_library(&mut self, library: ComponentLibrary) {
        self.libraries.insert(library.id.clone(), library);
    }
    
    /// Obtiene una librería por ID
    pub fn get_library(&self, id: &str) -> Option<&ComponentLibrary> {
        self.libraries.get(id)
    }
    
    /// Busca items en todas las librerías
    pub fn search_items(&self, query: &str) -> Vec<&LibraryItem> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        for library in self.libraries.values() {
            for category in &library.categories {
                for item in &category.items {
                    if item.name.to_lowercase().contains(&query_lower)
                        || item.description.to_lowercase().contains(&query_lower)
                        || item.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    {
                        results.push(item);
                    }
                }
            }
        }
        
        results
    }
    
    /// Instancia un componente en el canvas
    pub fn instantiate_component(
        &self,
        library_id: &str,
        item_id: &str,
        canvas_x: f32,
        canvas_y: f32,
    ) -> Option<Shape> {
        let library = self.libraries.get(library_id)?;
        
        let item = library.categories.iter()
            .flat_map(|c| &c.items)
            .find(|i| i.id == item_id)?;
        
        Some(self.create_shape_from_data(&item.data, canvas_x, canvas_y))
    }
    
    fn create_shape_from_data(&self, data: &ComponentData, x: f32, y: f32) -> Shape {
        // Crear forma basada en ComponentData
        // Integración con Canvas API
        Shape::new()
    }
    
    /// Importa una librería desde archivo
    pub fn import_library(&mut self, path: &str) -> Result<(), LibraryError> {
        let content = std::fs::read_to_string(path)?;
        let library: ComponentLibrary = serde_json::from_str(&content)?;
        self.register_library(library);
        Ok(())
    }
    
    /// Exporta una librería a archivo
    pub fn export_library(&self, library_id: &str, path: &str) -> Result<(), LibraryError> {
        let library = self.libraries.get(library_id)
            .ok_or(LibraryError::LibraryNotFound)?;
        
        let json = serde_json::to_string_pretty(library)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("Library not found")]
    LibraryNotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

---

## 🌐 Integración Web (WASM)

### API JavaScript

```javascript
// Uso desde JavaScript

// Obtener todas las librerías
const libraries = archFlowEditor.getLibraries();

// Buscar componentes
const results = archFlowEditor.searchLibraryItems("database");

// Instanciar componente en canvas
const shapeId = archFlowEditor.instantiateFromLibrary(
  "flowchart",      // library_id
  "database",       // item_id
  100,              // x
  200               // y
);

// Registrar librería personalizada
archFlowEditor.registerLibrary({
  id: "my-library",
  name: "My Components",
  categories: [...]
});

// Exportar librería
const json = archFlowEditor.exportLibrary("my-library");
downloadFile(json, "my-library.json");

// Importar librería
archFlowEditor.importLibrary(libraryJson);
```

### Eventos de Drag & Drop

```javascript
// En la librería
const componentItems = document.querySelectorAll('.component-item');

componentItems.forEach(item => {
  item.addEventListener('dragstart', (e) => {
    const libraryId = item.dataset.libraryId;
    const itemId = item.dataset.itemId;
    
    e.dataTransfer.setData('application/archflow-component', JSON.stringify({
      libraryId,
      itemId
    }));
    
    item.classList.add('dragging');
  });
  
  item.addEventListener('dragend', () => {
    item.classList.remove('dragging');
  });
});

// En el canvas
canvas.addEventListener('dragover', (e) => {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'copy';
});

canvas.addEventListener('drop', (e) => {
  e.preventDefault();
  
  const data = e.dataTransfer.getData('application/archflow-component');
  if (data) {
    const { libraryId, itemId } = JSON.parse(data);
    
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    
    // Convertir a coordenadas del canvas (considerando zoom/pan)
    const canvasX = (x - panX) / zoom;
    const canvasY = (y - panY) / zoom;
    
    archFlowEditor.instantiateFromLibrary(libraryId, itemId, canvasX, canvasY);
  }
});
```

---

## 📦 Formatos de Archivo

### 1. Formato de Librería (.archlib.json)

```json
{
  "id": "aws-architecture",
  "name": "AWS Architecture Icons",
  "description": "Official AWS architecture icons",
  "version": "1.0.0",
  "author": "Amazon Web Services",
  "categories": [
    {
      "id": "compute",
      "name": "Compute",
      "icon": "🖥️",
      "items": [
        {
          "id": "ec2",
          "name": "EC2",
          "description": "Elastic Compute Cloud",
          "preview": { "type": "Icon", "value": "🖥️" },
          "data": {
            "shape_type": { "Custom": { "path": "M0,0 h100 v60 h-100 z" } },
            "geometry": { "width": 100, "height": 60 },
            "style": {
              "fill_color": "#FF9900",
              "stroke_color": "#232F3E",
              "stroke_width": 2
            },
            "children": []
          },
          "tags": ["aws", "compute", "vm"]
        }
      ]
    }
  ],
  "metadata": {
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z",
    "is_builtin": false,
    "is_editable": true,
    "source": { "Community": { "url": "https://aws.amazon.com/architecture/icons/" } }
  }
}
```

### 2. Librerías Incluidas

Las siguientes librerías se incluirán por defecto:

1. **general.archlib.json** - Formas básicas
2. **flowchart.archlib.json** - Diagramas de flujo
3. **uml.archlib.json** - Diagramas UML
4. **c4-model.archlib.json** - Diagramas C4 (especialidad ArchFlow)

---

## 🎨 Componentes C4 Model (Especialidad)

Los diagramas C4 son el diferenciador principal de ArchFlow.

### Nivel 1: Context

```rust
// Context Level Components
let person = LibraryItem {
    id: "c4-person".to_string(),
    name: "Person".to_string(),
    data: ComponentData {
        shape_type: LibraryShapeType::Custom { 
            path: "M30,20 a10,10 0 1,0 0,20 h40 a10,10 0 1,0 0,-20 z".to_string() 
        },
        geometry: ComponentGeometry { width: 100, height: 80, default_x: None, default_y: None },
        style: ComponentStyle {
            fill_color: Some("#08427b".to_string()),
            stroke_color: Some("#052e56".to_string()),
            ..Default::default()
        },
        children: vec![],
    },
    // ...
};

let system = LibraryItem {
    id: "c4-system".to_string(),
    name: "Software System".to_string(),
    data: ComponentData {
        shape_type: LibraryShapeType::RoundedRectangle { radius: 8.0 },
        geometry: ComponentGeometry { width: 200, height: 100, default_x: None, default_y: None },
        style: ComponentStyle {
            fill_color: Some("#1168bd".to_string()),
            stroke_color: Some("#0b4884".to_string()),
            ..Default::default()
        },
        children: vec![],
    },
    // ...
};
```

### Nivel 2: Container

```rust
let web_app = LibraryItem {
    id: "c4-web-app".to_string(),
    name: "Web Application".to_string(),
    data: ComponentData {
        shape_type: LibraryShapeType::RoundedRectangle { radius: 4.0 },
        geometry: ComponentGeometry { width: 160, height: 80, default_x: None, default_y: None },
        style: ComponentStyle {
            fill_color: Some("#438dd5".to_string()),
            stroke_color: Some("#2e6299".to_string()),
            ..Default::default()
        },
        children: vec![],
    },
    // ...
};
```

---

## 📋 Checklist de Implementación

### Fase 1: Modelo de Datos
- [ ] Definir estructuras Rust (`ComponentLibrary`, `LibraryItem`, etc.)
- [ ] Implementar serialización/deserialización
- [ ] Crear tests unitarios
- [ ] Generar TypeScript types

### Fase 2: Manager
- [ ] Implementar `LibraryManager`
- [ ] Crear librerías built-in (General, Flowchart, UML)
- [ ] Implementar búsqueda
- [ ] Implementar import/export

### Fase 3: UI Web
- [ ] Crear HTML estructura del panel
- [ ] Implementar CSS según especificación
- [ ] Agregar búsqueda funcional
- [ ] Implementar colapsar/expandir categorías

### Fase 4: Drag & Drop
- [ ] Implementar drag desde librería
- [ ] Implementar drop en canvas
- [ ] Conversión de coordenadas
- [ ] Feedback visual durante drag

### Fase 5: Integración SDK
- [ ] Conectar con Canvas API
- [ ] Crear shapes desde ComponentData
- [ ] Manejar grupos complejos
- [ ] Tests de integración

### Fase 6: Librerías C4
- [ ] Crear librería C4 Model
- [ ] Implementar componentes C4
- [ ] Colores oficiales C4
- [ ] Documentación

---

## 🔗 Referencias

- [C4 Model](https://c4model.com/) - Official C4 Model documentation
- [draw.io Libraries](https://www.drawio.com/blog/custom-libraries) - Custom libraries in draw.io
- [Excalidraw Libraries](https://libraries.excalidraw.com/) - Community libraries
- [AWS Architecture Icons](https://aws.amazon.com/architecture/icons/) - AWS icons

---

*Especificación creada: Enero 2025*  
*Versión: 1.0*
