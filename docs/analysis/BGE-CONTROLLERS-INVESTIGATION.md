---
title: "BGE Controllers Investigation - Tipos y Referencias para Implementación Rust"
author: Claude Code
date: 2026-02-01
status: Final
context: Blender Game Engine Logic Bricks Controller Architecture
---

# Investigación: Controladores de Blender Game Engine (BGE)

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| Fecha | 2026-02-01 |
| Estado | Completada |
| Fuente | Blender BGE Source Code, UPBGE Documentation |
| Objetivo | Documentar todos los tipos de controladores BGE con ejemplos Rust |

---

## 🎯 Resumen Ejecutivo

Este documento investiga **todos los tipos de controladores** disponibles en Blender Game Engine (BGE) y proporciona:

1. **Lista completa de controladores BGE** con sus propiedades
2. **Ejemplo real traducido a Rust** de los controladores más importantes
3. **Referencias** para implementación de otros controladores
4. **Patrones arquitectónicos** comunes a todos los controladores
5. **Mapeo a la arquitectura de ArchFlow** (Cómo encajan en el sistema)

---

## 1. Arquitectura General de Controladores BGE

### 1.1 Jerarquía de Clases

```
SCA_IController (base abstract)
    ├── SCA_ANDController (todas las entradas deben ser TRUE)
    ├── SCA_ORController (al menos una entrada debe ser TRUE)
    ├── SCA_NANDController (NOT AND)
    ├── SCA_NORController (NOT OR)
    ├── SCA_XORController (Exclusive OR)
    ├── SCA_XNORController (Exclusive NOR)
    ├── SCA_ExpressionController (expresiones Python-like)
    └── SCA_PythonController (scripts Python completos)
```

### 1.2 Patrón Común a Todos los Controladores

**Cada controlador en BGE tiene**:

```python
# Propiedades comunes
controller.name          # Nombre del controlador
controller.type          # Tipo de controlador
controller.owner         # Objeto que posee el controlador
controller.priority      # Prioridad (0 = más alta)
controller.state         # Máscara de bits del estado
controller.active         # bool → TRUE si está activo

# Conexiones
controller.sensors        # Lista de sensores conectados
controller.actuators      # Lista de actuadores conectados

# Método principal
controller.update()      # Ejecutado cada frame cuando está activo
```

### 1.3 Flujo de Datos: Sensores → Controladores → Actuadores

```
┌─────────────┐
│   Sensores  │ ← Detectan eventos (teclado, colisión, timer, etc.)
└──────┬──────┘
       │ Pulso (TRUE/FALSE)
       ▼
┌─────────────────┐
│  Controladores  │ ← Evalúan lógica (AND, OR, Expression, Python)
└────────┬────────┘
         │ Pulso (si se cumple la condición)
         ▼
┌──────────────┐
│  Actuadores  │ ← Ejecutan acciones (mover, reproducir sonido, etc.)
└──────────────┘
```

---

## 2. Tipos de Controladores BGE - Referencia Completa

### 2.1 SCA_ANDController

**Fuente**: `source/gameengine/GameLogic/SCA_ANDController.cpp`

**Propiedades**:

```python
# No tiene propiedades únicas además de las base
# El comportamiento está definido por su tipo
```

**Comportamiento**:
- Actúa como una puerta lógica AND
- Se activa solo cuando **TODOS** los sensores conectados están en estado positivo
- Si algún sensor es FALSE, el controlador no se activa

**Ejemplo de uso BGE**:
```python
# AND Controller conectado a:
# - KeyboardSensor (tecla ESPACIO)
# - CollisionSensor (suelo)
# - PropertySensor (energía > 0)
# Solo se activa si TODOS son TRUE simultáneamente
```

**Traducción a Rust (ArchFlow)**:

```rust
use crate::sensors::{Sensor, SensorState};
use crate::actuators::Actuator;

/// Controlador AND: Se activa solo cuando todos los sensores están TRUE
pub struct AndController {
    pub id: u32,
    pub entity_id: EntityId,
    pub sensors: Vec<SensorId>,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32, // Máscara de estado del objeto
}

impl AndController {
    pub fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        // AND: Todos deben ser TRUE
        self.sensors.iter().all(|sensor_id| {
            sensor_states
                .get(sensor_id)
                .map(|state| state.is_positive())
                .unwrap_or(false)
        })
    }
    
    pub fn execute(&self, sensor_states: &HashMap<SensorId, SensorState>) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo de uso en ArchFlow
let and_controller = AndController {
    id: 1,
    entity_id: player_entity,
    sensors: vec![jump_key_sensor, ground_sensor, energy_sensor],
    actuators: vec![jump_actuator],
    priority: 0,
    state_mask: 0b0001,
};

// En el game loop
let active_actuators = and_controller.execute(&current_sensor_states);
for actuator in active_actuators {
    actuator.trigger();
}
```

**Optimizaciones 2026 aplicables**:
- **SIMD Evaluation**: Procesar múltiples estados de sensores en paralelo usando AVX2
- **Bitset filtering**: Usar bits para representar estados de sensores
- **Early exit**: Cortar evaluación al encontrar primer sensor FALSE

```rust
// Optimización con SIMD (procesa 8 sensores simultáneamente)
use std::simd::u8x4;

fn evaluate_and_simd(sensor_states: &[bool]) -> bool {
    // Empaquetar estados en u8x4 para procesamiento SIMD
    let chunks: Vec<u8x4> = sensor_states
        .chunks(4)
        .map(|chunk| {
            let packed = chunk.iter().enumerate().fold(0u8, |acc, (i, &state)| {
                acc | ((state as u8) << i)
            });
            u8x4::from_array([packed; 4])
        })
        .collect();
    
    // AND SIMD: todos los bytes deben ser 0xFF
    chunks.iter().all(|chunk| {
        chunk.reduce_and() == 0xFF
    })
}
```

---

### 2.2 SCA_ORController

**Fuente**: `source/gameengine/GameLogic/SCA_ORController.cpp`

**Propiedades**:
- No tiene propiedades únicas
- Hereda todas las propiedades base de SCA_IController

**Comportamiento**:
- Actúa como una puerta lógica OR
- Se activa cuando **AL MENOS UNO** de los sensores está en estado positivo
- Solo retorna FALSE si TODOS los sensores son FALSE

