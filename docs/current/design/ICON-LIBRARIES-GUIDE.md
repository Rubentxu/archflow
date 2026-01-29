# Icon Libraries Integration Guide - ArchFlow

## 📋 Guía de Integración de Librerías de Iconos Open Source

Integración de librerías de iconos SVG profesionales y open source para ArchFlow.

---

## 🎯 Librerías Recomendadas

### 1. **Phosphor Icons** ⭐ RECOMENDADO PRINCIPAL
- **Website**: https://phosphoricons.com
- **GitHub**: https://github.com/phosphor-icons/core
- **License**: MIT
- **Icons**: 9000+ iconos
- **Styles**: Thin, Light, Regular, Bold, Fill, Duotone
- **Por qué**: Modernos, consistentes, altamente personalizables

```bash
# Instalación npm
npm install @phosphor-icons/core

# Uso en HTML
<script src="https://unpkg.com/@phosphor-icons/core@2.0.2/script.js"></script>
<i class="ph ph-rectangle"></i>
<i class="ph-fill ph-circle"></i>
```

**Iconos útiles para ArchFlow:**
- `ph-selection`, `ph-selection-all` - Selección
- `ph-cursor-click` - Cursor
- `ph-square`, `ph-circle` - Formas básicas
- `ph-text-t` - Texto
- `ph-hand-grabbing`, `ph-hand` - Pan
- `ph-magnifying-glass-plus` - Zoom
- `ph-arrows-out-cardinal` - Mover
- `ph-arrows-clockwise` - Rotar
- `ph-copy`, `ph-trash`, `ph-scissors` - Acciones
- `ph-arrow-counter-clockwise` - Undo
- `ph-download`, `ph-upload` - Import/Export
- `ph-stack`, `ph-squares-four` - Layers
- `ph-users` - Colaboración
- `ph-gear`, `ph-sliders-horizontal` - Settings

---

### 2. **Lucide Icons** ⭐ ALTERNATIVA PRINCIPAL
- **Website**: https://lucide.dev
- **GitHub**: https://github.com/lucide-icons/lucide
- **License**: ISC
- **Icons**: 1000+ iconos
- **Fork de**: Feather Icons (mejorado)
- **Por qué**: Limpios, consistentes, excelente para UI

```bash
# Instalación npm
npm install lucide

# Uso en JavaScript
import { createIcons, icons } from 'lucide';
createIcons({ icons });
```

**Iconos útiles para ArchFlow:**
- `mouse-pointer-2`, `mouse-pointer-click` - Cursor
- `square`, `circle` - Formas
- `type` - Texto
- `hand`, `move` - Pan/Mover
- `zoom-in`, `zoom-out` - Zoom
- `rotate-ccw`, `rotate-cw` - Rotar
- `copy`, `trash-2`, `scissors` - Acciones
- `undo-2`, `redo-2` - Undo/Redo
- `download`, `upload` - Import/Export
- `layers`, `layer-stack` - Layers
- `users`, `user` - Colaboración
- `settings`, `sliders` - Settings
- `group`, `ungroup` - Grupos
- `align-left`, `align-center`, `align-right` - Alineación
- `bring-to-front`, `send-to-back` - Orden

---

### 3. **Heroicons** (por Tailwind Labs)
- **Website**: https://heroicons.com
- **GitHub**: https://github.com/tailwindlabs/heroicons
- **License**: MIT
- **Icons**: 292 iconos
- **Styles**: Outline (24px), Solid (24px), Mini (20px)
- **Por qué**: Diseñados por Tailwind Labs, muy pulidos

```bash
# Instalación npm
npm install heroicons

# Uso (React)
import { BeakerIcon } from '@heroicons/react/24/solid'
```

---

### 4. **Tabler Icons**
- **Website**: https://tabler-icons.io
- **GitHub**: https://github.com/tabler/tabler-icons
- **License**: MIT
- **Icons**: 4500+ iconos
- **Por qué**: Cantidad masiva de iconos, muy detallados

```bash
# Instalación npm
npm install @tabler/icons

# Uso
import { IconCircle } from '@tabler/icons';
```

---

### 5. **Simple Icons** (Marcas/Brands)
- **Website**: https://simpleicons.org
- **GitHub**: https://github.com/simple-icons/simple-icons
- **License**: CC0 (Public Domain)
- **Icons**: 2900+ logos de marcas
- **Por qué**: Perfecto para logos de AWS, Azure, GCP, etc.

