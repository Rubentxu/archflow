//! Properties panel module for ArchFlow SDK
//!
//! Provides functionality for editing shape properties:
//! - Position, size, rotation editing
//! - Color properties (fill, stroke)
//! - Validation of property values
//! - Multi-selection support with mixed values detection
//! - Live update with undo/redo support

use crate::canvas::{Canvas, Shape, ShapeChanges};
use crate::commands::{Command, CommandResult};
use crate::selection::SelectionDelta;
use archflow_core::{Color, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for property operations
#[derive(Debug, thiserror::Error)]
pub enum PropertyError {
    #[error("Shape not found: {0}")]
    ShapeNotFound(EntityId),
    #[error("Invalid property value: {0}")]
    InvalidValue(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("No shape selected")]
    NoSelection,
    #[error("Mixed values not supported for this operation")]
    MixedValues,
}

/// Type alias for property operation results
pub type PropertyResult<T> = Result<T, PropertyError>;

/// Represents the value of a property
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// No value (nothing selected)
    None,
    /// Single value (one shape selected or all have same value)
    Single(PrimitiveValue),
    /// Mixed values (multiple shapes with different values)
    Mixed,
}

/// Primitive property values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValue {
    /// No value (for optional properties)
    None,
    /// Floating point number
    Float(f32),
    /// Integer
    Int(i32),
    /// String
    String(String),
    /// Boolean
    Bool(bool),
    /// Color
    Color(Color),
}

impl PrimitiveValue {
    /// Returns the float value if applicable
    pub fn as_float(&self) -> Option<f32> {
        match self {
            PrimitiveValue::Float(f) => Some(*f),
            PrimitiveValue::Int(i) => Some(*i as f32),
            _ => None,
        }
    }

    /// Returns the int value if applicable
    pub fn as_int(&self) -> Option<i32> {
        match self {
            PrimitiveValue::Int(i) => Some(*i),
            PrimitiveValue::Float(f) => Some(*f as i32),
            _ => None,
        }
    }

    /// Returns the string value if applicable
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PrimitiveValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the bool value if applicable
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PrimitiveValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the color value if applicable
    pub fn as_color(&self) -> Option<Color> {
        match self {
            PrimitiveValue::Color(c) => Some(*c),
            _ => None,
        }
    }
}

impl Default for PrimitiveValue {
    fn default() -> Self {
        PrimitiveValue::None
    }
}

/// Property types that can be edited
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropertyType {
    /// X position
    X,
    /// Y position
    Y,
    /// Width
    Width,
    /// Height
    Height,
    /// Rotation angle
    Rotation,
    /// Fill color
    FillColor,
    /// Stroke color
    StrokeColor,
    /// Stroke width
    StrokeWidth,
    /// Opacity
    Opacity,
}