**Ejemplo de uso BGE**:
```python
# OR Controller conectado a:
# - KeyboardSensor (tecla W)
# - GamepadSensor (botón A)
# - AISensor (seguimiento automático activado)
# Se activa si CUALQUIERA es TRUE
```

**Traducción a Rust (ArchFlow)**:

```rust
/// Controlador OR: Se activa si al menos un sensor es TRUE
pub struct OrController {
    pub id: u32,
    pub entity_id: EntityId,
    pub sensors: Vec<SensorId>,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32,
}

impl OrController {
    pub fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        // OR: Al menos uno debe ser TRUE (early exit cuando encontramos uno)
        self.sensors.iter().any(|sensor_id| {
            sensor_states
                .get(sensor_id)
                .map(|state| state.is_positive())
                .unwrap_or(false)
        })
    }
    
    pub fn execute(&self, sensor_states: &HashMap<SensorId, SensorState>) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo: Mover personaje con multiple input methods
let movement_controller = OrController {
    id: 2,
    entity_id: player_entity,
    sensors: vec![keyboard_w_sensor, gamepad_a_sensor, ai_follow_sensor],
    actuators: vec![move_forward_actuator],
    priority: 0,
    state_mask: 0b0001,
};
```

**Optimizaciones 2026**:
- **Iterator::any**: Usa early exit nativamente (para en primer TRUE)
- **Branch prediction**: CPU predice correctamente patrones OR comunes
- **Cache locality**: Sensores evaluados en orden de frecuencia de TRUE

```rust
// Optimización: Ordenar sensores por frecuencia de TRUE
// para maximizar early exit hits
impl OrController {
    pub fn optimize_sensor_order(&mut self, stats: &HashMap<SensorId, f32>) {
        // stats: frecuencia de TRUE para cada sensor (0.0 - 1.0)
        self.sensors.sort_by(|a, b| {
            let freq_a = stats.get(a).unwrap_or(&0.5);
            let freq_b = stats.get(b).unwrap_or(&0.5);
            freq_b.partial_cmp(freq_a).unwrap() // Descendente
        });
    }
}
```

---

### 2.3 SCA_NANDController (NOT AND)

**Fuente**: `source/gameengine/GameLogic/SCA_NANDController.cpp`

**Comportamiento**:
- Actúa como la negación lógica de AND
- Se activa cuando **NO** todos los sensores son TRUE
- Equivalente a: `NOT (AND(sensores))`
- Matemáticamente universal: puede implementar cualquier función booleana

**Tabla de verdad**:
```
A | B | AND | NAND
-----------------
0 | 0 |  0  |  1
0 | 1 |  0  |  1
1 | 0 |  0  |  1
1 | 1 |  1  |  0
```

**Traducción a Rust**:

```rust
/// Controlador NAND: Se activa si NO todos los sensores son TRUE
pub struct NandController {
    pub id: u32,
    pub entity_id: EntityId,
    pub sensors: Vec<SensorId>,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32,
}

impl NandController {
    pub fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        // NAND: NOT (AND de todos)
        !self.sensors.iter().all(|sensor_id| {
            sensor_states
                .get(sensor_id)
                .map(|state| state.is_positive())
                .unwrap_or(false)
        })
    }
    
    pub fn execute(&self, sensor_states: &HashMap<SensorId, SensorState>) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo de uso: Animación por defecto
// Animación de idle se ejecuta a MENOS que se estén ejecutando
// animaciones específicas (movimiento, ataque, etc.)
let default_animation_controller = NandController {
    id: 3,
    entity_id: player_entity,
    sensors: vec![movement_anim_active, attack_anim_active],
    actuators: vec![idle_animation_actuator],
    priority: 100, // Baja prioridad (ejecutar después)
    state_mask: 0b0001,
};
```

---

### 2.4 SCA_NORController (NOT OR)

**Fuente**: `source/gameengine/GameLogic/SCA_NORController.cpp`

**Comportamiento**:
- Actúa como la negación lógica de OR
- Se activa solo cuando **TODOS** los sensores son FALSE
- Equivalente a: `NOT (OR(sensores))`
- Útil para condiciones "default" o "fallback"

**Tabla de verdad**:
```
A | B | OR | NOR
----------------
0 | 0 | 0  | 1
0 | 1 | 1  | 0
1 | 0 | 1  | 0
1 | 1 | 1  | 0
```

**Traducción a Rust**:

```rust
/// Controlador NOR: Se activa solo si todos los sensores son FALSE
pub struct NorController {
    pub id: u32,
    pub entity_id: EntityId,
    pub sensors: Vec<SensorId>,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32,
}

impl NorController {
    pub fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        // NOR: Ningún sensor debe ser TRUE
        !self.sensors.iter().any(|sensor_id| {
            sensor_states
                .get(sensor_id)
                .map(|state| state.is_positive())
                .unwrap_or(false)
        })
    }
    
    pub fn execute(&self, sensor_states: &HashMap<SensorId, SensorState>) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo: Sistema de alarma desactivado
// Solo se activa si no hay ningún sensor de peligro activo
let safety_controller = NorController {
    id: 4,
    entity_id: alarm_entity,
    sensors: vec![motion_sensor, smoke_sensor, glass_break_sensor],
    actuators: vec![disarm_alarm_actuator],
    priority: 0,
    state_mask: 0b0001,
};
```

---

### 2.5 SCA_XORController (Exclusive OR)

**Fuente**: `source/gameengine/GameLogic/SCA_XORController.cpp`

**Comportamiento**:
- Implementa lógica XOR (OR exclusivo)
- Para 2 entradas: TRUE si exactamente una es TRUE
- Para N entradas: TRUE si número impar de entradas son TRUE
- Útil para toggles y comportamientos alternantes

**Tabla de verdad**:
```
A | B | XOR
----------
0 | 0 |  0
0 | 1 |  1
1 | 0 |  1
1 | 1 |  0
```

**Traducción a Rust**:

```rust
/// Controlador XOR: Se activa si número impar de sensores son TRUE
pub struct XorController {
    pub id: u32,
    pub entity_id: EntityId,
    pub sensors: Vec<SensorId>,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32,
}

impl XorController {
    pub fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        // XOR: Contar TRUE, retorno TRUE si es impar
        let true_count = self.sensors.iter().filter(|sensor_id| {
            sensor_states
                .get(sensor_id)
                .map(|state| state.is_positive())
                .unwrap_or(false)
        }).count();
        
        true_count % 2 == 1
    }
    
    pub fn execute(&self, sensor_states: &HashMap<SensorId, SensorState>) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo: Toggle de pausa
// Se activa si se presiona P, pero solo si NO está ya pausado
let pause_toggle_controller = XorController {
    id: 5,
    entity_id: game_entity,
    sensors: vec![p_key_sensor, is_paused_sensor],
    actuators: vec![toggle_pause_actuator],
    priority: 0,
    state_mask: 0b0001,
};
```

**Optimización XOR con bitwise operations**:

```rust
// XOR puede implementarse eficientemente con XOR bitwise
// Empaquetar estados en bits de un entero
fn evaluate_xor_bitwise(sensor_states: &[bool]) -> bool {
    let packed: u64 = sensor_states
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &state)| {
            acc | ((state as u64) << i)
        });
    
    // XOR de todos los bits = TRUE si número impar de bits son TRUE
    packed.count_ones() % 2 == 1
}
```

---

### 2.6 SCA_ExpressionController

**Fuente**: `source/gameengine/GameLogic/SCA_ExpressionController.cpp`

**Propiedades**:

```python
expression    # String con la expresión a evaluar
```

**Comportamiento**:
- Evalúa una expresión similar a Python
- Soporta operadores matemáticos: +, -, *, /
- Soporta comparaciones: ==, !=, <, >, <=, >=
- Soporta lógica booleana: AND, OR, NOT
- Soporta operador condicional: if(condition, true_value, false_value)

**Sintaxis soportada**:

```
# Matemáticas
coins > 20
health / max_health > 0.5

# Lógica
Key_Inserted AND Fuel
NOT (is_falling)

# Condicional
if(health > 50, 1, 0)

# Complejo
(coins > 20) OR (has_key AND door_unlocked)
```

**Traducción a Rust (ArchFlow)**:

```rust
use std::collections::HashMap;

/// Tipos de valores soportados en expresiones
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl ExpressionValue {
    pub fn as_bool(&self) -> bool {
        match self {
            ExpressionValue::Boolean(b) => *b,
            ExpressionValue::Integer(i) => *i != 0,
            ExpressionValue::Float(f) => *f != 0.0,
            ExpressionValue::String(s) => !s.is_empty(),
        }
    }
    
    pub fn as_number(&self) -> f64 {
        match self {
            ExpressionValue::Integer(i) => *i as f64,
            ExpressionValue::Float(f) => *f,
            _ => 0.0,
        }
    }
}

/// AST de expresión
#[derive(Debug, Clone)]
pub enum Expression {
    // Literales
    Literal(ExpressionValue),
    Property(String), // coins, health, etc.
    
    // Operadores binarios
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    // Operadores unarios
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    
    // Condicional
    If {
        condition: Box<Expression>,
        true_value: Box<Expression>,
        false_value: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    // Matemáticos
    Add, Sub, Mul, Div,
    
    // Comparación
    Eq, Ne, Lt, Gt, Le, Ge,
    
    // Lógicos
    And, Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// Controlador de expresión
pub struct ExpressionController {
    pub id: u32,
    pub entity_id: EntityId,
    pub expression: Expression,
    pub actuators: Vec<ActuatorId>,
    pub priority: u32,
    pub state_mask: u32,
}

impl ExpressionController {
    /// Evalúa la expresión en el contexto dado
    pub fn evaluate(
        &self,
        sensor_states: &HashMap<SensorId, SensorState>,
        properties: &HashMap<String, ExpressionValue>,
    ) -> bool {
        self.eval_expr(&self.expression, sensor_states, properties)
            .as_bool()
    }
    
    fn eval_expr(
        &self,
        expr: &Expression,
        sensor_states: &HashMap<SensorId, SensorState>,
        properties: &HashMap<String, ExpressionValue>,
    ) -> ExpressionValue {
        match expr {
            Expression::Literal(v) => v.clone(),
            
            Expression::Property(name) => {
                properties.get(name).cloned().unwrap_or(ExpressionValue::Integer(0))
            }
            
            Expression::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, sensor_states, properties);
                let r = self.eval_expr(right, sensor_states, properties);
                self.apply_binary_op(*op, l, r)
            }
            
            Expression::UnaryOp { op, operand } => {
                let v = self.eval_expr(operand, sensor_states, properties);
                self.apply_unary_op(*op, v)
            }
            
            Expression::If { condition, true_value, false_value } => {
                if self.eval_expr(condition, sensor_states, properties).as_bool() {
                    self.eval_expr(true_value, sensor_states, properties)
                } else {
                    self.eval_expr(false_value, sensor_states, properties)
                }
            }
        }
    }
    
    fn apply_binary_op(&self, op: BinaryOperator, left: ExpressionValue, right: ExpressionValue) -> ExpressionValue {
        match op {
            // Matemáticos
            BinaryOperator::Add => ExpressionValue::Float(left.as_number() + right.as_number()),
            BinaryOperator::Sub => ExpressionValue::Float(left.as_number() - right.as_number()),
            BinaryOperator::Mul => ExpressionValue::Float(left.as_number() * right.as_number()),
            BinaryOperator::Div => ExpressionValue::Float(left.as_number() / right.as_number()),
            
            // Comparación
            BinaryOperator::Eq => ExpressionValue::Boolean(left == right),
            BinaryOperator::Ne => ExpressionValue::Boolean(left != right),
            BinaryOperator::Lt => ExpressionValue::Boolean(left.as_number() < right.as_number()),
            BinaryOperator::Gt => ExpressionValue::Boolean(left.as_number() > right.as_number()),
            BinaryOperator::Le => ExpressionValue::Boolean(left.as_number() <= right.as_number()),
            BinaryOperator::Ge => ExpressionValue::Boolean(left.as_number() >= right.as_number()),
            
            // Lógicos
            BinaryOperator::And => ExpressionValue::Boolean(left.as_bool() && right.as_bool()),
            BinaryOperator::Or => ExpressionValue::Boolean(left.as_bool() || right.as_bool()),
        }
    }
    
    fn apply_unary_op(&self, op: UnaryOperator, operand: ExpressionValue) -> ExpressionValue {
        match op {
            UnaryOperator::Not => ExpressionValue::Boolean(!operand.as_bool()),
            UnaryOperator::Negate => ExpressionValue::Float(-operand.as_number()),
        }
    }
    
    pub fn execute(
        &self,
        sensor_states: &HashMap<SensorId, SensorState>,
        properties: &HashMap<String, ExpressionValue>,
    ) -> Vec<ActuatorId> {
        if self.evaluate(sensor_states, properties) {
            self.actuators.clone()
        } else {
            vec![]
        }
    }
}

// Ejemplo de uso
// Crear expresión: coins > 20 OR has_key
let expr = Expression::BinaryOp {
    op: BinaryOperator::Or,
    left: Box::new(Expression::BinaryOp {
        op: BinaryOperator::Gt,
        left: Box::new(Expression::Property("coins".to_string())),
        right: Box::new(Expression::Literal(ExpressionValue::Integer(20))),
    }),
    right: Box::new(Expression::Property("has_key".to_string())),
};

let expr_controller = ExpressionController {
    id: 6,
    entity_id: door_entity,
    expression: expr,
    actuators: vec![open_door_actuator],
    priority: 0,
    state_mask: 0b0001,
};
```

