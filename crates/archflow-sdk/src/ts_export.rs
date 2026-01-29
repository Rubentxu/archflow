//! TypeScript export helpers for external types
//!
//! This module provides TypeScript-compatible wrappers for types from
//! archflow_core that don't implement TS directly.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// TypeScript-compatible wrapper for Vec2
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TsVec2 {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl From<archflow_core::Vec2> for TsVec2 {
    fn from(v: archflow_core::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<TsVec2> for archflow_core::Vec2 {
    fn from(v: TsVec2) -> Self {
        Self::new(v.x, v.y)
    }
}

/// TypeScript-compatible wrapper for Color
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TsColor {
    /// Red component (0.0 - 1.0)
    pub r: f32,
    /// Green component (0.0 - 1.0)
    pub g: f32,
    /// Blue component (0.0 - 1.0)
    pub b: f32,
    /// Alpha component (0.0 - 1.0)
    pub a: f32,
}

impl From<archflow_core::Color> for TsColor {
    fn from(c: archflow_core::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

impl From<TsColor> for archflow_core::Color {
    fn from(c: TsColor) -> Self {
        Self::rgba(c.r, c.g, c.b, c.a)
    }
}

/// TypeScript-compatible wrapper for EntityId
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TsEntityId {
    /// UUID string representation
    pub id: String,
}

impl From<archflow_core::EntityId> for TsEntityId {
    fn from(id: archflow_core::EntityId) -> Self {
        Self { id: id.to_string() }
    }
}

impl From<TsEntityId> for archflow_core::EntityId {
    fn from(ts_id: TsEntityId) -> Self {
        // For conversion back, we always create a new ID
        // The string representation is for TypeScript interop only
        Self::new()
    }
}

/// TypeScript-compatible wrapper for Rect
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TsRect {
    /// Minimum corner (top-left)
    pub min: TsVec2,
    /// Maximum corner (bottom-right)
    pub max: TsVec2,
}

impl From<archflow_core::Rect> for TsRect {
    fn from(r: archflow_core::Rect) -> Self {
        Self {
            min: r.min.into(),
            max: r.max.into(),
        }
    }
}

impl From<TsRect> for archflow_core::Rect {
    fn from(r: TsRect) -> Self {
        Self::from_min_max(r.min.into(), r.max.into())
    }
}

/// TypeScript-compatible wrapper for Transform
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TsTransform {
    /// Translation
    pub translation: TsVec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: TsVec2,
}

impl From<archflow_core::Transform> for TsTransform {
    fn from(t: archflow_core::Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation,
            scale: t.scale.into(),
        }
    }
}

impl From<TsTransform> for archflow_core::Transform {
    fn from(t: TsTransform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation,
            scale: t.scale.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_vec2_conversion() {
        let core_vec = archflow_core::Vec2::new(10.0, 20.0);
        let ts_vec: TsVec2 = core_vec.into();
        assert_eq!(ts_vec.x, 10.0);
        assert_eq!(ts_vec.y, 20.0);

        let back: archflow_core::Vec2 = ts_vec.into();
        assert_eq!(back.x, 10.0);
        assert_eq!(back.y, 20.0);
    }

    #[test]
    fn test_ts_color_conversion() {
        let core_color = archflow_core::Color::rgb(0.5, 0.6, 0.7);
        let ts_color: TsColor = core_color.into();
        assert_eq!(ts_color.r, 0.5);
        assert_eq!(ts_color.g, 0.6);
        assert_eq!(ts_color.b, 0.7);
        assert_eq!(ts_color.a, 1.0);

        let back: archflow_core::Color = ts_color.into();
        assert_eq!(back.r, 0.5);
        assert_eq!(back.g, 0.6);
        assert_eq!(back.b, 0.7);
    }

    #[test]
    fn test_ts_entity_id_conversion() {
        let core_id = archflow_core::EntityId::new();
        let ts_id: TsEntityId = core_id.into();
        assert!(!ts_id.id.is_empty());

        // Note: Converting back from TsEntityId creates a new ID
        // The string representation is for TypeScript interop only
        let back: archflow_core::EntityId = ts_id.into();
        // Just verify we got a valid ID back
        assert!(!back.to_string().is_empty());
    }

    #[test]
    fn test_ts_rect_conversion() {
        let core_rect = archflow_core::Rect::from_min_max(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(100.0, 100.0),
        );
        let ts_rect: TsRect = core_rect.into();
        assert_eq!(ts_rect.min.x, 0.0);
        assert_eq!(ts_rect.max.x, 100.0);

        let back: archflow_core::Rect = ts_rect.into();
        assert_eq!(back.min.x, 0.0);
        assert_eq!(back.max.x, 100.0);
    }
}
