# EPIC-005: 2D Transformation Matrix System
## Sistema de Matriz de Transformación 2D

---

## 📋 Metadatos

| Campo | Valor |
|-------|-------|
| **ID** | EPIC-005 |
| **Título** | 2D Transformation Matrix System |
| **Prioridad** | 🟡 MEDIA |
| **Complejidad** | Alta |
| **Estimación** | 2 semanas |
| **Depende de** | Ninguna (puede ser paralela) |
| **Bloquea** | Ninguna |
| **Estado** | 📝 Planeación - Requiere Implementación |
| **Fecha Creación** | 2025-01-28 |

---

## 🎯 Objetivo

Implementar un sistema completo de matrices de transformación 2D para soportar transformaciones complejas (composición, inversión, concatenación) con máximo rendimiento y type safety.

### Motivación

Las entidades actuales solo tienen posición simple. Sin matriz de transformación:
1. **No hay rotación nativa** en entidades
2. **No hay scale** como transformación
3. **No hay skew** u otras transformaciones
4. **No hay composición** de transformaciones
5. **Transformaciones son limitadas** a posición + tamaño

### Valor de Negocio

- **Poder de expresión**: Transformaciones arbitrarias
- **Composición**: Encadenar transformaciones
- **Performance**: GPU-friendly (vec4/mat4)
- **Future-proof**: Preparado para 3D si es necesario

---

## 📚 Investigación y Mejores Prácticas

### Fuentes Consultadas