**Parser de expresiones**:

```rust
use std::iter::Peekable;
use std::str::Chars;

/// Parser simple de expresiones estilo BGE
pub struct ExpressionParser {
    input: Peekable<Chars<'static>>,
}

impl ExpressionParser {
    pub fn parse(input: &str) -> Result<Expression, String> {
        let mut parser = ExpressionParser {
            input: input.to_string().into_chars().peekable(),
        };
        parser.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and()?;
        
        while let Some(&c) = self.input.peek() {
            if c == 'O' && self.peek_starts_with("OR") {
                self.consume("OR")?;
                let right = self.parse_and()?;
                left = Expression::BinaryOp {
                    op: BinaryOperator::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;
        
        while let Some(&c) = self.input.peek() {
            if c == 'A' && self.peek_starts_with("AND") {
                self.consume("AND")?;
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp {
                    op: BinaryOperator::And,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let left = self.parse_additive()?;
        
        // Verificar operadores de comparación
        if self.consume_if("==")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Eq,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        if self.consume_if(">=")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Ge,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        if self.consume_if("<=")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Le,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        if self.consume_if(">")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Gt,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        if self.consume_if("<")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Lt,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        if self.consume_if("!=")? {
            let right = self.parse_additive()?;
            return Ok(Expression::BinaryOp {
                op: BinaryOperator::Ne,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        
        Ok(left)
    }
    
    // ... más métodos de parsing
}
```

---

### 2.7 SCA_PythonController

**Fuente**: `source/gameengine/GameLogic/SCA_PythonController.cpp`

**Propiedades**:

```python
mode        # "Script" o "Module"
script      # Texto del script (modo Script)
module      # Nombre del módulo (modo Module)
```

**Comportamiento**:
- Ejecuta código Python arbitrario
- Tiene acceso completo a la API de BGE
- Puede acceder a sensores, actuadores, objetos, escenas, etc.
- Permite lógica arbitrariamente compleja

**API disponible en Python Controller**:

```python
import bge
from bge import logic

# Obtener controlador actual
cont = logic.getCurrentController()
owner = cont.owner  # Objeto dueño del controlador

# Acceder a sensores
sensor = cont.sensors["sensor_name"]
if sensor.positive:
    # Sensor activado

# Acceder a actuadores
actuator = cont.actuators["actuator_name"]
cont.activate(actuator)  # Activar
cont.deactivate(actuator)  # Desactivar

# Acceder a la escena
scene = logic.getCurrentScene()
objects = scene.objects
other_object = objects["OtherObject"]

# Modificar propiedades
owner["property_name"] = value
```

**Traducción a Rust (ArchFlow)**:

```rust
/// Controlador con script (similar a Python Controller)
/// En Rust, usamos closures o traits para lógica personalizada
pub trait ScriptController: Send + Sync {
    fn execute(
        &mut self,
        entity_id: EntityId,
        sensor_states: &HashMap<SensorId, SensorState>,
        world: &mut World,
    ) -> Vec<ActuatorCommand>;
}

/// Comando generado por un script controller
pub enum ActuatorCommand {
    Activate(ActuatorId),
    Deactivate(ActuatorId),
    SetParam(ActuatorId, String, ExpressionValue),
}

/// Contenedor de script controller
pub struct ScriptControllerContainer {
    pub id: u32,
    pub entity_id: EntityId,
    pub script: Box<dyn ScriptController>,
    pub priority: u32,
    pub state_mask: u32,
}

// Ejemplo: Script para movimiento de personaje
struct PlayerMovementScript;

impl ScriptController for PlayerMovementScript {
    fn execute(
        &mut self,
        entity_id: EntityId,
        sensor_states: &HashMap<SensorId, SensorState>,
        world: &mut World,
    ) -> Vec<ActuatorCommand> {
        let mut commands = Vec::new();
        
        // Obtener estado de sensores
        let w_pressed = sensor_states
            .get(&SensorId::new(entity_id, "w_key"))
            .map(|s| s.is_positive())
            .unwrap_or(false);
            
        let is_grounded = sensor_states
            .get(&SensorId::new(entity_id, "ground_contact"))
            .map(|s| s.is_positive())
            .unwrap_or(false);
        
        // Lógica arbitrariamente compleja
        if w_pressed && is_grounded {
            // Activar movimiento
            commands.push(ActuatorCommand::SetParam(
                ActuatorId::new(entity_id, "movement"),
                "velocity".to_string(),
                ExpressionValue::Float(10.0),
            ));
        }
        
        // Más lógica...
        
        commands
    }
}

// Ejemplo de uso
let player_script = ScriptControllerContainer {
    id: 7,
    entity_id: player_entity,
    script: Box::new(PlayerMovementScript),
    priority: 0,
    state_mask: 0b0001,
};

// En el game loop
let commands = player_script.script.execute(
    player_entity,
    &current_sensor_states,
    &mut world,
);
for command in commands {
    match command {
        ActuatorCommand::Activate(id) => activate_actuator(id),
        ActuatorCommand::Deactivate(id) => deactivate_actuator(id),
        ActuatorCommand::SetParam(id, param, value) => set_actuator_param(id, param, value),
    }
}
```

**Script System con Lua/Wasm**:

```rust
// Alternativa: Integrar Lua o WebAssembly para scripting
// permitiendo hot-reload y lógica más flexible

use mlua::{Lua, Function};

pub struct LuaScriptController {
    lua: Lua,
    script_name: String,
}

impl LuaScriptController {
    pub fn new(script: &str) -> Result<Self, String> {
        let lua = Lua::new();
        lua.load(script, script_name).exec()
            .map_err(|e| e.to_string())?;
        Ok(LuaScriptController {
            lua,
            script_name: script_name.to_string(),
        })
    }
}

impl ScriptController for LuaScriptController {
    fn execute(
        &mut self,
        entity_id: EntityId,
        sensor_states: &HashMap<SensorId, SensorState>,
        world: &mut World,
    ) -> Vec<ActuatorCommand> {
        // Llamar a función Lua con contexto
        self.lua.globals().set("entity_id", entity_id.to_bits()).unwrap();
        
        // Pasar estados de sensores como tabla Lua
        let sensor_table = self.lua.create_table().unwrap();
        for (id, state) in sensor_states {
            sensor_table.set(id.to_string(), state.is_positive()).unwrap();
        }
        self.lua.globals().set("sensors", sensor_table).unwrap();
        
        // Ejecutar función update
        let update_func: Function = self.lua
            .globals()
            .get("update")
            .unwrap();
            
        let result: Vec<(String, String, ExpressionValue)> = update_func
            .call(())
            .unwrap();
        
        // Convertir resultado a ActuatorCommands
        result.into_iter().map(|(actuator, action, value)| {
            ActuatorCommand::SetParam(
                ActuatorId::from_string(&actuator),
                action,
                value,
            )
        }).collect()
    }
}
```

---

## 3. Sistema de Estados (State Machine)

### 3.1 Máscaras de Estado en BGE

BGE usa un sistema de **máscaras de bits** para gestionar múltiples estados:

```python
# Máscara de estado del objeto
obj.state = 0b0101  # Estados 1 y 3 activos

# Controladores también tienen máscaras
controller.state = 0b0001  # Solo se ejecuta en estado 1

# Estados visibles en Blender
# State 1: Idle
# State 2: Walking
# State 3: Running
# State 4: Jumping
# State 5: Attacking
# ...
# State 30: Max
```

### 3.2 Traducción a Rust (ArchFlow)

```rust
/// Máscara de estado usando bits (hasta 30 estados)
pub type StateMask = u32;

/// Controlador con sistema de estados
pub trait StatefulController {
    fn state_mask(&self) -> StateMask;
    fn should_execute(&self, entity_state: StateMask) -> bool {
        // Se ejecuta si la intersección de bits no es vacía
        self.state_mask() & entity_state != 0
    }
}

// Ejemplo: Estados de personaje
pub mod player_states {
    pub const IDLE: StateMask = 0b0001;
    pub const WALKING: StateMask = 0b0010;
    pub const RUNNING: StateMask = 0b0100;
    pub const JUMPING: StateMask = 0b1000;
    pub const ATTACKING: StateMask = 0b0001_0000;
    // ... hasta 30 estados
}

// Controlador que solo se ejecuta en estado IDLE
pub struct IdleAnimationController {
    pub state_mask: StateMask,
    pub actuators: Vec<ActuatorId>,
}

impl StatefulController for IdleAnimationController {
    fn state_mask(&self) -> StateMask {
        self.state_mask
    }
}

// En el game loop
let entity_state = player_states::IDLE;
for controller in &controllers {
    if controller.should_execute(entity_state) {
        controller.execute(&sensor_states);
    }
}
```

**Optimización con Sparse Sets para Estados**:

```rust
// Sparse set: O(1) lookup + iteración eficiente
use std::collections::HashSet;

pub struct StateMachine {
    pub current_states: HashSet<StateMask>,
    pub transitions: HashMap<(StateMask, SensorId), StateMask>,
}

impl StateMachine {
    pub fn transition(&mut self, event: SensorId) -> Option<StateMask> {
        for &state in &self.current_states {
            let key = (state, event);
            if let Some(&new_state) = self.transitions.get(&key) {
                self.current_states.remove(&state);
                self.current_states.insert(new_state);
                return Some(new_state);
            }
        }
        None
    }
}
```

---

## 4. Sistema de Prioridades

### 4.1 Prioridades en BGE

```python
# Prioridad 0 = más alta (ejecuta primero)
# Prioridad 100 = más baja (ejecuta al final)

controller.priority = 0  # Input handling
controller.priority = 10  # Physics
controller.priority = 20  # Animation
controller.priority = 100  # Default behaviors
```

### 4.2 Traducción a Rust con Priority Queue

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Controlador con prioridad (min-heap: menor número = mayor prioridad)
#[derive(Debug, Clone)]
pub struct PrioritizedController {
    pub priority: u32,
    pub controller: ControllerType,
    pub id: u32,
}

impl PartialEq for PrioritizedController {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PrioritizedController {}

impl PartialOrd for PrioritizedController {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedController {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invertir orden para min-heap
        other.priority.cmp(&self.priority)
    }
}

/// Sistema de ejecución con prioridades
pub struct ControllerScheduler {
    controllers: BinaryHeap<PrioritizedController>,
}

impl ControllerScheduler {
    pub fn new() -> Self {
        ControllerScheduler {
            controllers: BinaryHeap::new(),
        }
    }
    
    pub fn register(&mut self, controller: PrioritizedController) {
        self.controllers.push(controller);
    }
    
    pub fn execute_all(&mut self, sensor_states: &HashMap<SensorId, SensorState>) {
        let mut controllers_to_execute = Vec::new();
        
        // Extraer en orden de prioridad
        while let Some(controller) = self.controllers.pop() {
            controllers_to_execute.push(controller);
        }
        
        // Ejecutar en orden
        for controller in controllers_to_execute {
            // ... ejecutar controlador
            
            // Reinsertar para siguiente frame
            self.controllers.push(controller);
        }
    }
}
```

**Optimización: Bucket Sort para Prioridades**:

```rust
// Si el rango de prioridades es pequeño (0-100), usar bucket array
// para evitar overhead de heap

pub struct BucketScheduler {
    buckets: Vec<Vec<ControllerType>>,
    max_priority: usize,
}

impl BucketScheduler {
    pub fn new(max_priority: usize) -> Self {
        BucketScheduler {
            buckets: vec![Vec::new(); max_priority + 1],
            max_priority,
        }
    }
    