```bash
# Instalación npm
npm install simple-icons

# Uso
import { siAmazonaws, siGooglecloud, siMicrosoftazure } from 'simple-icons';
```

**Marcas útiles para diagramas cloud:**
- AWS, Azure, GCP
- Docker, Kubernetes
- GitHub, GitLab
- MongoDB, PostgreSQL, MySQL
- React, Vue, Angular
- Node.js, Python, Java
- Nginx, Apache
- Linux, Windows

---

## 🎨 Iconos Específicos por Categoría

### Formas Geométricas (General Library)
Como no hay iconos de formas geométricas perfectas en estas librerías, recomendamos:

**Opción 1**: SVGs inline simples
```html
<!-- Rectangle -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="3" y="5" width="18" height="14" rx="2"/>
</svg>

<!-- Circle -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <circle cx="12" cy="12" r="9"/>
</svg>

<!-- Diamond -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 2L22 12L12 22L2 12Z"/>
</svg>

<!-- Triangle -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 3L22 20H2L12 3Z"/>
</svg>

<!-- Hexagon -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 2L21 7V17L12 22L3 17V7L12 2Z"/>
</svg>
```

**Opción 2**: Usar Lucide con stroke-width modificado
```html
<!-- Cuadrado desde Lucide -->
<i data-lucide="square" stroke-width="1.5"></i>
```

---

### Diagramas de Flujo (Flowchart Library)

Usar combinación de iconos + SVGs simples:

```html
<!-- Start/End (Rounded rect) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="4" y="8" width="16" height="8" rx="4"/>
</svg>

<!-- Process (Rectangle) -->
<i class="ph ph-square"></i>

<!-- Decision (Diamond) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 2L22 12L12 22L2 12Z"/>
</svg>

<!-- Database (Cylinder) -->
<i class="ph ph-cylinder"></i>

<!-- Document -->
<i class="ph ph-file-text"></i>

<!-- Arrow -->
<i class="ph ph-arrow-right"></i>
```

---

### Diagramas UML (UML Library)

```html
<!-- Class (Rectangle con compartimentos) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="3" y="3" width="18" height="18"/>
  <line x1="3" y1="9" x2="21" y2="9"/>
  <line x1="3" y1="15" x2="21" y2="15"/>
</svg>

<!-- Actor (Person) -->
<i class="ph ph-user"></i>

<!-- Use Case (Ellipse - usar oval) -->
<i class="ph ph-circle" style="transform: scaleX(1.5);"></i>

<!-- Package (Folder) -->
<i class="ph ph-folder"></i>

<!-- Component (Hexágono simplificado) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 2L21 7V17L12 22L3 17V7L12 2Z"/>
  <line x1="12" y1="12" x2="12" y2="12"/>
</svg>
```

---

### Iconos Cloud (AWS/Azure/GCP)

**AWS Architecture Icons** (Oficiales):
- **Source**: https://aws.amazon.com/architecture/icons/
- **Format**: SVG, PNG
- **License**: Permissive (verificar uso específico)
- **Descargar**: https://aws.amazon.com/architecture/icons/

**Azure Architecture Icons** (Oficiales):
- **Source**: https://learn.microsoft.com/en-us/azure/architecture/icons/
- **Format**: SVG
- **License**: Microsoft Permissive License
- **Descargar**: https://learn.microsoft.com/en-us/azure/architecture/icons/

**Google Cloud Icons** (Oficiales):
- **Source**: https://cloud.google.com/icons
- **Format**: SVG, PNG
- **License**: Permissive
- **Descargar**: https://cloud.google.com/icons

**Alternativa - Simple Icons** (para logos simples):
```javascript
import { siAmazonaws, siMicrosoftazure, siGooglecloud } from 'simple-icons';
```

---

### C4 Model Icons (Especialidad ArchFlow)

Para C4 Model, necesitamos iconos específicos:

```html
<!-- Person (User con caja) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <circle cx="12" cy="8" r="4"/>
  <path d="M4 20C4 16 7 13 12 13C17 13 20 16 20 20"/>
</svg>

<!-- Software System (Caja con borde grueso) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2.5">
  <rect x="3" y="6" width="18" height="12" rx="2"/>
</svg>

<!-- Container (Caja redondeada) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="4" y="5" width="16" height="14" rx="4"/>
</svg>

<!-- Component (Hexágono) -->
<svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
  <path d="M12 2L21 7V17L12 22L3 17V7L12 2Z"/>
</svg>
```

