//! Error - Tipos de error del módulo core

use thiserror::Error;

/// Error del módulo core
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid transform: {0}")]
    InvalidTransform(String),

    #[error("Invalid rectangle: {0}")]
    InvalidRect(String),

    #[error("Invalid color: {0}")]
    InvalidColor(#[from] super::color::ColorParseError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Resultado con error de core
pub type CoreResult<T> = Result<T, CoreError>;