1. **[nalgebra-glm Documentation](https://docs.rs/nalgebra-glm)**
   - Librería de álgebra lineal para Rust
   - 3x3 matrices para transformaciones 2D
   - Probada en producción

2. **[Affine Transforms Crate](https://lib.rs/crates/affine_transforms)**
   - Diseñada específicamente para transforms
   - Soporta 2D y 3D
   - Conversiones entre representaciones

3. **[glam-rs Issue #25](https://github.com/bitshifter/glam-rs/issues/25)**
   - Discusión sobre optimización de matrices
   - 4x3 floats suficiente para affine (no 4x4)
   - Memory layout considerations

4. **[Understanding Affine Transformation Matrix](https://www.oreateai.com/blog/understanding-the-affine-transformation-matrix-a-key-to-2d-graphics/0cd7dcb5440e264a29ec6b230c514138)**
   - Guía comprensiva de affine transforms
   - Matemáticas detrás de transforms
   - Casos de uso en 2D graphics

5. **[How to create a 2D matrix in Rust](https://timclicks.dev/tip/how-to-create-a-2d-matrix-in-rust)**
   - Tutorial de implementación
   - Ejemplos prácticos de código

### Decisiones Arquitectónicas

#### 1. **Usar nalgebra para Operaciones Matriciales**

**Razón**: Librería probada, type-safe,高性能

```rust
use nalgebra::{Matrix3, Vector2, Rotation2, Scale2, Translation2};

pub struct Transform {
    // Matriz 3x3 para transformaciones 2D affine
    matrix: Matrix3<f64>,
}

impl Transform {
    /// Crear transformación identidad
    pub fn identity() -> Self {
        Self {
            matrix: Matrix3::identity(),
        }
    }

    /// Crear traslación
    pub fn from_translation(x: f64, y: f64) -> Self {
        Self {
            matrix: Translation2::new(x, y).to_homogeneous(),
        }
    }

    /// Crear rotación (en grados)
    pub fn from_rotation(degrees: f64) -> Self {
        Self {
            matrix: Rotation2::new(degrees.to_radians()).to_homogeneous(),
        }
    }

    /// Crear scale
    pub fn from_scale(sx: f64, sy: Option<f64>) -> Self {
        Self {
            matrix: Scale2::new(sx, sy.unwrap_or(sx)).to_homogeneous(),
        }
    }

    /// Componer con otra transformación
    pub fn compose(&self, other: &Transform) -> Self {
        Self {
            matrix: self.matrix * other.matrix,
        }
    }

    /// Transformar un punto
    pub fn transform_point(&self, point: Vector2<f64>) -> Vector2<f64> {
        self.matrix.transform_point(&point)
    }

    /// Invertir transformación
    pub fn inverse(&self) -> Option<Transform> {
        self.matrix.try_inverse().map(|matrix| Transform { matrix })
    }

    /// Extraer componentes
    pub fn decomposition(&self) -> TransformDecomposition {
        // Descomponer en: traslación, rotación, scale
        // Algoritmo de matriz SVD similar
        TransformDecomposition {
            translation: Vector2::new(self.matrix[(0, 2)], self.matrix[(1, 2)]),
            rotation: self.matrix[(0, 0)].atan2(self.matrix[(1, 0)]).to_degrees(),
            scale_x: (self.matrix[(0, 0)].powi(2) + self.matrix[(1, 0)].powi(2)).sqrt(),
            scale_y: (self.matrix[(0, 1)].powi(2) + self.matrix[(1, 1)].powi(2)).sqrt(),
        }
    }
}
```

**Ventajas**:
- ✅ Type-safe (no errors de dimensión)
- ✅ Probada en producción
- ✅ Optimizada (SIMD)
- ✅ API ergonómica

**Desventajas**:
- ⚠️ Dependencia externa
- ⚠️ Overhead si solo se necesita transformaciones simples

#### 2. **Memory Layout: Array-of-Structs (AoS)**

**Razón**: Mejor para uso general y cache locality

```rust
// Representation: Column-major ( nalgebra default)
pub struct Transform {
    // [a  b  tx]
    // [c  d  ty]
    // [0  0  1 ]
    pub matrix: [[f64; 3]; 3],
}

// En memoria (row-major para visualización):
// [a, c, 0, b, d, 0, tx, ty, 1]
```

**Ventajas**:
- ✅ Estándar en graphics
- ✅ Cache-friendly para operaciones comunes
- ✅ Compatible con APIs existentes

#### 3. **Representación Compacta para Storage**

**Razón**: Ahorrar memoria para entidades con transformación simple

```rust
pub enum CompactTransform {
    /// Solo traslación (caso más común)
    Translation { x: f64, y: f64 },

    /// Traslación + escala
    Scale {
        x: f64,
        y: f64,
        scale_x: f64,
        scale_y: f64,
    },

    /// Transformación completa (matriz 3x3)
    Full(Transform),
}

impl CompactTransform {
    pub fn to_matrix(&self) -> Matrix3<f64> {
        match self {
            CompactTransform::Translation { x, y } => {
                Transform::from_translation(*x, *y).matrix
            }
            CompactTransform::Scale { x, y, scale_x, scale_y } => {
                Transform::from_translation(*x, *y)
                    .compose(&Transform::from_scale(*scale_x, Some(*scale_y)))
                    .matrix
            }
            CompactTransform::Full(t) => t.matrix,
        }
    }
}
```

**Ventajas**:
- ✅ Ahorra memoria para casos comunes
- ✅ Opción de upgrade a Full cuando se necesita
- ✅ Transparente para usuario

---

## 🏗️ Arquitectura Propuesta

### Diagrama de Componentes

```
┌─────────────────────────────────────────────────────────────┐
│                    Entity                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • id: EntityId                                   │  │
│  │  • position: Vec2                                  │  │
│  │  • size: Vec2                                     │  │
│  │  • transform: CompactTransform (NEW)               │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Transform                                 │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  • from_translation(x, y)                           │  │
│  │  • from_rotation(degrees)                           │  │
│  │  • from_scale(sx, sy)                               │  │
│  │  • compose(other)                                   │  │
│  │  • inverse()                                        │  │
│  │  • transform_point(point)                           │  │
│  │  • decomposition()                                  │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┴───────────────┐
         │                               │
         ▼                               ▼
┌─────────────────┐           ┌─────────────────┐
│  nalgebra crate  │           │ Storage Layer   │
│  • Matrix3       │           │  • Serialized  │
│  • Vector2       │           │  • Compact      │
│  • Rotation2     │           │                 │
└─────────────────┘           └─────────────────┘
```

### Módulos

```
archflow-core/src/
└── transform/
    ├── mod.rs                 # Re-exports
    ├── transform.rs           # Transform struct
    ├── compact.rs             # CompactTransform enum
    ├── decomposition.rs       # TransformDecomposition
    └── ops.rs                 # Operaciones de transformación
```

---

## 📝 Historias de Usuario

### US-005.1: Transform Básica

**Como** desarrollador del SDK
**Quiero** crear transformaciones básicas
**Para** mover, rotar y escalar entidades

#### Criterios de Aceptación

- [ ] **CA-001**: from_translation() crea traslación
- [ ] **CA-002**: from_rotation() crea rotación
- [ ] **CA-003**: from_scale() crea scale
- [ ] **CA-004**: transform_point() aplica transformación
- [ ] **CA-005**: Operaciones son type-safe

#### Tests TDD

```rust
#[test]
fn test_identity_transform() {
    let t = Transform::identity();
    let point = Vector2::new(100.0, 100.0);

    let transformed = t.transform_point(point);

    assert_eq!(transformed.x, 100.0);
    assert_eq!(transformed.y, 100.0);
}

#[test]
fn test_translation_transform() {
    let t = Transform::from_translation(50.0, 25.0);
    let point = Vector2::new(100.0, 100.0);

    let transformed = t.transform_point(point);

    assert_eq!(transformed.x, 150.0);
    assert_eq!(transformed.y, 125.0);
}

#[test]
fn test_rotation_transform() {
    let t = Transform::from_rotation(90.0);
    let point = Vector2::new(100.0, 0.0); // En el eje X

    let transformed = t.transform_point(point);

    assert!((transformed.x - 0.0).abs() < 0.001); // Cerca de 0
    assert!((transformed.y - 100.0).abs() < 0.001); // Cerca de 100
}

#[test]
fn test_scale_transform() {
    let t = Transform::from_scale(2.0, None); // Uniform scale
    let point = Vector2::new(100.0, 100.0);

    let transformed = t.transform_point(point);

    assert_eq!(transformed.x, 200.0);
    assert_eq!(transformed.y, 200.0);
}

#[test]
fn test_non_uniform_scale() {
    let t = Transform::from_scale(2.0, Some(3.0));
    let point = Vector2::new(100.0, 100.0);

    let transformed = t.transform_point(point);

    assert_eq!(transformed.x, 200.0); // 2x
    assert_eq!(transformed.y, 300.0); // 3x
}

#[test]
fn test_transform_performance() {
    let t = Transform::from_rotation(45.0)
        .compose(&Transform::from_scale(2.0, None))
        .compose(&Transform::from_translation(100.0, 50.0));

    let point = Vector2::new(10.0, 20.0);

    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = t.transform_point(point);
    }
    let elapsed = start.elapsed();

    // Debe ser muy rápido (< 1ns por operación)
    assert!(elapsed.as_nanos() < 100_000, "Transform too slow: {:?}", elapsed);
}
```

---

### US-005.2: Composición de Transformaciones

**Como** desarrollador del SDK
**Quiero** componer múltiples transformaciones
**Para** crear transforms complejas

#### Criterios de Aceptación

- [ ] **CA-001**: compose() encadena transformaciones
- [ ] **CA-002**: Orden de composición es correcto (T1 * T2)
- [ ] **CA-003**: Composición es asociativa
- [ ] **CA-004**: Funciona con cualquier número de transforms

#### Tests TDD

```rust
#[test]
fn test_transform_composition() {
    let t1 = Transform::from_translation(10.0, 0.0);
    let t2 = Transform::from_scale(2.0, None);

    let composed = t1.compose(&t2);
    let point = Vector2::new(100.0, 100.0);

    let transformed = composed.transform_point(point);

    // Primero scale (100 -> 200), luego translate (+10)
    assert_eq!(transformed.x, 210.0);
    assert_eq!(transformed.y, 200.0);
}

#[test]
fn test_composition_order() {
    let t_trans = Transform::from_translation(10.0, 0.0);
    let t_scale = Transform::from_scale(2.0, None);
    let point = Vector2::new(100.0, 100.0);

    // T_trans * T_scale: Primero scale, luego translate
    let result1 = t_trans.compose(&t_scale).transform_point(point);

    // T_scale * T_trans: Primero translate, luego scale
    let result2 = t_scale.compose(&t_trans).transform_point(point);

    // Orden importa
    assert_ne!(result1, result2);
}

#[test]
fn test_composition_associativity() {
    let t1 = Transform::from_translation(10.0, 0.0);
    let t2 = Transform::from_scale(2.0, None);
    let t3 = Transform::from_rotation(45.0);

    let point = Vector2::new(100.0, 100.0);

    // (t1 * t2) * t3 == t1 * (t2 * t3)
    let result1 = t1.compose(&t2).compose(&t3).transform_point(point);
    let result2 = t1.compose(&t2.compose(&t3)).transform_point(point);

    assert!((result1 - result2).norm() < 0.001);
}

#[test]
fn test_multiple_composition() {
    let transforms = vec![
        Transform::from_translation(10.0, 20.0),
        Transform::from_rotation(45.0),
        Transform::from_scale(2.0, None),
        Transform::from_translation(-5.0, -10.0),
    ];

    let composed = transforms.into_iter()
        .fold(Transform::identity(), |acc, t| acc.compose(&t));

    let point = Vector2::new(100.0, 100.0);
    let transformed = composed.transform_point(point);

    // Verificar que la transformación se aplicó
    assert_ne!(transformed, point);
}
```

---

### US-005.3: Inversión de Transformación

**Como** desarrollador del SDK
**Quiero** invertir transformaciones
**Para** calcular coordenadas locales desde globales

#### Criterios de Aceptación

- [ ] **CA-001**: inverse() retorna inversa de transformación
- [ ] **CA-002**: T * T⁻¹ = I (identidad)
- [ ] **CA-003**: Retorna None si no es invertible
- [ ] **CA-004**: Funciona con transforms compuestas

#### Tests TDD

```rust
#[test]
fn test_inverse_translation() {
    let t = Transform::from_translation(10.0, 20.0);
    let inv = t.inverse().unwrap();

    let point = Vector2::new(100.0, 100.0);
    let transformed = t.transform_point(point);
    let restored = inv.transform_point(transformed);

    assert!((restored - point).norm() < 0.001);
}

#[test]
fn test_inverse_rotation() {
    let t = Transform::from_rotation(45.0);
    let inv = t.inverse().unwrap();

    let point = Vector2::new(100.0, 0.0);
    let transformed = t.transform_point(point);
    let restored = inv.transform_point(transformed);

    assert!((restored - point).norm() < 0.001);
}

#[test]
fn test_inverse_scale() {
    let t = Transform::from_scale(2.0, None);
    let inv = t.inverse().unwrap();

    let point = Vector2::new(100.0, 100.0);
    let transformed = t.transform_point(point);
    let restored = inv.transform_point(transformed);

    assert!((restored - point).norm() < 0.001);
}

#[test]
fn test_inverse_composed() {
    let t1 = Transform::from_translation(10.0, 20.0);
    let t2 = Transform::from_scale(2.0, None);
    let t3 = Transform::from_rotation(45.0);

    let composed = t1.compose(&t2).compose(&t3);
    let inv = composed.inverse().unwrap();

    let point = Vector2::new(100.0, 100.0);
    let transformed = composed.transform_point(point);
    let restored = inv.transform_point(transformed);

    assert!((restored - point).norm() < 0.001);
}

#[test]
fn test_inverse_non_invertible_returns_none() {
    // Transformación con scale 0 no es invertible
    let t = Transform::from_scale(0.0, None);

    let inv = t.inverse();

    assert!(inv.is_none());
}
```

---

### US-005.4: Descomposición de Transformación

**Como** desarrollador del SDK
**Quiero** descomponer transformación en componentes
**Para** inspeccionar y editar transforms

#### Criterios de Aceptación

- [ ] **CA-001**: decomposition() retorna TRS components
- [ ] **CA-002**: Funciona con transforms compuestas
- [ ] **CA-003**: Precisión razonable (< 0.01° error)
- [ ] **CA-004**: Handle edge cases (scale negativo, etc.)

#### Implementación

```rust
pub struct TransformDecomposition {
    pub translation: Vector2<f64>,
    pub rotation: f64, // grados
    pub scale_x: f64,
    pub scale_y: f64,
    pub skew_x: f64,
    pub skew_y: f64,
}

impl Transform {
    /// Descomponer transformación en componentes TRS + skew
    ///
    /// Basado en: https://math.stackexchange.com/questions/237369/given-this-transformation-matrix-decompose-into-scale-rotation-and-translation
    pub fn decomposition(&self) -> TransformDecomposition {
        let m = &self.matrix;

        // Traslation es directa (columna 2)
        let translation = Vector2::new(m[(0, 2)], m[(1, 2)]);

        // Scale y Skew de las columnas 0 y 1
        let scale_x = (m[(0, 0)].powi(2) + m[(1, 0)].powi(2)).sqrt();
        let scale_y = (m[(0, 1)].powi(2) + m[(1, 1)].powi(2)).sqrt();

        // Rotación es el ángulo del eje X rotado
        let rotation = m[(1, 0)].atan2(m[(0, 0)]).to_degrees();

        // Skew
        let tan_skew_x = m[(0, 1)] * m[(0, 0)] + m[(1, 1)] * m[(1, 0)];
        let tan_skew_y = (m[(0, 0)] * m[(1, 0)] + m[(0, 1)] * m[(1, 1)]) / (scale_x * scale_y);

        TransformDecomposition {
            translation,
            rotation,
            scale_x,
            scale_y,
            skew_x: tan_skew_x.atan().to_degrees(),
            skew_y: tan_skew_y.atan().to_degrees(),
        }
    }

    /// Reconstruir transformación desde componentes
    pub fn from_decomposition(decomp: &TransformDecomposition) -> Self {
        let mut t = Transform::identity();

        // Aplicar en orden: translate -> rotate -> scale -> skew
        t = t.compose(&Transform::from_translation(decomp.translation.x, decomp.translation.y));
        t = t.compose(&Transform::from_rotation(decomp.rotation));
        t = t.compose(&Transform::from_scale(decomp.scale_x, Some(decomp.scale_y)));

        // Skew (si es necesario)
        if decomp.skew_x != 0.0 || decomp.skew_y != 0.0 {
            // Implementar skew transform
            t = t.compose(&Transform::from_skew(decomp.skew_x, decomp.skew_y));
        }

        t
    }
}
```

#### Tests TDD

```rust
#[test]
fn test_decomposition_identity() {
    let t = Transform::identity();
    let decomp = t.decomposition();

    assert_eq!(decomp.translation.x, 0.0);
    assert_eq!(decomp.translation.y, 0.0);
    assert_eq!(decomp.rotation, 0.0);
    assert_eq!(decomp.scale_x, 1.0);
    assert_eq!(decomp.scale_y, 1.0);
}

#[test]
fn test_decomposition_translation() {
    let t = Transform::from_translation(10.0, 20.0);
    let decomp = t.decomposition();

    assert_eq!(decomp.translation.x, 10.0);
    assert_eq!(decomp.translation.y, 20.0);
    assert_eq!(decomp.rotation, 0.0);
    assert_eq!(decomp.scale_x, 1.0);
    assert_eq!(decomp.scale_y, 1.0);
}

#[test]
fn test_decomposition_rotation() {
    let t = Transform::from_rotation(45.0);
    let decomp = t.decomposition();

    assert!((decomp.translation.x).abs() < 0.001);
    assert!((decomp.translation.y).abs() < 0.001);
    assert!((decomp.rotation - 45.0).abs() < 0.1);
    assert!((decomp.scale_x - 1.0).abs() < 0.001);
    assert!((decomp.scale_y - 1.0).abs() < 0.001);
}

#[test]
fn test_decomposition_scale() {
    let t = Transform::from_scale(2.0, Some(3.0));
    let decomp = t.decomposition();

    assert_eq!(decomp.scale_x, 2.0);
    assert_eq!(decomp.scale_y, 3.0);
}

#[test]
fn test_decomposition_reconstructs_original() {
    let original = Transform::from_translation(10.0, 20.0)
        .compose(&Transform::from_rotation(45.0))
        .compose(&Transform::from_scale(2.0, None));

    let decomp = original.decomposition();
    let reconstructed = Transform::from_decomposition(&decomp);

    let test_point = Vector2::new(100.0, 100.0);
    let result1 = original.transform_point(test_point);
    let result2 = reconstructed.transform_point(test_point);

    assert!((result1 - result2).norm() < 0.01, "Decomposition not accurate: {:?} vs {:?}", result1, result2);
}

#[test]
fn test_decomposition_performance() {
    let t = Transform::from_translation(10.0, 20.0)
        .compose(&Transform::from_rotation(45.0))
        .compose(&Transform::from_scale(2.0, None));

    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = t.decomposition();
    }
    let elapsed = start.elapsed();

    // Debe ser rápido
    assert!(elapsed.as_micros() < 1000, "Decomposition too slow: {:?}", elapsed);
}
```

---

### US-005.5: CompactTransform para Storage

**Como** desarrollador del SDK
**Quiero** representación compacta de transformaciones
**Para** ahorrar memoria en entidades

#### Criterios de Aceptación

- [ ] **CA-001**: Translation usa menos memoria que Full
- [ ] **CA-002**: to_matrix() convierte a Matrix3
- [ ] **CA-003**: Upgrading de Translation a Full es posible
- [ ] **CA-004**: Serialize/deserialize funciona

#### Implementación

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompactTransform {
    /// Solo traslación (8 bytes)
    Translation {
        x: f64,
        y: f64,
    },

    /// Traslación + scale uniforme (24 bytes)
    Scale {
        x: f64,
        y: f64,
        scale: f64,
    },

    /// Traslación + scale no uniforme (32 bytes)
    ScaleNonUniform {
        x: f64,
        y: f64,
        scale_x: f64,
        scale_y: f64,
    },

    /// Transformación completa (144 bytes = 9 * 8 * 2)
    Full {
        // Matriz 3x3 en row-major order
        #[serde(with = "serde_arrays")]
        matrix: [[f64; 3]; 3],
    },
}

impl CompactTransform {
    /// Memoria usada
    pub fn memory_usage(&self) -> usize {
        match self {
            CompactTransform::Translation { .. } => 16, // x + y + enum tag
            CompactTransform::Scale { .. } => 32,
            CompactTransform::ScaleNonUniform { .. } => 40,
            CompactTransform::Full { .. } => 152, // 9*8*2 + tag
        }
    }

    /// Convertir a Matrix3
    pub fn to_matrix(&self) -> Matrix3<f64> {
        match self {
            CompactTransform::Translation { x, y } => {
                Transform::from_translation(*x, *y).matrix
            }
            CompactTransform::Scale { x, y, scale } => {
                Transform::from_translation(*x, *y)
                    .compose(&Transform::from_scale(*scale, Some(*scale)))
                    .matrix
            }
            CompactTransform::ScaleNonUniform { x, y, scale_x, scale_y } => {
                Transform::from_translation(*x, *y)
                    .compose(&Transform::from_scale(*scale_x, Some(*scale_y)))
                    .matrix
            }
            CompactTransform::Full { matrix } => {
                // Convert from row-major to nalgebra column-major
                Matrix3::new(
                    matrix[0][0], matrix[1][0], matrix[2][0],
                    matrix[0][1], matrix[1][1], matrix[2][1],
                    matrix[0][2], matrix[1][2], matrix[2][2],
                )
            }
        }
    }

    /// Crear desde Matrix3
    pub fn from_matrix(matrix: &Matrix3<f64>) -> Self {
        let decomp = Transform { matrix: *matrix }.decomposition();

        // Intentar representación más simple posible
        if decomp.scale_x == 1.0 && decomp.scale_y == 1.0 && decomp.rotation == 0.0 {
            return CompactTransform::Translation {
                x: decomp.translation.x,
                y: decomp.translation.y,
            };
        }

        if decomp.rotation == 0.0 && decomp.scale_x == decomp.scale_y {
            return CompactTransform::Scale {
                x: decomp.translation.x,
                y: decomp.translation.y,
                scale: decomp.scale_x,
            };
        }

        if decomp.rotation == 0.0 {
            return CompactTransform::ScaleNonUniform {
                x: decomp.translation.x,
                y: decomp.translation.y,
                scale_x: decomp.scale_x,
                scale_y: decomp.scale_y,
            };
        }

        // Full transform
        let matrix_array = [
            [matrix[(0, 0)], matrix[(0, 1)], matrix[(0, 2)]],
            [matrix[(1, 0)], matrix[(1, 1)], matrix[(1, 2)]],
            [matrix[(2, 0)], matrix[(2, 1)], matrix[(2, 2)]],
        ];

        CompactTransform::Full {
            matrix: matrix_array,
        }
    }
}
```

#### Tests TDD

```rust
#[test]
fn test_translation_uses_less_memory() {
    let compact = CompactTransform::Translation { x: 10.0, y: 20.0 };
    let full = CompactTransform::Full {
        matrix: Transform::from_translation(10.0, 20.0).matrix.into(),
    };

    assert!(compact.memory_usage() < full.memory_usage());
}

#[test]
fn test_compact_to_matrix_identity() {
    let compact = CompactTransform::Translation { x: 0.0, y: 0.0 };
    let matrix = compact.to_matrix();

    assert_eq!(matrix, Matrix3::identity());
}

#[test]
fn test_compact_to_matrix_translation() {
    let compact = CompactTransform::Translation { x: 10.0, y: 20.0 };
    let matrix = compact.to_matrix();

    let expected = Transform::from_translation(10.0, 20.0).matrix;
    assert_eq!(matrix, expected);
}

#[test]
fn test_matrix_to_compact_translation() {
    let matrix = Transform::from_translation(10.0, 20.0).matrix;
    let compact = CompactTransform::from_matrix(&matrix);

    assert_matches!(compact, CompactTransform::Translation { x: 10.0, y: 20.0 });
}

#[test]
fn test_matrix_to_complect_scale_nonuniform() {
    let matrix = Transform::from_translation(10.0, 20.0)
        .compose(&Transform::from_scale(2.0, Some(3.0)))
        .matrix;

    let compact = CompactTransform::from_matrix(&matrix);

    assert_matches!(compact, CompactTransform::ScaleNonUniform { scale_x: 2.0, scale_y: 3.0, .. });
}

#[test]
fn test_serialize_deserialize_compact() {
    let original = CompactTransform::Translation { x: 10.0, y: 20.0 };

    let serialized = bincode::serialize(&original).unwrap();
    let deserialized: CompactTransform = bincode::deserialize(&serialized).unwrap();

    assert_eq!(deserialized.to_matrix(), original.to_matrix());
}

#[test]
fn test_memory_savings() {
    let translation = CompactTransform::Translation { x: 10.0, y: 20.0 };
    let scale = CompactTransform::Scale { x: 10.0, y: 20.0, scale: 2.0 };
    let full = CompactTransform::Full {
        matrix: Transform::from_translation(10.0, 20.0).matrix.into(),
    };

    let total = translation.memory_usage() + scale.memory_usage() + full.memory_usage();

    // La suma de compactos debe ser significativamente menor que 3 * full
    assert!(total < 3 * full.memory_usage());
}
```

---

## 🔬 Protocolo de Investigación

### Investigación 1: nalgebra vs Implementación Propia

**Objetivo**: Determinar si usar crate o implementar propio

**Método**:
1. Prototipar con nalgebra
2. Prototipar implementación propia minimal
3. Benchmark ambos enfoques
4. Evaluar trade-offs

**Métricas**:
- Performance (ns por operación)
- Binary size
- Compilation time
- API ergonomics

### Investigación 2: Memory Layout Impact

**Objetivo**: Evaluar AoS vs SoA para transforms

**Método**:
1. Benchmark operaciones comunes
2. Medir cache misses
3. Evaluar SIMD potential

**Métricas**:
- Cache hit rate
- Instructions per operation
- Memory bandwidth

---

## 📊 Métricas de Éxito

### Performance

| Métrica | Target | Medición |
|---------|--------|----------|
| Transform point | < 10ns | Benchmark |
| Compose | < 20ns | Benchmark |
| Inverse | < 50ns | Benchmark |
| Decompose | < 100ns | Benchmark |

### Memory

| Métrica | Target | Medición |
|---------|--------|----------|
| Translation size | 16 bytes | sizeof |
| Full transform size | 152 bytes | sizeof |
| Savings per entity | > 50% | Profile |

---

## 🚀 Plan de Implementación

### Sprint 1: Transform Básica (Semana 1)

- [ ] Wrapper sobre nalgebra
- [ ] from_translation/rotation/scale
- [ ] transform_point
- [ ] Tests básicos

### Sprint 2: Composición e Inversión (Semana 2)

- [ ] compose
- [ ] inverse
- [ ] decomposition
- [ ] Tests completos

---

## 📖 Referencias

- [nalgebra-glm docs](https://docs.rs/nalgebra-glm)
- [Affine Transforms crate](https://lib.rs/crates/affine_transforms)
- [glam-rs Issue #25](https://github.com/bitshifter/glam-rs/issues/25)
- [Affine Transform Matrix Guide](https://www.oreateai.com/blog/understanding-the-affine-transformation-matrix-a-key-to-2d-graphics/0cd7dcb5440e264a29ec6b230c514138)
- [2D Matrix in Rust](https://timclicks.dev/tip/how-to-create-a-2d-matrix-in-rust)

---

## 🔗 Dependencias

- **nalgebra** = "0.33" (o latest)
- **serde** = { version = "1.0", features = ["derive"] }
- **bincode** = "1.3" (para serialización compacta)

---

**Versión**: 1.0.0
**Última actualización**: 2025-01-28
**Autores**: ArchFlow Development Team
