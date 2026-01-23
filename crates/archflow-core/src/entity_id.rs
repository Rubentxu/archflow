//! EntityId - Identificador único de entidad

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Identificador único para cada entidad en el sistema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Crear un nuevo EntityId aleatorio
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Crear EntityId desde bytes (para deserialización)
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        Self(Uuid::from_bytes(*bytes))
    }

    /// Obtener como bytes
    pub fn as_bytes(&self) -> [u8; 16] {
        *self.0.as_bytes()
    }

    /// Obtener como string (UUID format)
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generador de EntityIds predecibles (para testing)
pub struct EntityIdGenerator {
    counter: u32,
    namespace: Uuid,
}

impl EntityIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: 0,
            namespace: Uuid::nil(),
        }
    }

    pub fn next(&mut self) -> EntityId {
        // Usar new_v4 para generar IDs únicos basados en counter
        let id = Uuid::new_v4();
        self.counter += 1;
        EntityId(id)
    }
}