impl PropertyType {
    /// Returns the display name for the property
    pub fn display_name(&self) -> &'static str {
        match self {
            PropertyType::X => "X Position",
            PropertyType::Y => "Y Position",
            PropertyType::Width => "Width",
            PropertyType::Height => "Height",
            PropertyType::Rotation => "Rotation",
            PropertyType::FillColor => "Fill Color",
            PropertyType::StrokeColor => "Stroke Color",
            PropertyType::StrokeWidth => "Stroke Width",
            PropertyType::Opacity => "Opacity",
        }
    }

    /// Returns true if this property requires validation (e.g., must be positive)
    pub fn requires_validation(&self) -> bool {
        matches!(
            self,
            PropertyType::Width
                | PropertyType::Height
                | PropertyType::StrokeWidth
                | PropertyType::Opacity
        )
    }

    /// Validates a value for this property type
    pub fn validate(&self, value: &PrimitiveValue) -> PropertyResult<()> {
        match self {
            PropertyType::Width | PropertyType::Height => {
                if let Some(v) = value.as_float() {
                    if v <= 0.0 {
                        return Err(PropertyError::ValidationFailed(format!(
                            "{} must be positive, got {}",
                            self.display_name(),
                            v
                        )));
                    }
                }
            }
            PropertyType::StrokeWidth => {
                if let Some(v) = value.as_float() {
                    if v < 0.0 {
                        return Err(PropertyError::ValidationFailed(format!(
                            "{} must be non-negative, got {}",
                            self.display_name(),
                            v
                        )));
                    }
                }
            }
            PropertyType::Opacity => {
                if let Some(v) = value.as_float() {
                    if v < 0.0 || v > 1.0 {
                        return Err(PropertyError::ValidationFailed(format!(
                            "{} must be between 0 and 1, got {}",
                            self.display_name(),
                            v
                        )));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Stores original property values for undo
#[derive(Clone, Debug, Default)]
pub struct OriginalProperties {
    values: HashMap<EntityId, ShapeProperties>,
}

impl OriginalProperties {
    /// Creates a new original properties store
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Stores original properties for a shape
    pub fn store(&mut self, shape_id: EntityId, props: ShapeProperties) {
        self.values.insert(shape_id, props);
    }

    /// Gets original properties for a shape
    pub fn get(&self, shape_id: EntityId) -> Option<&ShapeProperties> {
        self.values.get(&shape_id)
    }
}

/// Properties of a shape for editing
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeProperties {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub fill_color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub opacity: f32,
}

impl ShapeProperties {
    /// Creates properties from a shape
    pub fn from_shape(shape: &Shape) -> Self {
        Self {
            x: shape.x,
            y: shape.y,
            width: shape.width,
            height: shape.height,
            rotation: shape.rotation,
            fill_color: shape.fill_color,
            stroke_color: shape.stroke_color,
            stroke_width: shape.stroke_width,
            opacity: shape.opacity,
        }
    }

    /// Applies these properties to a ShapeChanges
    pub fn to_changes(&self) -> ShapeChanges {
        ShapeChanges {
            x: Some(self.x),
            y: Some(self.y),
            width: Some(self.width),
            height: Some(self.height),
            rotation: Some(self.rotation),
            fill_color: Some(self.fill_color),
            stroke_color: Some(self.stroke_color),
            stroke_width: Some(self.stroke_width),
            opacity: Some(self.opacity),
        }
    }
}

/// Manager for shape properties
pub struct PropertiesManager {
    /// Currently selected shape IDs
    selected_shapes: Vec<EntityId>,
    /// Property update bus for live updates
    update_callbacks: Vec<Box<dyn Fn(&PropertyUpdateEvent)>>,
}

impl Default for PropertiesManager {
    fn default() -> Self {
        Self {
            selected_shapes: Vec::new(),
            update_callbacks: Vec::new(),
        }
    }
}

impl std::fmt::Debug for PropertiesManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertiesManager")
            .field("selected_shapes", &self.selected_shapes)
            .field("update_callbacks", &self.update_callbacks.len())
            .finish()
    }
}

/// Event sent when properties are updated
#[derive(Clone, Debug)]
pub struct PropertyUpdateEvent {
    pub shape_ids: Vec<EntityId>,
    pub property_type: PropertyType,
    pub new_value: PropertyValue,
}

impl PropertiesManager {
    /// Creates a new properties manager
    pub fn new() -> Self {
        Self {
            selected_shapes: Vec::new(),
            update_callbacks: Vec::new(),
        }
    }

    /// Sets the selected shapes
    pub fn set_selection(&mut self, shape_ids: Vec<EntityId>) {
        self.selected_shapes = shape_ids;
    }

    /// Gets the current selection
    pub fn selection(&self) -> &[EntityId] {
        &self.selected_shapes
    }

    /// Clears the selection
    pub fn clear_selection(&mut self) {
        self.selected_shapes.clear();
    }

    /// Returns true if a single shape is selected
    pub fn is_single_selection(&self) -> bool {
        self.selected_shapes.len() == 1
    }

    /// Returns true if multiple shapes are selected
    pub fn is_multi_selection(&self) -> bool {
        self.selected_shapes.len() > 1
    }

    /// Gets a property value for the current selection
    pub fn get_property_value(&self, canvas: &Canvas, property: PropertyType) -> PropertyValue {
        if self.selected_shapes.is_empty() {
            return PropertyValue::None;
        }

        let mut values: Vec<PrimitiveValue> = Vec::new();

        for &shape_id in &self.selected_shapes {
            if let Some(shape) = canvas.get_shape(shape_id) {
                let value = match property {
                    PropertyType::X => PrimitiveValue::Float(shape.x),
                    PropertyType::Y => PrimitiveValue::Float(shape.y),
                    PropertyType::Width => PrimitiveValue::Float(shape.width),
                    PropertyType::Height => PrimitiveValue::Float(shape.height),
                    PropertyType::Rotation => PrimitiveValue::Float(shape.rotation),
                    PropertyType::FillColor => PrimitiveValue::Color(shape.fill_color),
                    PropertyType::StrokeColor => shape
                        .stroke_color
                        .map(PrimitiveValue::Color)
                        .unwrap_or(PrimitiveValue::None),
                    PropertyType::StrokeWidth => PrimitiveValue::Float(shape.stroke_width),
                    PropertyType::Opacity => PrimitiveValue::Float(shape.opacity),
                };
                values.push(value);
            }
        }

        if values.is_empty() {
            PropertyValue::None
        } else if values.iter().all(|v| v == &values[0]) {
            PropertyValue::Single(values[0].clone())
        } else {
            PropertyValue::Mixed
        }
    }

    /// Checks if the current selection has mixed values for a property
    pub fn has_mixed_values(&self, canvas: &Canvas, property: PropertyType) -> bool {
        matches!(
            self.get_property_value(canvas, property),
            PropertyValue::Mixed
        )
    }

    /// Creates a command to update a property
    pub fn create_update_command(
        &self,
        property: PropertyType,
        value: PrimitiveValue,
    ) -> PropertyResult<UpdatePropertyCommand> {
        if self.selected_shapes.is_empty() {
            return Err(PropertyError::NoSelection);
        }

        // Validate the value
        property.validate(&value)?;

        Ok(UpdatePropertyCommand::new(
            self.selected_shapes.clone(),
            property,
            value,
        ))
    }

    /// Registers a callback for property updates
    pub fn on_update<F>(&mut self, callback: F)
    where
        F: Fn(&PropertyUpdateEvent) + 'static,
    {
        self.update_callbacks.push(Box::new(callback));
    }

    /// Notifies all registered callbacks of an update
    fn notify_update(&self, event: PropertyUpdateEvent) {
        for callback in &self.update_callbacks {
            callback(&event);
        }
    }
}

/// Helper to convert PrimitiveValue to Option<Color>
impl From<&PrimitiveValue> for Option<Color> {
    fn from(value: &PrimitiveValue) -> Self {
        match value {
            PrimitiveValue::Color(c) => Some(*c),
            _ => None,
        }
    }
}

/// Command to update a property
#[derive(Clone, Debug)]
pub struct UpdatePropertyCommand {
    shape_ids: Vec<EntityId>,
    property: PropertyType,
    new_value: PrimitiveValue,
    original_properties: OriginalProperties,
    executed: bool,
}

impl UpdatePropertyCommand {
    /// Creates a new update property command
    pub fn new(
        shape_ids: Vec<EntityId>,
        property: PropertyType,
        new_value: PrimitiveValue,
    ) -> Self {
        Self {
            shape_ids,
            property,
            new_value,
            original_properties: OriginalProperties::new(),
            executed: false,
        }
    }

    /// Gets the shape IDs
    pub fn shape_ids(&self) -> &[EntityId] {
        &self.shape_ids
    }

    /// Gets the property type
    pub fn property(&self) -> PropertyType {
        self.property
    }

    /// Gets the new value
    pub fn new_value(&self) -> &PrimitiveValue {
        &self.new_value
    }
}

impl Command for UpdatePropertyCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Capture original properties
        for &shape_id in &self.shape_ids {
            if let Some(shape) = canvas.get_shape(shape_id) {
                self.original_properties
                    .store(shape_id, ShapeProperties::from_shape(shape));
            }
        }

        // Apply the new value to each shape
        for &shape_id in &self.shape_ids {
            if canvas.get_shape(shape_id).is_none() {
                continue;
            }

            let changes = match self.property {
                PropertyType::X => ShapeChanges {
                    x: self.new_value.as_float(),
                    y: None,
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::Y => ShapeChanges {
                    x: None,
                    y: self.new_value.as_float(),
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::Width => ShapeChanges {
                    x: None,
                    y: None,
                    width: self.new_value.as_float(),
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::Height => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: self.new_value.as_float(),
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::Rotation => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    rotation: self.new_value.as_float(),
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::FillColor => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: self.new_value.as_color(),
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::StrokeColor => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: Some(self.new_value.as_color()),
                    stroke_width: None,
                    opacity: None,
                },
                PropertyType::StrokeWidth => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: self.new_value.as_float(),
                    opacity: None,
                },
                PropertyType::Opacity => ShapeChanges {
                    x: None,
                    y: None,
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: self.new_value.as_float(),
                },
            };

            canvas.update_shape(shape_id, changes);
        }

        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Restore original properties
        for &shape_id in &self.shape_ids {
            if let Some(original) = self.original_properties.get(shape_id) {
                canvas.update_shape(shape_id, original.to_changes());
            }
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        match self.property {
            PropertyType::X => "Update X position",
            PropertyType::Y => "Update Y position",
            PropertyType::Width => "Update width",
            PropertyType::Height => "Update height",
            PropertyType::Rotation => "Update rotation",
            PropertyType::FillColor => "Update fill color",
            PropertyType::StrokeColor => "Update stroke color",
            PropertyType::StrokeWidth => "Update stroke width",
            PropertyType::Opacity => "Update opacity",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties_manager_creation() {
        let manager = PropertiesManager::new();
        assert!(manager.selection().is_empty());
        assert!(!manager.is_single_selection());
        assert!(!manager.is_multi_selection());
    }

    #[test]
    fn test_set_selection() {
        let mut manager = PropertiesManager::new();
        let ids = vec![EntityId::new(), EntityId::new()];

        manager.set_selection(ids.clone());
        assert_eq!(manager.selection().len(), 2);
        assert!(manager.is_multi_selection());
    }

    #[test]
    fn test_get_property_value_single() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 200.0, 50.0, 75.0);

        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![id]);

        let x_value = manager.get_property_value(&canvas, PropertyType::X);
        match x_value {
            PropertyValue::Single(PrimitiveValue::Float(v)) => assert_eq!(v, 100.0),
            _ => panic!("Expected single float value"),
        }
    }

    #[test]
    fn test_get_property_value_mixed() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 200.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![id1, id2]);

        let x_value = manager.get_property_value(&canvas, PropertyType::X);
        assert!(matches!(x_value, PropertyValue::Mixed));
    }

    #[test]
    fn test_get_property_value_none() {
        let canvas = Canvas::new(800.0, 600.0);
        let manager = PropertiesManager::new();

        let x_value = manager.get_property_value(&canvas, PropertyType::X);
        assert!(matches!(x_value, PropertyValue::None));
    }

    #[test]
    fn test_has_mixed_values() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 200.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![id1, id2]);

        assert!(manager.has_mixed_values(&canvas, PropertyType::X));
        assert!(!manager.has_mixed_values(&canvas, PropertyType::Height));
    }

    #[test]
    fn test_shape_properties_from_shape() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 200.0, 50.0, 75.0);
        let shape = canvas.get_shape(id).unwrap();

        let props = ShapeProperties::from_shape(shape);
        assert_eq!(props.x, 100.0);
        assert_eq!(props.y, 200.0);
        assert_eq!(props.width, 50.0);
        assert_eq!(props.height, 75.0);
    }

    #[test]
    fn test_property_validation_width() {
        let result = PropertyType::Width.validate(&PrimitiveValue::Float(-10.0));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::Width.validate(&PrimitiveValue::Float(0.0));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::Width.validate(&PrimitiveValue::Float(100.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_validation_height() {
        let result = PropertyType::Height.validate(&PrimitiveValue::Float(-5.0));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::Height.validate(&PrimitiveValue::Float(50.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_validation_opacity() {
        let result = PropertyType::Opacity.validate(&PrimitiveValue::Float(-0.5));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::Opacity.validate(&PrimitiveValue::Float(1.5));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::Opacity.validate(&PrimitiveValue::Float(0.5));
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_validation_stroke_width() {
        let result = PropertyType::StrokeWidth.validate(&PrimitiveValue::Float(-2.0));
        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));

        let result = PropertyType::StrokeWidth.validate(&PrimitiveValue::Float(0.0));
        assert!(result.is_ok());

        let result = PropertyType::StrokeWidth.validate(&PrimitiveValue::Float(5.0));
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_property_command() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 200.0, 50.0, 50.0);

        let mut cmd =
            UpdatePropertyCommand::new(vec![id], PropertyType::X, PrimitiveValue::Float(300.0));

        cmd.execute(&mut canvas).unwrap();
        assert_eq!(canvas.get_shape(id).unwrap().x, 300.0);

        cmd.undo(&mut canvas).unwrap();
        assert_eq!(canvas.get_shape(id).unwrap().x, 100.0);
    }

    #[test]
    fn test_update_property_command_multi_selection() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut cmd = UpdatePropertyCommand::new(
            vec![id1, id2],
            PropertyType::FillColor,
            PrimitiveValue::Color(Color::rgb(1.0, 0.0, 0.0)),
        );

        cmd.execute(&mut canvas).unwrap();
        assert_eq!(
            canvas.get_shape(id1).unwrap().fill_color,
            Color::rgb(1.0, 0.0, 0.0)
        );
        assert_eq!(
            canvas.get_shape(id2).unwrap().fill_color,
            Color::rgb(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn test_create_update_command_no_selection() {
        let manager = PropertiesManager::new();
        let result = manager.create_update_command(PropertyType::X, PrimitiveValue::Float(100.0));

        assert!(matches!(result, Err(PropertyError::NoSelection)));
    }

    #[test]
    fn test_create_update_command_validation_fail() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 200.0, 50.0, 50.0);

        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![id]);

        let result =
            manager.create_update_command(PropertyType::Width, PrimitiveValue::Float(-10.0));

        assert!(matches!(result, Err(PropertyError::ValidationFailed(_))));
    }

    #[test]
    fn test_primitive_value_conversions() {
        let float_val = PrimitiveValue::Float(10.5);
        assert_eq!(float_val.as_float(), Some(10.5));
        assert_eq!(float_val.as_int(), Some(10));

        let int_val = PrimitiveValue::Int(42);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_float(), Some(42.0));

        let bool_val = PrimitiveValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_float(), None);

        let color = Color::rgb(1.0, 0.5, 0.0);
        let color_val = PrimitiveValue::Color(color);
        assert_eq!(color_val.as_color(), Some(color));
        assert_eq!(color_val.as_float(), None);
    }

    #[test]
    fn test_property_type_display_names() {
        assert_eq!(PropertyType::X.display_name(), "X Position");
        assert_eq!(PropertyType::Y.display_name(), "Y Position");
        assert_eq!(PropertyType::Width.display_name(), "Width");
        assert_eq!(PropertyType::Height.display_name(), "Height");
        assert_eq!(PropertyType::FillColor.display_name(), "Fill Color");
        assert_eq!(PropertyType::Opacity.display_name(), "Opacity");
    }

    #[test]
    fn test_update_property_preserves_unaffected() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 200.0, 50.0, 75.0);
        let original_y = canvas.get_shape(id).unwrap().y;
        let original_width = canvas.get_shape(id).unwrap().width;

        let mut cmd =
            UpdatePropertyCommand::new(vec![id], PropertyType::X, PrimitiveValue::Float(300.0));

        cmd.execute(&mut canvas).unwrap();

        // Y and width should be preserved
        assert_eq!(canvas.get_shape(id).unwrap().y, original_y);
        assert_eq!(canvas.get_shape(id).unwrap().width, original_width);
    }

    #[test]
    fn test_clear_selection() {
        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![EntityId::new()]);
        assert_eq!(manager.selection().len(), 1);

        manager.clear_selection();
        assert!(manager.selection().is_empty());
    }

    #[test]
    fn test_is_single_selection() {
        let mut manager = PropertiesManager::new();
        manager.set_selection(vec![EntityId::new()]);
        assert!(manager.is_single_selection());

        manager.set_selection(vec![EntityId::new(), EntityId::new()]);
        assert!(!manager.is_single_selection());
    }
}
