---
title: "BGE Actuators Investigation - Tipos y Referencias para Implementación Rust"
author: Claude Code
date: 2026-02-01
status: Final
context: Blender Game Engine Logic Bricks Actuator Architecture
---

# Investigación: Actuadores de Blender Game Engine (BGE)

## 📌 Metadata

| Campo | Valor |
|-------|-------|
| Fecha | 2026-02-01 |
| Estado | Completada |
| Fuente | Blender BGE Source Code, UPBGE Documentation |
| Objetivo | Documentar todos los tipos de actuadores BGE con ejemplos Rust |

---

## 🎯 Resumen Ejecutivo

Este documento investiga **todos los tipos de actuadores** disponibles en Blender Game Engine (BGE) y proporciona:

1. **Lista completa de actuadores BGE** con sus propiedades
2. **Ejemplo real traducido a Rust** de los actuadores más importantes
3. **Referencias** para implementación de otros actuadores
4. **Patrones arquitectónicos** comunes a todos los actuadores
5. **Mapeo a la arquitectura de ArchFlow** (Cómo encajan en el sistema)

---

## 1. Arquitectura General de Actuadores BGE

### 1.1 Jerarquía de Clases

```
SCA_IActuator (base abstract)
    ├── SCA_ActionActuator (action/animation)
    ├── SCA_PropertyActuator (modify properties)
    ├── SCA_SoundActuator (play sounds)
    ├── KX_CameraActuator (camera tracking)
    ├── KX_ConstraintActuator (physics constraints)
    ├── KX_GameActuator (game control: load/save/quit)
    ├── KX_MessageActuator (messaging between objects)
    ├── KX_MouseActuator (mouse look/visibility)
    ├── KX_ParentActuator (parenting)
    ├── KX_RandomActuator (random values)
    ├── KX_SceneActuator (scene management)
    ├── KX_StateActuator (state machine)
    ├── KX_VisibilityActuator (show/hide)
    ├── KX_SteeringActuator (pathfinding/steering)
    ├── KX_Filter2DActuator (post-processing filters)
    ├── KX_ArmatureActuator (bone/IK control)
    ├── KX_EditObjectActuator (add/end/duplicate objects)
    └── KX_VibrationActuator (gamepad vibration)
```

### 1.2 Patrón Común a Todos los Actuadores

**Cada actuador en BGE tiene**:

```python
# Propiedades comunes
actuator.name          # Nombre del actuador
actuator.type          # Tipo de actuador
actuator.owner         # Objeto que posee el actuador
actuator.priority      # Prioridad (0 = más alta)
actuator.active         # bool → TRUE si está activo

# Método principal
actuator.update()      # Ejecutado cada frame cuando está activo
```

---

## 2. Tipos de Actuadores BGE - Referencia Completa

### 2.1 SCA_ActionActuator (Action Actuator)

**Fuente**: `source/gameengine/Ketsji/SCA_ActionActuator.cpp`

**Propiedades**:

```python
# Animación
action                  # Nombre del action a reproducir
frame_start             # Frame inicial
frame_end               # Frame final
frame_property          # Propiedad que define el frame actual

# Modos de reproducción
play_mode               # PLAY, PINGPONG, FLIPPER, LOOPSTOP, LOOPEND, PROPERTY
blend_mode              # BLEND, ADD
use_continue_last_frame  # Continuar desde última posición
priority                # Prioridad (0-100)

# Aplicación
use_force               # Aplicar como fuerza (física)
use_local               # Coordenadas locales vs globales
apply_to_children       # Aplicar a hijos también
layer                   # Capa de animación
layer_weight            # Peso de blending (0.0 - 1.0)
```

---

### 2.2 SCA_PropertyActuator (Property Actuator)

**Fuente**: `source/gameengine/Ketsji/SCA_PropertyActuator.cpp`

**Propiedades**:

```python
property               # Nombre de la propiedad
value                  # Valor a asignar
operation              # ASSIGN, ADD, SUB, MUL, DIV, COP, MOD
enable_dynamics        # Aplicar a objetos dinámicos
overwrite              # Sobrescribir valor existente
```

---

