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

    /// Crear EntityId desde un string UUID
    pub fn from_str(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }

    /// Crear EntityId desde bytes (para deserialización)
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        Self(Uuid::from_bytes(*bytes))
    }

    /// Crear EntityId desde u128 (para testing)
    pub fn from_u128(v: u128) -> Self {
        let bytes = v.to_be_bytes();
        Self(Uuid::from_bytes(bytes))
    }

    /// Obtener como u128 (para testing)
    pub fn as_u128(&self) -> u128 {
        let bytes = self.0.as_bytes();
        u128::from_be_bytes(*bytes)
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

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(entity_id: EntityId) -> Self {
        entity_id.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generador de EntityIds predecibles (para testing)
#[derive(Debug, Clone)]
pub struct EntityIdGenerator {
    counter: u32,
}

impl EntityIdGenerator {
    /// Crear nuevo generador
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Generar el siguiente EntityId
    pub fn generate(&mut self) -> EntityId {
        let id = Uuid::new_v4();
        self.counter += 1;
        EntityId(id)
    }
}

impl Default for EntityIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}