    pub fn register(&mut self, priority: usize, controller: ControllerType) {
        if priority <= self.max_priority {
            self.buckets[priority].push(controller);
        }
    }
    
    pub fn execute_all(&mut self, sensor_states: &HashMap<SensorId, SensorState>) {
        // Ejecutar en orden de prioridad (0 primero)
        for bucket in &mut self.buckets {
            for controller in bucket {
                controller.execute(sensor_states);
            }
        }
    }
}
```

---

## 5. Sistema de Wiring (Conexiones)

### 5.1 Wiring en BGE

En BGE, las conexiones se hacen visualmente:

```
┌────────┐     ┌───────────────┐     ┌──────────┐
│ Sensor │────▶│  Controller   │────▶│ Actuator │
└────────┘     └───────────────┘     └──────────┘
     ┌───────────────┐
     │  Controller   │────▶│ Actuator │
     └───────────────┘     └──────────┘
```

### 5.2 Traducción a Rust: Wiring Table

```rust
/// Tabla de conexiones (Wiring Table)
pub struct WiringTable {
    /// Mapa: Controller ID → Vec<Sensor IDs>
    pub controller_sensors: HashMap<ControllerId, Vec<SensorId>>,
    
    /// Mapa: Controller ID → Vec<Actuator IDs>
    pub controller_actuators: HashMap<ControllerId, Vec<ActuatorId>>,
    
    /// Mapa: Sensor ID → Vec<Controller IDs> (para broadcast eficiente)
    pub_sensor_controllers: HashMap<SensorId, Vec<ControllerId>>,
}

impl WiringTable {
    pub fn new() -> Self {
        WiringTable {
            controller_sensors: HashMap::new(),
            controller_actuators: HashMap::new(),
            sensor_controllers: HashMap::new(),
        }
    }
    
    /// Conectar sensor a controlador
    pub fn connect(&mut self, sensor: SensorId, controller: ControllerId) {
        self.controller_sensors
            .entry(controller)
            .or_insert_with(Vec::new)
            .push(sensor);
            
        self.sensor_controllers
            .entry(sensor)
            .or_insert_with(Vec::new)
            .push(controller);
    }
    
    /// Desconectar sensor de controlador
    pub fn disconnect(&mut self, sensor: SensorId, controller: ControllerId) {
        if let Some(sensors) = self.controller_sensors.get_mut(&controller) {
            sensors.retain(|s| s != &sensor);
        }
        
        if let Some(controllers) = self.sensor_controllers.get_mut(&sensor) {
            controllers.retain(|c| c != &controller);
        }
    }
    