**Colores C4 Oficiales**:
- Person: `#08427b`
- External Person: `#686868`
- Software System: `#1168bd`
- External System: `#999999`
- Container: `#438dd5`
- Component: `#85bbf0`

---

## 🛠️ Implementación

### 1. Estructura de Assets

```
assets/
├── icons/
│   ├── lib/                    # Iconos de librerías open source
│   │   ├── phosphor/          # Phosphor Icons
│   │   ├── lucide/            # Lucide Icons
│   │   └── simple/            # Simple Icons (brands)
│   ├── custom/                 # Iconos SVGs personalizados
│   │   ├── shapes/            # Formas geométricas
│   │   ├── flowchart/         # Símbolos de flujo
│   │   ├── uml/               # Símbolos UML
│   │   └── c4/                # Símbolos C4 Model
│   └── cloud/
│       ├── aws/               # Iconos AWS oficiales
│       ├── azure/             # Iconos Azure oficiales
│       └── gcp/               # Iconos GCP oficiales
└── fonts/
    └── ...
```

### 2. Configuración Package.json

```json
{
  "dependencies": {
    "@phosphor-icons/core": "^2.0.2",
    "lucide": "^0.294.0",
    "simple-icons": "^11.0.0"
  }
}
```

### 3. Sistema de Iconos en Rust

```rust
// crates/archflow-sdk/src/icons/mod.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Referencia a un icono
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IconRef {
    pub source: IconSource,
    pub name: String,
    pub style: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum IconSource {
    Phosphor,      // @phosphor-icons/core
    Lucide,        // lucide
    Simple,        // simple-icons
    Heroicons,     // heroicons
    Custom,        // SVGs personalizados
}

impl IconRef {
    /// Crea referencia a Phosphor Icon
    pub fn phosphor(name: &str) -> Self {
        Self {
            source: IconSource::Phosphor,
            name: name.to_string(),
            style: Some("regular".to_string()),
        }
    }
    
    /// Crea referencia a Lucide Icon
    pub fn lucide(name: &str) -> Self {
        Self {
            source: IconSource::Lucide,
            name: name.to_string(),
            style: None,
        }
    }
    
    /// Crea referencia a Simple Icon (brand)
    pub fn simple(name: &str) -> Self {
        Self {
            source: IconSource::Simple,
            name: name.to_string(),
            style: None,
        }
    }
    
    /// Crea referencia a icono SVG personalizado
    pub fn custom(name: &str) -> Self {
        Self {
            source: IconSource::Custom,
            name: name.to_string(),
            style: None,
        }
    }
    
    /// Genera el HTML/SVG para el icono
    pub fn to_html(&self) -> String {
        match self.source {
            IconSource::Phosphor => {
                format!(r#"<i class="ph {} ph-{}"></i>"#, 
                    self.style.as_ref().unwrap_or(&"regular".to_string()),
                    self.name
                )
            }
            IconSource::Lucide => {
                format!(r#"<i data-lucide="{}"></i>"#, self.name)
            }
            IconSource::Simple => {
                format!(r#"<svg class="icon-simple" data-icon="{}"></svg>"#, self.name)
            }
            IconSource::Custom => {
                format!(r#"<svg class="icon-custom icon-{}"></svg>"#, self.name)
            }
            IconSource::Heroicons => {
                format!(r#"<svg class="icon-hero" data-icon="{}"></svg>"#, self.name)
            }
        }
    }
}
```

### 4. Uso en Component Library

```rust
// Librería con iconos de diferentes fuentes
impl LibraryManager {
    fn create_general_library() -> ComponentLibrary {
        ComponentLibrary {
            id: "general".to_string(),
            name: "General".to_string(),
            categories: vec![
                LibraryCategory {
                    id: "basic".to_string(),
                    name: "Básicos".to_string(),
                    icon: IconRef::phosphor("shapes"),
                    items: vec![
                        LibraryItem {
                            id: "rect".to_string(),
                            name: "Rectangle".to_string(),
                            preview: ItemPreview::CustomSvg(include_str!("../../../assets/icons/custom/shapes/rectangle.svg")),
                            // ...
                        },
                        LibraryItem {
                            id: "circle".to_string(),
                            name: "Circle".to_string(),
                            preview: ItemPreview::CustomSvg(include_str!("../../../assets/icons/custom/shapes/circle.svg")),
                            // ...
                        },
                    ],
                },
            ],
            // ...
        }
    }
    
    fn create_aws_library() -> ComponentLibrary {
        ComponentLibrary {
            id: "aws".to_string(),
            name: "AWS Architecture".to_string(),
            categories: vec![
                LibraryCategory {
                    id: "compute".to_string(),
                    name: "Compute".to_string(),
                    icon: IconRef::simple("amazonaws"), // Simple Icons
                    items: vec![
                        LibraryItem {
                            id: "ec2".to_string(),
                            name: "EC2".to_string(),
                            preview: ItemPreview::SvgPath("assets/icons/cloud/aws/ec2.svg".to_string()),
                            // ...
                        },
                    ],
                },
            ],
            // ...
        }
    }
}
```