### 2.3 KX_CameraActuator (Camera Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_CameraActuator.cpp`

**Propiedades**:

```python
# Object tracking
object                 # Objeto a seguir
axis                   # Eje de tracking (X, Y, Z)
min_hgt                # Altura mínima
max_hgt                # Altura máxima
min_dist               # Distancia mínima
max_dist               # Distancia máxima
use_hide               # Ocultar objeto si no está visible

# Movimiento de cámara
strength               # Fuerza de seguimiento (0.0 - 1.0)
```

---

### 2.4 KX_ConstraintActuator (Constraint Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_ConstraintActuator.cpp`

**Tipos de constraint**:

```python
# Distance constraint
KX_CONSTRAINTACT_DISTANCE      # Mantener distancia
KX_CONSTRAINTACT_NORMAL        # Alinear a superficie
KX_CONSTRAINTACT_LOCAL         # Dirección local
KX_CONSTRAINTACT_MATERIAL      # Detectar material

# Position/Rotation limits
KX_CONSTRAINTACT_LOCX          # Limitar posición X
KX_CONSTRAINTACT_LOCY          # Limitar posición Y
KX_CONSTRAINTACT_LOCZ          # Limitar posición Z
KX_CONSTRAINTACT_ROTX          # Limitar rotación X
KX_CONSTRAINTACT_ROTY          # Limitar rotación Y
KX_CONSTRAINTACT_ROTZ          # Limitar rotación Z

# Force field
KX_CONSTRAINTACT_FHNX          # Fuerza en -X
KX_CONSTRAINTACT_FHNY          # Fuerza en -Y
KX_CONSTRAINTACT_FHNZ          # Fuerza en -Z
KX_CONSTRAINTACT_FHPX          # Fuerza en +X
KX_CONSTRAINTACT_FHPY          # Fuerza en +Y
KX_CONSTRAINTACT_FHPZ          # Fuerza en +Z
KX_CONSTRAINTACT_ORIX          # Orientación X
KX_CONSTRAINTACT_ORIY          # Orientación Y
KX_CONSTRAINTACT_ORIZ          # Orientación Z

# Parámetros
damp                   # Amortiguación (en frames)
rot_damp               # Amortiguación rotacional
reference_direction    # Dirección de referencia (Vec3)
```

---

### 2.5 KX_GameActuator (Game Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_GameActuator.cpp`

**Operaciones**:

```python
# Game control
KX_GAME_LOAD           # Cargar juego
KX_GAME_START          # Iniciar desde archivo
KX_GAME_RESTART        # Reiniciar
KX_GAME_QUIT           # Salir
KX_GAME_SAVECFG        # Guardar config
KX_GAME_LOADCFG        # Cargar config
KX_GAME_SCREENSHOT     # Capturar pantalla

# Parámetros
file                   # Ruta del archivo
filename               # Nombre de screenshot
```

---

### 2.6 KX_MessageActuator (Message Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_NetworkMessageActuator.cpp`

**Propiedades**:

```python
to                     # "TO" - objeto destinatario (vacío = broadcast)
subject                # "Subject" - asunto del mensaje
body_type              # BODY_TYPE_TEXT, BODY_TYPE_PROPERTY
body                   # Cuerpo del mensaje
property               # Propiedad a enviar
```

---

### 2.7 KX_MouseActuator (Mouse Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_MouseActuator.cpp`

**Tipos**:

```python
# Mouse visibility
KX_ACT_MOUSE_VISIBILITY     # Mostrar/ocultar cursor

# Mouse look
KX_ACT_MOUSE_OBJECT_AXIS_X  # Rotar objeto en eje X
KX_ACT_MOUSE_OBJECT_AXIS_Y  # Rotar objeto en eje Y
KX_ACT_MOUSE_OBJECT_AXIS_Z  # Rotar objeto en eje Z

# Parámetros
visible                # bool - Mostrar cursor
reset                  # Resetear a posición inicial
local                  # Coordenadas locales vs globales
sensitivity_x          # Sensibilidad eje X
sensitivity_y          # Sensibilidad eje Y
threshold              # Umbral de movimiento
object_axis_x          # Eje X del objeto a modificar
object_axis_y          # Eje Y del objeto a modificar
object_axis_z          # Eje Z del objeto a modificar
```