    /// Obtener controladores conectados a un sensor
    pub fn get_controllers_for_sensor(&self, sensor: SensorId) -> &[ControllerId] {
        self.sensor_controllers
            .get(&sensor)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    
    /// Obtener sensores conectados a un controlador
    pub fn get_sensors_for_controller(&self, controller: ControllerId) -> &[SensorId] {
        self.controller_sensors
            .get(&controller)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    
    /// Obtener actuadores conectados a un controlador
    pub fn get_actuators_for_controller(&self, controller: ControllerId) -> &[ActuatorId] {
        self.controller_actuators
            .get(&controller)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

// Ejemplo de uso
let mut wiring = WiringTable::new();

// Conectar: keyboard sensor → AND controller → jump actuator
wiring.connect(keyboard_sensor_id, and_controller_id);
wiring.connect(ground_sensor_id, and_controller_id);
wiring.connect(energy_sensor_id, and_controller_id);
wiring.set_actuators(and_controller_id, vec![jump_actuator_id]);

// En el game loop: cuando keyboard sensor se activa
for controller_id in wiring.get_controllers_for_sensor(keyboard_sensor_id) {
    let controller = get_controller(*controller_id);
    let sensor_states = collect_sensor_states(controller, &wiring);
    
    if controller.evaluate(&sensor_states) {
        for actuator_id in wiring.get_actuators_for_controller(*controller_id) {
            trigger_actuator(*actuator_id);
        }
    }
}
```

---

## 6. Mapeo BGE → ArchFlow

### 6.1 Arquitectura BGE

```
┌─────────────┐
│   Sensores  │
└──────┬──────┘
       │ Pulso
       ▼
┌─────────────────┐     ┌──────────────┐
│  Controladores  │────▶│  Actuadores  │
│  (AND/OR/etc)   │     └──────────────┘
└─────────────────┘
```

### 6.2 Arquitectura ArchFlow

```
┌─────────────────┐
│   Hardware/JS   │ ← Input events (teclado, mouse)
└────────┬────────┘
         │ SharedArrayBuffer
         ▼
┌─────────────────┐
│   PulseBus      │ ← Event streaming
└────────┬────────┘
         │ Pulse (16 bytes)
         ▼
┌─────────────────────────────┐
│   Wiring Table              │ ← Route events
└────────┬────────────────────┘
         │ Filtered pulses
         ▼
┌─────────────────────────────┐
│   Controller Layer          │
│   - Rust native controllers │
│   - Lua/Wasm scripts        │
│   - Expression evaluator    │
└────────┬────────────────────┘
         │ Actuation commands
         ▼
┌─────────────────────────────┐
│   Actuator Dispatcher       │
└────────┬────────────────────┘
         │ Execute
         ▼
┌─────────────────────────────┐
│   ECS World (SoA)           │
└─────────────────────────────┘
```

### 6.3 Diferencias Clave

| Aspecto | BGE | ArchFlow |
|---------|-----|-----------|
| **Ejecución** | Python interpreter | Rust native + Wasm |
| **Memoria** | AoS (Array of Structures) | SoA (Structure of Arrays) |
| **Eventos** | Poll-based | Push-based (PulseBus) |
| **Sincronización** | GIL | Lock-free + atomics |
| **Extensibilidad** | Python scripts | Lua/Wasm hot-reload |
| **Prioridades** | Heapsort | Bucket sort O(1) |
| **Estados** | Bitmask | Sparse set + bitmask |

---

## 7. Patrones de Diseño Recomendados

### 7.1 Controller Composition

```rust
/// Combinar múltiples controladores en uno
pub trait Controller {
    fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool;
    fn execute(&self) -> Vec<ActuatorCommand>;
}

/// Compose controllers con AND
pub struct AndCompose<A, B> {
    pub a: A,
    pub b: B,
}

impl<A: Controller, B: Controller> Controller for AndCompose<A, B> {
    fn evaluate(&self, sensor_states: &HashMap<SensorId, SensorState>) -> bool {
        self.a.evaluate(sensor_states) && self.b.evaluate(sensor_states)
    }
    
    fn execute(&self) -> Vec<ActuatorCommand> {
        let mut cmds = self.a.execute();
        cmds.extend(self.b.execute());
        cmds
    }
}

// Ejemplo de uso
let complex_controller = AndCompose {
    a: ground_check_controller,
    b: energy_check_controller,
};
```

### 7.2 Chain of Responsibility

```rust
/// Cadena de controladores con fallback
pub struct ControllerChain {
    pub controllers: Vec<Box<dyn Controller>>,
}

impl ControllerChain {
    pub fn add(&mut self, controller: Box<dyn Controller>) {
        self.controllers.push(controller);
    }
    
    pub fn execute_first_match(
        &self,
        sensor_states: &HashMap<SensorId, SensorState>,
    ) -> Option<Vec<ActuatorCommand>> {
        for controller in &self.controllers {
            if controller.evaluate(sensor_states) {
                return Some(controller.execute());
            }
        }
        None
    }
}
```

### 7.3 Observer Pattern para Sensores

```rust
/// Observador de cambios en sensores
pub trait SensorObserver {
    fn on_sensor_change(&mut self, sensor: SensorId, old_state: SensorState, new_state: SensorState);
}

/// Sensor con observers
pub struct ObservableSensor {
    pub id: SensorId,
    pub state: SensorState,
    pub observers: Vec<Box<dyn SensorObserver>>,
}

impl ObservableSensor {
    pub fn set_state(&mut self, new_state: SensorState) {
        let old_state = self.state.clone();
        self.state = new_state;
        
        for observer in &mut self.observers {
            observer.on_sensor_change(self.id, old_state.clone(), new_state.clone());
        }
    }
}
```

---

## 8. Performance y Optimizaciones

### 8.1 Caché de Estados de Sensores

```rust
/// Caché de estados para evitar re-evaluaciones
pub struct SensorStateCache {
    cache: HashMap<SensorId, (SensorState, u32)>, // estado + frame de validación
    current_frame: u32,
}

impl SensorStateCache {
    pub fn new() -> Self {
        SensorStateCache {
            cache: HashMap::new(),
            current_frame: 0,
        }
    }
    
    pub fn get_or_compute<F>(&mut self, sensor: SensorId, compute: F) -> SensorState
    where
        F: FnOnce() -> SensorState,
    {
        if let Some((state, frame)) = self.cache.get(&sensor) {
            if *frame == self.current_frame {
                return state.clone();
            }
        }
        
        let state = compute();
        self.cache.insert(sensor, (state.clone(), self.current_frame));
        state
    }
    
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
    }
}
```

### 8.2 Batch Processing de Controladores

```rust
/// Procesar múltiples controladores en paralelo usando rayon
use rayon::prelude::*;

pub fn process_controllers_parallel(
    controllers: &[PrioritizedController],
    sensor_states: &HashMap<SensorId, SensorState>,
) -> Vec<Vec<ActuatorCommand>> {
    controllers
        .par_iter() // Iteración en paralelo
        .map(|controller| {
            if controller.should_execute() {
                controller.execute(sensor_states)
            } else {
                vec![]
            }
        })
        .collect()
}
```

### 8.3 SIMD para Evaluación de Puertas Lógicas

```rust
/// Evaluar múltiples AND/OR controllers usando SIMD
use std::simd::{u8x32, u8x32};

pub fn evaluate_and_batch(
    sensor_states: &[&[bool]], // Cada sub-slice es un conjunto de sensores
) -> Vec<bool> {
    // Asumir máximo 32 sensores por controlador
    let mut results = Vec::with_capacity(sensor_states.len());
    
    for &states in sensor_states {
        // Empaquetar estados en u8x32
        let packed: u8x32 = states
            .iter()
            .enumerate()
            .fold(u8x32::splat(0), |mut acc, (i, &state)| {
                acc[i] = state as u8;
                acc
            });
        
        // AND SIMD: todos deben ser 1
        let all_true = packed.reduce_and() == 1;
        results.push(all_true);
    }
    
    results
}
```

---

## 9. Ejemplo Completo: Player Controller

```rust
use std::collections::HashMap;

/// Ejemplo completo: Controlador de personaje para un juego de plataformas
pub struct PlayerControllerExample {
    // Sensores
    pub keyboard_w: SensorId,
    pub keyboard_a: SensorId,
    pub keyboard_s: SensorId,
    pub keyboard_d: SensorId,
    pub keyboard_space: SensorId,
    pub ground_sensor: SensorId,
    pub energy_sensor: SensorId,
    
    // Controladores
    pub movement_or: OrController,
    pub jump_and: AndController,
    pub animation_nand: NandController,
    
    // Actuadores
    pub move_forward: ActuatorId,
    pub move_backward: ActuatorId,
    pub move_left: ActuatorId,
    pub move_right: ActuatorId,
    pub jump: ActuatorId,
    pub idle_anim: ActuatorId,
    pub walk_anim: ActuatorId,
    pub jump_anim: ActuatorId,
    
    // Wiring table
    pub wiring: WiringTable,
}

impl PlayerControllerExample {
    pub fn new() -> Self {
        let mut wiring = WiringTable::new();
        
        // IDs (ejemplo)
        let player_id = EntityId::new(1);
        
        let keyboard_w = SensorId::new(player_id, "keyboard_w");
        let keyboard_a = SensorId::new(player_id, "keyboard_a");
        let keyboard_s = SensorId::new(player_id, "keyboard_s");
        let keyboard_d = SensorId::new(player_id, "keyboard_d");
        let keyboard_space = SensorId::new(player_id, "keyboard_space");
        let ground_sensor = SensorId::new(player_id, "ground_contact");
        let energy_sensor = SensorId::new(player_id, "energy_check");
        
        let move_forward = ActuatorId::new(player_id, "move_forward");
        let move_backward = ActuatorId::new(player_id, "move_backward");
        let move_left = ActuatorId::new(player_id, "move_left");
        let move_right = ActuatorId::new(player_id, "move_right");
        let jump = ActuatorId::new(player_id, "jump");
        let idle_anim = ActuatorId::new(player_id, "idle_anim");
        let walk_anim = ActuatorId::new(player_id, "walk_anim");
        let jump_anim = ActuatorId::new(player_id, "jump_anim");
        
        // Controlador de movimiento: OR entre WASD
        let movement_or = OrController {
            id: 1,
            entity_id: player_id,
            sensors: vec![keyboard_w, keyboard_a, keyboard_s, keyboard_d],
            actuators: vec![],
            priority: 0,
            state_mask: player_states::IDLE | player_states::WALKING,
        };
        
        // Controlador de salto: AND entre espacio, suelo Y energía
        let jump_and = AndController {
            id: 2,
            entity_id: player_id,
            sensors: vec![keyboard_space, ground_sensor, energy_sensor],
            actuators: vec![jump],
            priority: 0,
            state_mask: 0xFFFF, // Cualquier estado
        };
        
        // Controlador de animación idle: NAND de movimientos
        let animation_nand = NandController {
            id: 3,
            entity_id: player_id,
            sensors: vec![keyboard_w, keyboard_a, keyboard_s, keyboard_d],
            actuators: vec![idle_anim],
            priority: 100,
            state_mask: player_states::IDLE,
        };
        
        // Conectar wiring
        let movement_or_id = ControllerId::new(player_id, "movement_or");
        let jump_and_id = ControllerId::new(player_id, "jump_and");
        
        wiring.connect(keyboard_w, movement_or_id);
        wiring.connect(keyboard_a, movement_or_id);
        wiring.connect(keyboard_s, movement_or_id);
        wiring.connect(keyboard_d, movement_or_id);
        
        wiring.connect(keyboard_space, jump_and_id);
        wiring.connect(ground_sensor, jump_and_id);
        wiring.connect(energy_sensor, jump_and_id);
        
        wiring.set_actuators(movement_or_id, vec![walk_anim]);
        wiring.set_actuators(jump_and_id, vec![jump]);
        
        PlayerControllerExample {
            keyboard_w,
            keyboard_a,
            keyboard_s,
            keyboard_d,
            keyboard_space,
            ground_sensor,
            energy_sensor,
            movement_or,
            jump_and,
            animation_nand,
            move_forward,
            move_backward,
            move_left,
            move_right,
            jump,
            idle_anim,
            walk_anim,
            jump_anim,
            wiring,
        }
    }
    
    pub fn update(&self, sensor_states: &HashMap<SensorId, SensorState>) {
        // Evaluar controladores
        let movement_active = self.movement_or.execute(sensor_states);
        let jump_active = self.jump_and.execute(sensor_states);
        let idle_active = self.animation_nand.execute(sensor_states);
        
        // Ejecutar actuadores basado en estados
        for actuator_id in movement_active {
            match actuator_id {
                id if id == self.walk_anim => {
                    // Determinar dirección y activar movimiento
                    if sensor_states[&self.keyboard_w].is_positive() {
                        trigger_actuator(self.move_forward);
                    } else if sensor_states[&self.keyboard_s].is_positive() {
                        trigger_actuator(self.move_backward);
                    } else if sensor_states[&self.keyboard_a].is_positive() {
                        trigger_actuator(self.move_left);
                    } else if sensor_states[&self.keyboard_d].is_positive() {
                        trigger_actuator(self.move_right);
                    }
                    trigger_actuator(self.walk_anim);
                }
                _ => {}
            }
        }
        
        for actuator_id in jump_active {
            trigger_actuator(actuator_id);
        }
        
        for actuator_id in idle_active {
            trigger_actuator(actuator_id);
        }
    }
}
```

---

## 10. Referencias y Recursos

### 10.1 Source Code BGE

- **SCA_IController**: `source/gameengine/GameLogic/SCA_IController.cpp`
- **SCA_ANDController**: `source/gameengine/GameLogic/SCA_ANDController.cpp`
- **SCA_ORController**: `source/gameengine/GameLogic/SCA_ORController.cpp`
- **SCA_ExpressionController**: `source/gameengine/GameLogic/SCA_ExpressionController.cpp`
- **SCA_PythonController**: `source/gameengine/GameLogic/SCA_PythonController.cpp`

### 10.2 Documentación UPBGE

- **Logic Bricks Introduction**: https://upbge.org/docs/latest/manual/manual/logic_bricks/introduction.html
- **Controllers Reference**: https://upbge.org/docs/latest/manual/manual/logic_bricks/controllers/introduction.html
- **Expression Controller**: https://upbge.org/docs/latest/manual/manual/logic_bricks/controllers/types/expression.html
- **Python Controller**: https://upbge.org/docs/latest/manual/manual/logic_bricks/controllers/types/python.html

### 10.3 Crates Rust Relacionados

- **Bevy ECS**: `bevy_ecs` - Entity Component System
- **Behavior Trees**: `bevy_behave`, `bevior_tree` - Árboles de comportamiento
- **Lua Scripting**: `mlua` - Integración Lua
- **Wasm Scripting**: `wasmer`, `wasmtime` - WebAssembly runtime
- **Expression Parsing**: `pest`, `nom` - Parsers de expresiones

### 10.4 Patrones de Diseño

- **State Machine**: Typestate pattern en Rust
- **Observer**: Pub/sub con `tokio::sync::broadcast`
- **Chain of Responsibility**: Trait objects con `Box<dyn Controller>`
- **Strategy**: Generics + traits para controladores intercambiables

---

## 11. Conclusión

Los controladores de BGE representan una capa intermedia crítica entre sensores y actuadores, permitiendo lógica compleja sin programación explícita. En ArchFlow, esta funcionalidad se expande significativamente:

1. **Zero-cost abstractions**: Controladores como traits compile-time
2. **Scripting flexible**: Lua/Wasm para lógica dinámica
3. **Performance**: SIMD, SoA, sparse sets para 100K+ entidades
4. **Type safety**: Errores de lógica detectados en compilación
5. **Hot-reload**: Scripts recargables en runtime

El sistema de controladores de ArchFlow mantiene la filosofía de BGE (lógica visual + scripting potente) mientras añade las ventajas de Rust (seguridad, performance, modern tooling).

---

**Fin del documento**