### 5. JavaScript Helper

```javascript
// utils/icons.js

import { createIcons, icons as lucideIcons } from 'lucide';

// Inicializar Lucide icons
export function initLucideIcons() {
  createIcons({ icons: lucideIcons });
}

// Mapeo de iconos por categoría
export const ICONS = {
  tools: {
    select: { lib: 'phosphor', name: 'cursor' },
    rect: { lib: 'custom', name: 'rectangle' },
    ellipse: { lib: 'custom', name: 'circle' },
    line: { lib: 'phosphor', name: 'line-segment' },
    text: { lib: 'phosphor', name: 'text-t' },
    hand: { lib: 'phosphor', name: 'hand-grabbing' },
  },
  actions: {
    undo: { lib: 'phosphor', name: 'arrow-counter-clockwise' },
    redo: { lib: 'phosphor', name: 'arrow-clockwise' },
    copy: { lib: 'phosphor', name: 'copy' },
    cut: { lib: 'phosphor', name: 'scissors' },
    paste: { lib: 'phosphor', name: 'clipboard-text' },
    delete: { lib: 'phosphor', name: 'trash' },
  },
  view: {
    zoomIn: { lib: 'phosphor', name: 'magnifying-glass-plus' },
    zoomOut: { lib: 'phosphor', name: 'magnifying-glass-minus' },
    fit: { lib: 'phosphor', name: 'arrows-out' },
    grid: { lib: 'phosphor', name: 'grid-four' },
  }
};

// Renderizar icono
export function renderIcon(iconRef) {
  switch (iconRef.source) {
    case 'Phosphor':
      return `<i class="ph ph-${iconRef.name}"></i>`;
    case 'Lucide':
      return `<i data-lucide="${iconRef.name}"></i>`;
    case 'Custom':
      return loadCustomSvg(iconRef.name);
    default:
      return `<i class="ph ph-question"></i>`;
  }
}

// Cargar SVG personalizado
async function loadCustomSvg(name) {
  const response = await fetch(`/assets/icons/custom/${name}.svg`);
  return await response.text();
}
```

---

## 📦 Descarga de Iconos Oficiales Cloud

### Script de descarga (Python/Node)

```python
# scripts/download-cloud-icons.py
#!/usr/bin/env python3
"""
Descarga iconos oficiales de AWS, Azure y GCP
"""
import os
import urllib.request
import zipfile
import shutil

ASSETS_DIR = "assets/icons/cloud"

def download_aws_icons():
    """Descarga iconos AWS Architecture"""
    url = "https://d1.awsstatic.com/webteam/architecture-icons/AWS-Architecture-Icons_SVG_2023.11.20.zip"
    
    print("Descargando iconos AWS...")
    urllib.request.urlretrieve(url, "/tmp/aws-icons.zip")
    
    # Extraer
    with zipfile.ZipFile("/tmp/aws-icons.zip", 'r') as zip_ref:
        zip_ref.extractall("/tmp/aws-icons")
    
    # Mover a assets
    aws_dir = f"{ASSETS_DIR}/aws"
    os.makedirs(aws_dir, exist_ok=True)
    shutil.copytree("/tmp/aws-icons", aws_dir, dirs_exist_ok=True)
    
    print(f"✓ Iconos AWS descargados en {aws_dir}")

def download_azure_icons():
    """Descarga iconos Azure Architecture"""
    url = "https://learn.microsoft.com/en-us/azure/architecture/icons/"
    # Seguir instrucciones oficiales de Microsoft
    print("⚠ Azure icons: Descargar manualmente desde:")
    print("  https://learn.microsoft.com/en-us/azure/architecture/icons/")

def download_gcp_icons():
    """Descarga iconos GCP"""
    print("⚠ GCP icons: Descargar manualmente desde:")
    print("  https://cloud.google.com/icons")

if __name__ == "__main__":
    os.makedirs(ASSETS_DIR, exist_ok=True)
    download_aws_icons()
    download_azure_icons()
    download_gcp_icons()
```