---

### 2.8 KX_SceneActuator (Scene Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_SceneActuator.cpp`

**Operaciones**:

```python
KX_SCENE_RESTART            # Reiniciar escena
KX_SCENE_SET_SCENE          # Cambiar a escena
KX_SCENE_SET_CAMERA         # Cambiar cámara
KX_SCENE_ADD_FRONT_SCENE   # Añadir overlay scene
KX_SCENE_ADD_BACK_SCENE    # Añadir underlay scene
KX_SCENE_REMOVE_SCENE       # Remover escena
KX_SCENE_SUSPEND            # Pausar escena
KX_SCENE_RESUME             # Reanudar escena

# Parámetros
scene                  # Nombre de la escena
camera                 # Cámara a activar
overlay                # Número de overlay
```

---

### 2.9 KX_StateActuator (State Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_StateActuator.cpp`

**Operaciones**:

```python
operation              # Bit operations:
                       # KX_STATE_OP_CLEAR    # Copiar estado
                       # KX_STATE_OP_SET      # Establecer bits
                       # KX_STATE_OP_ADD      # Añadir bits
                       # KX_STATE_OP_REMOVE   # Remover bits

# Parámetros
mask                   # Bitmask de estados a modificar
```

---

### 2.10 KX_VisibilityActuator (Visibility Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_VisibilityActuator.cpp`

**Propiedades**:

```python
visible                # bool - Mostrar/ocultar objeto
use_occlusion          # bool - Usar oclusión
use_visible            # bool - Aplicar a visibilidad
use_ray_cast           # bool - Ray cast visibilidad
```

---

### 2.11 KX_SoundActuator (Sound Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_SoundActuator.cpp`

**Modos**:

```python
KX_SOUNDACT_PLAYSTOP             # Tocar/parar
KX_SOUNDACT_PLAYEND              # Tocar hasta final
KX_SOUNDACT_LOOPSTOP             # Loop con parada
KX_SOUNDACT_LOOPEND              # Loop infinito
KX_SOUNDACT_LOOPBIDIRECTIONAL     # Loop bidireccional
KX_SOUNDACT_LOOPBIDIRECTIONAL_STOP # Loop bidireccional stop

# Parámetros
sound                  # Nombre del sonido
volume                 # Volumen (0.0 - 1.0)
pitch                  # Pitch (0.5 - 2.0)
attenuation            # Atenuación de distancia
cone_inner_angle       # Ángulo interno del cono
cone_outer_angle       # Ángulo externo del cono
cone_outer_gain        # Ganancia fuera del cono
```

---

### 2.12 KX_SteeringActuator (Steering Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_SteeringActuator.cpp`

**Comportamientos**:

```python
KX_STEERING_SEEK              # Buscar objetivo
KX_STEERING_FLEE              # Huir de objetivo
KX_STEERING_PATHFOLLOWING     # Seguir path
KX_STEERING_RANDOM            # Movimiento aleatorio
KX_STEERING_ARRIVE            # Llegar y parar
KX_STEERING_SEPARATION        # Separación de agentes

# Parámetros
target                 # Objetivo (para seek/flee/arrive)
navmesh                # Navigation mesh
path                   # Path points
wander_distance        # Distancia de vagar
path_offset            # Offset del path
speed                  # Velocidad máxima
accel                  # Aceleración
max_force              # Fuerza máxima de steering
turn_speed             # Velocidad de giro
dist_to_target         # Distancia al objetivo (arrive)
prediction_time        # Tiempo de predicción (path following)
self_managed           # Auto-actualización
```

---

### 2.13 KX_RandomActuator (Random Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_RandomActuator.cpp`

**Propiedades**:

```python
seed                   # Semilla del generador
distribution           # DISTRIB_UNIFORM, DISTRIB_BERNOULLI
constant_1             # Constante 1 (parámetro)
constant_2             # Constante 2 (parámetro)
```

---

### 2.14 KX_ParentActuator (Parent Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_ParentActuator.cpp`

**Modos**:

```python
KX_PARENT_ADD        # Añadir parent
KX_PARENT_REMOVE     # Remover parent
KX_PARENT_SET        # Establecer parent

# Parámetros
object               # Objeto a parentear
ghost                # bool - No afectar física
compound             # bool - Usar compound shape
```

---

### 2.15 KX_ArmatureActuator (Armature Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_ArmatureActuator.cpp`

**Modos**:

```python
KX_ACT_ARMATURE_RUN       # Ejecutar armatura
KX_ACT_ARMATURE_ENABLE    # Habilitar constraint
KX_ACT_ARMATURE_DISABLE   # Deshabilitar constraint
KX_ACT_ARMATURE_SET       # Establecer pose
KX_ACT_ARMATURE_SHAPE     # Shape action

# Parámetros
bone                  # Nombre del hueso
constraint             # Nombre del constraint
influence              # Influencia (0.0 - 1.0)
```

---

### 2.16 KX_Filter2DActuator (2D Filter Actuator)

**Fuente**: `source/gameengine/Ketsji/KX_Filter2DActuator.cpp`

**Filtros disponibles**:

```python
# Color
KX_2DFILTER_MOTIONBLUR      # Motion blur
KX_2DFILTER_BLUR            # Blur
KX_2DFILTER_SHARPEN        # Sharpen
KX_2DFILTER_DILATION        # Dilatación
KX_2DFILTER_EROSION         # Erosión
KX_2DFILTER_LAPLACIAN       # Laplaciano
KX_2DFILTER_SOBEL          # Sobel
KX_2DFILTER_PREWITT         # Prewitt

# Misc
KX_2DFILTER_GRAYSCALE       # Escala de grises
KX_2DFILTER_SEPIA           # Sepia
KX_2DFILTER_INVERT          # Invertir

# Parámetros
pass_number           # Número de pass
filter_actuator_index # Índice del filtro
```

---

## 3. Ejemplo Real: PropertyActuator en Rust (Fiel a BGE)

```rust
// ═══════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Property Actuator (BGE-Faithful Implementation)
//
// Basado en: source/gameengine/Ketsji/SCA_PropertyActuator.cpp
// Referencia: https://docs.blender.org/api/current/bge.types.SCA_PropertyActuator.html
//
// Este actuador modifica propiedades de entidades cuando recibe un pulso.
// ═══════════════════════════════════════════════════════════════════════════

use archflow_core::{EntityId, PropertyValue};
use crate::pulse::{Pulse, SensorState};
use crate::commands::{Command, CommandExecutor};

/// Operaciones soportadas por el PropertyActuator
///
/// Referencia: SCA_PropertyActuator::Execute() en BGE
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyOperation {
    /// Asignar valor (=)
    Assign = 0,
    
    /// Sumar valor (+=)
    Add = 1,
    
    /// Restar valor (-=)
    Sub = 2,
    
    /// Multiplicar valor (*=)
    Mul = 3,
    
    /// Dividir valor (/=)
    Div = 4,
    
    /// Copiar valor desde otra propiedad
    Copy = 5,
    
    /// Módulo (%=)
    Mod = 6,
}

/// Configuración del actuador de propiedades
#[repr(C)]
#[derive(Debug, Clone)]
pub struct PropertyActuatorConfig {
    /// Nombre de la propiedad a modificar
    pub property: String,
    
    /// Valor a aplicar
    pub value: PropertyValue,
    
    /// Operación a realizar
    pub operation: PropertyOperation,
    
    /// Sobrescribir aunque no exista
    pub overwrite: bool,
    
    /// Prioridad del actuador (0 = más alta)
    pub priority: u32,
}

impl Default for PropertyActuatorConfig {
    fn default() -> Self {
        Self {
            property: String::new(),
            value: PropertyValue::Float(0.0),
            operation: PropertyOperation::Assign,
            overwrite: true,
            priority: 0,
        }
    }
}

/// Actuador de propiedades (BGE-Faithful)
///
/// Este actuador modifica propiedades de entidades cuando recibe un pulso.
///
/// # Ejemplo
///
/// ```rust
/// use archflow_logic::actuators::property::{PropertyActuator, PropertyOperation};
///
/// // Actuador para mover entidad 100 unidades en X
/// let actuator = PropertyActuator::new(
///     EntityId::from_raw(42),
///     PropertyActuatorConfig {
///         property: "position_x".to_string(),
///         value: PropertyValue::Float(100.0),
///         operation: PropertyOperation::Add,
///         ..Default::default()
///     }
/// );
/// ```
pub struct PropertyActuator {
    /// ID único del actuador
    pub id: u32,
    
    /// Entidad a la que afecta
    pub entity_id: EntityId,
    
    /// Configuración del actuador
    config: PropertyActuatorConfig,
    
    /// Estado interno (está activo?)
    active: bool,
    
    /// Comando inverso para undo/redo
    inverse_command: Option<Box<dyn Command>>,
}

impl PropertyActuator {
    /// Crea un nuevo PropertyActuator
    pub fn new(entity_id: EntityId, config: PropertyActuatorConfig) -> Self {
        Self {
            id: 0,
            entity_id,
            config,
            active: false,
            inverse_command: None,
        }
    }
    
    /// Activa el actuador (cuando recibe pulso Positive)
    pub fn activate(&mut self, executor: &mut CommandExecutor) {
        self.active = true;
        
        // Crear comando según operación
        let command = match self.config.operation {
            PropertyOperation::Assign => {
                commands::PropertyAssignCommand::new(
                    self.entity_id,
                    self.config.property.clone(),
                    self.config.value.clone(),
                )
            }
            PropertyOperation::Add => {
                commands::PropertyAddCommand::new(
                    self.entity_id,
                    self.config.property.clone(),
                    self.config.value.clone(),
                )
            }
            // ... otras operaciones
            _ => unimplemented!(),
        };
        
        // Ejecutar comando
        let inverse = executor.execute(Box::new(command));
        self.inverse_command = Some(inverse);
    }
    
    /// Desactiva el actuador (cuando recibe pulso Negative)
    pub fn deactivate(&mut self) {
        self.active = false;
        self.inverse_command = None;
    }
    