---

## 📄 Licencias

### Resumen de Licencias

| Librería | Licencia | Uso Comercial | Modificación | Atribución |
|----------|----------|---------------|--------------|------------|
| **Phosphor Icons** | MIT | ✅ | ✅ | Recomendada |
| **Lucide** | ISC | ✅ | ✅ | Recomendada |
| **Heroicons** | MIT | ✅ | ✅ | Recomendada |
| **Tabler Icons** | MIT | ✅ | ✅ | Recomendada |
| **Simple Icons** | CC0 | ✅ | ✅ | No requerida |
| **AWS Icons** | Permissive | ✅ | Verificar | Verificar |
| **Azure Icons** | MS License | ✅ | ❌ | Requerida |

### Atribución (package.json)

```json
{
  "name": "archflow-web",
  "version": "1.0.0",
  "dependencies": {
    "@phosphor-icons/core": "^2.0.2",
    "lucide": "^0.294.0"
  },
  "credits": {
    "icons": [
      "Phosphor Icons - https://phosphoricons.com (MIT)",
      "Lucide Icons - https://lucide.dev (ISC)",
      "Simple Icons - https://simpleicons.org (CC0)",
      "AWS Architecture Icons - https://aws.amazon.com/architecture/icons/",
      "Azure Architecture Icons - https://learn.microsoft.com/en-us/azure/architecture/icons/"
    ]
  }
}
```

---

## ✅ Checklist de Integración

### Fase 1: Setup (Week 1)
- [ ] Instalar Phosphor Icons vía npm
- [ ] Instalar Lucide Icons vía npm
- [ ] Crear estructura de carpetas assets/icons/
- [ ] Descargar iconos AWS oficiales
- [ ] Descargar iconos Azure oficiales
- [ ] Crear SVGs personalizados (formas geométricas)

### Fase 2: Modelo de Datos (Week 1)
- [ ] Implementar `IconRef` y `IconSource` en Rust
- [ ] Generar TypeScript types
- [ ] Crear mapeo iconos por categoría

### Fase 3: UI Integration (Week 2)
- [ ] Configurar Phosphor Icons en HTML
- [ ] Configurar Lucide Icons
- [ ] Crear helper JavaScript para renderizado
- [ ] Integrar con Component Library panel

### Fase 4: Cloud Icons (Week 2)
- [ ] Organizar iconos AWS/AZURE/GCP
- [ ] Crear mapeo nombres → archivos SVG
- [ ] Implementar lazy loading de iconos

### Fase 5: Custom Icons (Week 3)
- [ ] Crear SVGs formas geométricas
- [ ] Crear SVGs C4 Model
- [ ] Crear SVGs Flowchart/UML
- [ ] Optimizar SVGs (svgo)

---

## 🚀 Comandos Rápidos

```bash
# 1. Instalar dependencias de iconos
cd packages/archflow-web
npm install @phosphor-icons/core lucide simple-icons

# 2. Descargar iconos cloud
python3 scripts/download-cloud-icons.py

# 3. Optimizar SVGs personalizados
npx svgo assets/icons/custom/**/*.svg

# 4. Construir
wasm-pack build --target web
```

---

## 📚 Referencias

- [Phosphor Icons](https://phosphoricons.com) - Iconos modernos y versátiles
- [Lucide Icons](https://lucide.dev) - Fork mejorado de Feather Icons
- [Heroicons](https://heroicons.com) - Por Tailwind Labs
- [Tabler Icons](https://tabler-icons.io) - 4500+ iconos
- [Simple Icons](https://simpleicons.org) - Logos de marcas
- [AWS Icons](https://aws.amazon.com/architecture/icons/) - Iconos oficiales AWS
- [Azure Icons](https://learn.microsoft.com/en-us/azure/architecture/icons/) - Iconos oficiales Azure
- [SVGO](https://github.com/svg/svgo) - Optimizador de SVGs

---

*Documento actualizado: Enero 2025*  
*Recomendación principal: Phosphor Icons + Lucide Icons*