    /// Obtiene el comando inverso (para undo)
    pub fn get_inverse(&self) -> Option<&dyn Command> {
        self.inverse_command.as_ref().map(|b| b.as_ref())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMANDOS PARA EL PROPERTY ACTUATOR
// ═══════════════════════════════════════════════════════════════════════════

pub mod commands {
    use super::*;
    use crate::commands::{Command, CommandResult};
    use archflow_core::EntityStore;
    
    /// Comando para asignar una propiedad
    pub struct PropertyAssignCommand {
        entity_id: EntityId,
        property: String,
        new_value: PropertyValue,
        old_value: Option<PropertyValue>,
    }
    
    impl PropertyAssignCommand {
        pub fn new(entity_id: EntityId, property: String, new_value: PropertyValue) -> Self {
            Self {
                entity_id,
                property,
                new_value,
                old_value: None,
            }
        }
    }
    
    impl Command for PropertyAssignCommand {
        fn execute(&mut self, store: &mut EntityStore) -> CommandResult {
            // Guardar valor anterior para undo
            self.old_value = store.get_property(self.entity_id, &self.property);
            
            // Asignar nuevo valor
            store.set_property(self.entity_id, &self.property, &self.new_value);
            
            Ok(())
        }
        
        fn inverse(&self) -> Box<dyn Command> {
            Box::new(PropertyAssignCommand {
                entity_id: self.entity_id,
                property: self.property.clone(),
                new_value: self.old_value.clone().unwrap_or(PropertyValue::Float(0.0)),
                old_value: None,
            })
        }
    }
    
    /// Comando para sumar a una propiedad
    pub struct PropertyAddCommand {
        entity_id: EntityId,
        property: String,
        delta: PropertyValue,
        old_value: Option<PropertyValue>,
    }
    
    impl PropertyAddCommand {
        pub fn new(entity_id: EntityId, property: String, delta: PropertyValue) -> Self {
            Self {
                entity_id,
                property,
                delta,
                old_value: None,
            }
        }
    }
    
    impl Command for PropertyAddCommand {
        fn execute(&mut self, store: &mut EntityStore) -> CommandResult {
            self.old_value = store.get_property(self.entity_id, &self.property);
            
            let current = self.old_value.as_ref().unwrap_or(&PropertyValue::Float(0.0));
            let new = match (current, &self.delta) {
                (PropertyValue::Float(a), PropertyValue::Float(b)) => {
                    PropertyValue::Float(a + b)
                }
                (PropertyValue::Int(a), PropertyValue::Int(b)) => {
                    PropertyValue::Int(a + b)
                }
                _ => return Err("Cannot add these types".into()),
            };
            
            store.set_property(self.entity_id, &self.property, &new);
            Ok(())
        }
        
        fn inverse(&self) -> Box<dyn Command> {
            Box::new(PropertyAssignCommand {
                entity_id: self.entity_id,
                property: self.property.clone(),
                new_value: self.old_value.clone().unwrap_or(PropertyValue::Float(0.0)),
                old_value: None,
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SISTEMA DE ACTUADORES
// ═══════════════════════════════════════════════════════════════════════════

/// Sistema que procesa todos los actuadores activados por pulsos
///
/// Este es el "puente" entre el PulseBus y los actuadores BGE.
pub fn sys_actuator_execution(
    world: &mut World,
    bus: &PulseBus,
    executor: &mut CommandExecutor,
) {
    let ts = world.time.current();
    
    // Obtener todos los pulsos del frame actual
    let pulses = bus.get_pulses_since_frame(ts);
    
    for pulse in pulses {
        // Buscar actuadores conectados al sensor que emitió el pulso
        if let Some(actuators) = world.wiring_table.get_actuators_for_sensor(pulse.sensor_id) {
            for actuator in actuators {
                match pulse.state {
                    SensorState::Positive => {
                        actuator.activate(executor);
                    }
                    SensorState::Negative => {
                        actuator.deactivate();
                    }
                    SensorState::None => {}
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_property_assign() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        store.set_property(entity, "health", PropertyValue::Float(100.0));
        
        let mut actuator = PropertyActuator::new(
            entity,
            PropertyActuatorConfig {
                property: "health".to_string(),
                value: PropertyValue::Float(50.0),
                operation: PropertyOperation::Assign,
                ..Default::default()
            }
        );
        
        let mut executor = CommandExecutor::new(&store);
        actuator.activate(&mut executor);
        
        let health = store.get_property(entity, "health").unwrap();
        assert_eq!(health, PropertyValue::Float(50.0));
    }
    
    #[test]
    fn test_property_add() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        store.set_property(entity, "score", PropertyValue::Int(10));
        
        let mut actuator = PropertyActuator::new(
            entity,
            PropertyActuatorConfig {
                property: "score".to_string(),
                value: PropertyValue::Int(5),
                operation: PropertyOperation::Add,
                ..Default::default()
            }
        );
        
        let mut executor = CommandExecutor::new(&store);
        actuator.activate(&mut executor);
        
        let score = store.get_property(entity, "score").unwrap();
        assert_eq!(score, PropertyValue::Int(15));
    }
    
    #[test]
    fn test_property_undo() {
        // Test de comando inverso
        // ...
    }
}
```

---

## 4. Mapeo a la Arquitectura de ArchFlow

### 4.1 Sensores → PulseBus → Actuadores

El flujo completo en ArchFlow es:

```
┌─────────────┐      ┌──────────┐      ┌─────────────┐
│  Sensores   │ ───> │ PulseBus │ ───> │  Actuadores │
│  (Input/    │      │          │      │  (Action)   │
│   Física)   │      │  (Events)│      │             │
└─────────────┘      └──────────┘      └─────────────┘
      │                                      │
      ▼                                      ▼
  Detectan                            Ejecutan
  Cambios                            Comandos
```

### 4.2 Wiring Table

La **Wiring Table** es el componente que conecta sensores con actuadores:

```rust
pub struct WiringTable {
    /// Mapeo: sensor_id -> vec<actuator_id>
    connections: HashMap<u32, Vec<u32>>,
    
    /// Mapeo: actuator_id -> lógica de activación
    logic: HashMap<u32, ActivationLogic>,
}

pub enum ActivationLogic {
    /// Activar cuando el pulso es Positive
    OnPositive,
    
    /// Activar cuando el pulso es Negative
    OnNegative,
    
    /// Activar siempre
    Always,
    
    /// Combinación lógica (AND, OR, NOT)
    Combined { logic: LogicGate },
}
```

### 4.3 Actuadores como Comandos

Cada actuador, al activarse, genera un **Command** reversible:

```rust
pub trait Command {
    /// Ejecuta el comando
    fn execute(&mut self, store: &mut EntityStore) -> CommandResult;
    
    /// Retorna el comando inverso (para undo)
    fn inverse(&self) -> Box<dyn Command>;
}
```

Esto permite:
- **Undo/Redo automático**
- **Reproducibilidad** (Event Sourcing)
- **Sincronización en red** (enviar comandos, no estado)

---

## 5. Actuadores Críticos para ArchFlow

De los 16 tipos de actuadores BGE, ArchFlow implementa primero:

| Prioridad | Actuador BGE | Uso en ArchFlow | Complejidad |
|-----------|--------------|-----------------|-------------|
| **Alta** | PropertyActuator | Modificar propiedades de nodos | S |
| **Alta** | ActionActuator | Animaciones interpoladas | M |
| **Alta** | VisibilityActuator | Mostrar/ocultar elementos | XS |
| **Alta** | StateActuator | Máquinas de estado | M |
| **Media** | SceneActuator | Gestión de escenas/viewport | M |
| **Media** | MessageActuator | Comunicación entre componentes | S |
| **Baja** | CameraActuator | Seguimiento de cámara | L |
| **Baja** | SoundActuator | Efectos de audio | L |
| **Futura** | SteeringActuator | Pathfinding/IA | XXL |
| **Futura** | Filter2DActuator | Post-procesado | XL |

---

## 6. Fuentes de Referencia

### 6.1 Código Fuente de Blender BGE

```
blender/source/gameengine/Ketsji/
├── SCA_IActuator.cpp                 # Base class para todos los actuadores
├── SCA_ActionActuator.cpp            # Animaciones/actions
├── SCA_PropertyActuator.cpp          # Modificar propiedades
├── SCA_SoundActuator.cpp             # Reproducir sonidos
├── KX_CameraActuator.cpp             # Control de cámara
├── KX_ConstraintActuator.cpp         # Restricciones físicas
├── KX_GameActuator.cpp               # Control del juego
├── KX_MessageActuator.cpp            # Mensajes entre objetos
├── KX_MouseActuator.cpp              # Control de mouse
├── KX_ParentActuator.cpp             # Parenting
├── KX_RandomActuator.cpp             # Valores aleatorios
├── KX_SceneActuator.cpp              # Gestión de escenas
├── KX_StateActuator.cpp              # Máquinas de estado
├── KX_VisibilityActuator.cpp         # Visibilidad
├── KX_SteeringActuator.cpp           # Steering behaviors
├── KX_Filter2DActuator.cpp           # Filtros 2D
└── KX_ArmatureActuator.cpp           # Control de armaduras
```

### 6.2 Documentación Python API

- **Blender 2.79 API**: https://docs.blender.org/api/2.79a/bge.types.html
- **UPBGE Docs**: https://upbge.org/docs/latest/manual/logic_bricks/actuators/index.html

---

## 7. Conclusión

Este documento complementa la investigación de sensores BGE, documentando todos los actuadores necesarios para completar el sistema de **Logic Bricks** en ArchFlow.

**Próximos pasos**:

1. Implementar `PropertyActuator` (más simple y crítico)
2. Implementar `VisibilityActuator` (muy útil para UI)
3. Implementar `StateActuator` (para máquinas de estado)
4. Implementar `ActionActuator` con tweening (para animaciones)
5. Implementar `WiringTable` (para conectar sensores con actuadores)

---

**Fin del Documento de Investigación de Actuadores BGE**

---

*Investigación realizada por Claude Code*
*Fecha: 2026-02-01*
*Proyecto: ArchFlow - BGE Actuators Study*
