//! Alignment and distribution module for ArchFlow SDK
//!
//! Provides tools for aligning and distributing shapes on the canvas:
//! - Alignment: Left, Center, Right, Top, Middle, Bottom
//! - Distribution: Horizontal, Vertical
//! - All operations support undo/redo via Command pattern

use crate::canvas::{Canvas, ShapeChanges};
use crate::commands::{Command, CommandResult};
use crate::selection::SelectionDelta;
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error type for alignment operations
#[derive(Debug, thiserror::Error)]
pub enum AlignmentError {
    #[error("Insufficient shapes for alignment: need at least {0}, got {1}")]
    InsufficientShapes(usize, usize),
    #[error("Shape not found: {0}")]
    ShapeNotFound(EntityId),
    #[error("Invalid alignment operation: {0}")]
    InvalidOperation(String),
}

/// Type alias for alignment operation results
pub type AlignmentResult<T> = Result<T, AlignmentError>;

/// Alignment axis (horizontal or vertical)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlignmentAxis {
    /// Horizontal axis (X coordinate)
    Horizontal,
    /// Vertical axis (Y coordinate)
    Vertical,
}

/// Horizontal alignment options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorizontalAlign {
    /// Align to left edges
    Left,
    /// Align to horizontal centers
    Center,
    /// Align to right edges
    Right,
}

/// Vertical alignment options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerticalAlign {
    /// Align to top edges
    Top,
    /// Align to vertical centers (middle)
    Middle,
    /// Align to bottom edges
    Bottom,
}

/// Alignment type combining axis and position
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlignmentType {
    /// Horizontal alignment
    Horizontal(HorizontalAlign),
    /// Vertical alignment
    Vertical(VerticalAlign),
}

/// Distribution axis
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistributionAxis {
    /// Distribute horizontally (along X axis)
    Horizontal,
    /// Distribute vertically (along Y axis)
    Vertical,
}

/// Stores original positions for undo operations
#[derive(Clone, Debug)]
struct OriginalPositions {
    positions: HashMap<EntityId, Vec2>,
}

impl OriginalPositions {
    fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    fn insert(&mut self, id: EntityId, pos: Vec2) {
        self.positions.insert(id, pos);
    }

    fn get(&self, id: EntityId) -> Option<Vec2> {
        self.positions.get(&id).copied()
    }
}

/// Command to align shapes
#[derive(Clone, Debug)]
pub struct AlignCommand {
    /// Shape IDs to align
    shape_ids: Vec<EntityId>,
    /// Type of alignment
    alignment: AlignmentType,
    /// Original positions for undo
    original_positions: OriginalPositions,
    /// Whether command has been executed
    executed: bool,
}

impl AlignCommand {
    /// Creates a new align command
    pub fn new(shape_ids: Vec<EntityId>, alignment: AlignmentType) -> AlignmentResult<Self> {
        if shape_ids.len() < 2 {
            return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
        }

        Ok(Self {
            shape_ids,
            alignment,
            original_positions: OriginalPositions::new(),
            executed: false,
        })
    }

    /// Gets the shape IDs
    pub fn shape_ids(&self) -> &[EntityId] {
        &self.shape_ids
    }

    /// Gets the alignment type
    pub fn alignment(&self) -> AlignmentType {
        self.alignment
    }
}

impl Command for AlignCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Capture original positions
        for &id in &self.shape_ids {
            if let Some(shape) = canvas.get_shape(id) {
                self.original_positions
                    .insert(id, Vec2::new(shape.x, shape.y));
            }
        }

        // Perform alignment
        let result = match self.alignment {
            AlignmentType::Horizontal(align) => {
                align_shapes_horizontal(canvas, &self.shape_ids, align)
            }
            AlignmentType::Vertical(align) => align_shapes_vertical(canvas, &self.shape_ids, align),
        };

        if let Err(e) = result {
            return Err(crate::commands::CommandError::ExecutionFailed(
                e.to_string(),
            ));
        }

        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Restore original positions
        for &id in &self.shape_ids {
            if let Some(original_pos) = self.original_positions.get(id) {
                let changes = ShapeChanges {
                    x: Some(original_pos.x),
                    y: Some(original_pos.y),
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                };
                canvas.update_shape(id, changes);
            }
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        match self.alignment {
            AlignmentType::Horizontal(HorizontalAlign::Left) => "Align left",
            AlignmentType::Horizontal(HorizontalAlign::Center) => "Align center horizontally",
            AlignmentType::Horizontal(HorizontalAlign::Right) => "Align right",
            AlignmentType::Vertical(VerticalAlign::Top) => "Align top",
            AlignmentType::Vertical(VerticalAlign::Middle) => "Align middle",
            AlignmentType::Vertical(VerticalAlign::Bottom) => "Align bottom",
        }
    }
}

/// Command to distribute shapes evenly
#[derive(Clone, Debug)]
pub struct DistributeCommand {
    /// Shape IDs to distribute
    shape_ids: Vec<EntityId>,
    /// Axis to distribute along
    axis: DistributionAxis,
    /// Original positions for undo
    original_positions: OriginalPositions,
    /// Whether command has been executed
    executed: bool,
}

impl DistributeCommand {
    /// Creates a new distribute command
    pub fn new(shape_ids: Vec<EntityId>, axis: DistributionAxis) -> AlignmentResult<Self> {
        if shape_ids.len() < 2 {
            return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
        }

        Ok(Self {
            shape_ids,
            axis,
            original_positions: OriginalPositions::new(),
            executed: false,
        })
    }

    /// Gets the shape IDs
    pub fn shape_ids(&self) -> &[EntityId] {
        &self.shape_ids
    }

    /// Gets the distribution axis
    pub fn axis(&self) -> DistributionAxis {
        self.axis
    }
}

impl Command for DistributeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Capture original positions
        for &id in &self.shape_ids {
            if let Some(shape) = canvas.get_shape(id) {
                self.original_positions
                    .insert(id, Vec2::new(shape.x, shape.y));
            }
        }

        // Perform distribution
        let result = match self.axis {
            DistributionAxis::Horizontal => distribute_shapes_horizontal(canvas, &self.shape_ids),
            DistributionAxis::Vertical => distribute_shapes_vertical(canvas, &self.shape_ids),
        };

        if let Err(e) = result {
            return Err(crate::commands::CommandError::ExecutionFailed(
                e.to_string(),
            ));
        }

        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Restore original positions
        for &id in &self.shape_ids {
            if let Some(original_pos) = self.original_positions.get(id) {
                let changes = ShapeChanges {
                    x: Some(original_pos.x),
                    y: Some(original_pos.y),
                    width: None,
                    height: None,
                    rotation: None,
                    fill_color: None,
                    stroke_color: None,
                    stroke_width: None,
                    opacity: None,
                };
                canvas.update_shape(id, changes);
            }
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        match self.axis {
            DistributionAxis::Horizontal => "Distribute horizontally",
            DistributionAxis::Vertical => "Distribute vertically",
        }
    }
}

/// Aligns shapes horizontally
fn align_shapes_horizontal(
    canvas: &mut Canvas,
    shape_ids: &[EntityId],
    align: HorizontalAlign,
) -> AlignmentResult<()> {
    if shape_ids.len() < 2 {
        return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
    }

    // Collect shape data
    let mut shapes_data: Vec<(EntityId, f32, f32, f32)> = Vec::new();
    for &id in shape_ids {
        if let Some(shape) = canvas.get_shape(id) {
            shapes_data.push((id, shape.x, shape.width, shape.x + shape.width));
        } else {
            return Err(AlignmentError::ShapeNotFound(id));
        }
    }

    // Calculate alignment value
    let align_value = match align {
        HorizontalAlign::Left => shapes_data
            .iter()
            .map(|(_, x, _, _)| *x)
            .fold(f32::INFINITY, f32::min),
        HorizontalAlign::Center => {
            let min_x = shapes_data
                .iter()
                .map(|(_, x, _, _)| *x)
                .fold(f32::INFINITY, f32::min);
            let max_right = shapes_data
                .iter()
                .map(|(_, _, _, right)| *right)
                .fold(f32::NEG_INFINITY, f32::max);
            (min_x + max_right) / 2.0
        }
        HorizontalAlign::Right => shapes_data
            .iter()
            .map(|(_, _, _, right)| *right)
            .fold(f32::NEG_INFINITY, f32::max),
    };

    // Apply alignment
    for (id, x, width, _) in shapes_data {
        let new_x = match align {
            HorizontalAlign::Left => align_value,
            HorizontalAlign::Center => align_value - width / 2.0,
            HorizontalAlign::Right => align_value - width,
        };

        if (new_x - x).abs() > f32::EPSILON {
            let changes = ShapeChanges {
                x: Some(new_x),
                y: None,
                width: None,
                height: None,
                rotation: None,
                fill_color: None,
                stroke_color: None,
                stroke_width: None,
                opacity: None,
            };
            canvas.update_shape(id, changes);
        }
    }

    Ok(())
}

/// Aligns shapes vertically
fn align_shapes_vertical(
    canvas: &mut Canvas,
    shape_ids: &[EntityId],
    align: VerticalAlign,
) -> AlignmentResult<()> {
    if shape_ids.len() < 2 {
        return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
    }

    // Collect shape data
    let mut shapes_data: Vec<(EntityId, f32, f32, f32)> = Vec::new();
    for &id in shape_ids {
        if let Some(shape) = canvas.get_shape(id) {
            shapes_data.push((id, shape.y, shape.height, shape.y + shape.height));
        } else {
            return Err(AlignmentError::ShapeNotFound(id));
        }
    }

    // Calculate alignment value
    let align_value = match align {
        VerticalAlign::Top => shapes_data
            .iter()
            .map(|(_, y, _, _)| *y)
            .fold(f32::INFINITY, f32::min),
        VerticalAlign::Middle => {
            let min_y = shapes_data
                .iter()
                .map(|(_, y, _, _)| *y)
                .fold(f32::INFINITY, f32::min);
            let max_bottom = shapes_data
                .iter()
                .map(|(_, _, _, bottom)| *bottom)
                .fold(f32::NEG_INFINITY, f32::max);
            (min_y + max_bottom) / 2.0
        }
        VerticalAlign::Bottom => shapes_data
            .iter()
            .map(|(_, _, _, bottom)| *bottom)
            .fold(f32::NEG_INFINITY, f32::max),
    };

    // Apply alignment
    for (id, y, height, _) in shapes_data {
        let new_y = match align {
            VerticalAlign::Top => align_value,
            VerticalAlign::Middle => align_value - height / 2.0,
            VerticalAlign::Bottom => align_value - height,
        };

        if (new_y - y).abs() > f32::EPSILON {
            let changes = ShapeChanges {
                x: None,
                y: Some(new_y),
                width: None,
                height: None,
                rotation: None,
                fill_color: None,
                stroke_color: None,
                stroke_width: None,
                opacity: None,
            };
            canvas.update_shape(id, changes);
        }
    }

    Ok(())
}

/// Distributes shapes horizontally with equal spacing
fn distribute_shapes_horizontal(
    canvas: &mut Canvas,
    shape_ids: &[EntityId],
) -> AlignmentResult<()> {
    if shape_ids.len() < 2 {
        return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
    }

    // Collect shape data sorted by current X position
    let mut shapes_data: Vec<(EntityId, f32, f32)> = Vec::new();
    for &id in shape_ids {
        if let Some(shape) = canvas.get_shape(id) {
            shapes_data.push((id, shape.x, shape.width));
        } else {
            return Err(AlignmentError::ShapeNotFound(id));
        }
    }

    // Sort by X position
    shapes_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Calculate total width and spacing
    let min_x = shapes_data.first().map(|(_, x, _)| *x).unwrap_or(0.0);
    let max_right = shapes_data.last().map(|(_, x, w)| x + w).unwrap_or(0.0);
    let total_width = max_right - min_x;

    // Calculate spacing between shapes
    let total_shapes_width: f32 = shapes_data.iter().map(|(_, _, w)| w).sum();
    let available_space = total_width - total_shapes_width;
    let spacing = if shape_ids.len() > 1 {
        available_space / (shape_ids.len() - 1) as f32
    } else {
        0.0
    };

    // Distribute shapes
    let mut current_x = min_x;
    for (id, _, width) in shapes_data {
        let changes = ShapeChanges {
            x: Some(current_x),
            y: None,
            width: None,
            height: None,
            rotation: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };
        canvas.update_shape(id, changes);
        current_x += width + spacing;
    }

    Ok(())
}

/// Distributes shapes vertically with equal spacing
fn distribute_shapes_vertical(canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
    if shape_ids.len() < 2 {
        return Err(AlignmentError::InsufficientShapes(2, shape_ids.len()));
    }

    // Collect shape data sorted by current Y position
    let mut shapes_data: Vec<(EntityId, f32, f32)> = Vec::new();
    for &id in shape_ids {
        if let Some(shape) = canvas.get_shape(id) {
            shapes_data.push((id, shape.y, shape.height));
        } else {
            return Err(AlignmentError::ShapeNotFound(id));
        }
    }

    // Sort by Y position
    shapes_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Calculate total height and spacing
    let min_y = shapes_data.first().map(|(_, y, _)| *y).unwrap_or(0.0);
    let max_bottom = shapes_data.last().map(|(_, y, h)| y + h).unwrap_or(0.0);
    let total_height = max_bottom - min_y;

    // Calculate spacing between shapes
    let total_shapes_height: f32 = shapes_data.iter().map(|(_, _, h)| h).sum();
    let available_space = total_height - total_shapes_height;
    let spacing = if shape_ids.len() > 1 {
        available_space / (shape_ids.len() - 1) as f32
    } else {
        0.0
    };

    // Distribute shapes
    let mut current_y = min_y;
    for (id, _, height) in shapes_data {
        let changes = ShapeChanges {
            x: None,
            y: Some(current_y),
            width: None,
            height: None,
            rotation: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };
        canvas.update_shape(id, changes);
        current_y += height + spacing;
    }

    Ok(())
}

/// Manager for alignment and distribution operations
#[derive(Debug, Default)]
pub struct AlignmentManager;

impl AlignmentManager {
    /// Creates a new alignment manager
    pub fn new() -> Self {
        Self
    }

    /// Creates an alignment command
    pub fn create_align_command(
        &self,
        shape_ids: Vec<EntityId>,
        alignment: AlignmentType,
    ) -> AlignmentResult<AlignCommand> {
        AlignCommand::new(shape_ids, alignment)
    }

    /// Creates a distribution command
    pub fn create_distribute_command(
        &self,
        shape_ids: Vec<EntityId>,
        axis: DistributionAxis,
    ) -> AlignmentResult<DistributeCommand> {
        DistributeCommand::new(shape_ids, axis)
    }

    /// Aligns shapes to the left
    pub fn align_left(&self, canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
        align_shapes_horizontal(canvas, shape_ids, HorizontalAlign::Left)
    }

    /// Aligns shapes to horizontal center
    pub fn align_center_horizontal(
        &self,
        canvas: &mut Canvas,
        shape_ids: &[EntityId],
    ) -> AlignmentResult<()> {
        align_shapes_horizontal(canvas, shape_ids, HorizontalAlign::Center)
    }

    /// Aligns shapes to the right
    pub fn align_right(&self, canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
        align_shapes_horizontal(canvas, shape_ids, HorizontalAlign::Right)
    }

    /// Aligns shapes to the top
    pub fn align_top(&self, canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
        align_shapes_vertical(canvas, shape_ids, VerticalAlign::Top)
    }

    /// Aligns shapes to vertical middle
    pub fn align_middle(&self, canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
        align_shapes_vertical(canvas, shape_ids, VerticalAlign::Middle)
    }

    /// Aligns shapes to the bottom
    pub fn align_bottom(&self, canvas: &mut Canvas, shape_ids: &[EntityId]) -> AlignmentResult<()> {
        align_shapes_vertical(canvas, shape_ids, VerticalAlign::Bottom)
    }

    /// Distributes shapes horizontally
    pub fn distribute_horizontal(
        &self,
        canvas: &mut Canvas,
        shape_ids: &[EntityId],
    ) -> AlignmentResult<()> {
        distribute_shapes_horizontal(canvas, shape_ids)
    }

    /// Distributes shapes vertically
    pub fn distribute_vertical(
        &self,
        canvas: &mut Canvas,
        shape_ids: &[EntityId],
    ) -> AlignmentResult<()> {
        distribute_shapes_vertical(canvas, shape_ids)
    }
}

/// Extension trait for Canvas to support alignment operations
pub trait CanvasAlignmentExt {
    /// Aligns the current selection
    fn align_selection(&mut self, alignment: AlignmentType) -> AlignmentResult<()>;

    /// Distributes the current selection
    fn distribute_selection(&mut self, axis: DistributionAxis) -> AlignmentResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_left() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 150.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(300.0, 200.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager.align_left(&mut canvas, &[id1, id2, id3]).unwrap();

        // All shapes should be aligned to the leftmost X (100.0)
        assert_eq!(canvas.get_shape(id1).unwrap().x, 100.0);
        assert_eq!(canvas.get_shape(id2).unwrap().x, 100.0);
        assert_eq!(canvas.get_shape(id3).unwrap().x, 100.0);
    }

    #[test]
    fn test_align_center_horizontal() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(0.0, 100.0, 100.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 150.0, 100.0, 50.0);

        let manager = AlignmentManager::new();
        manager
            .align_center_horizontal(&mut canvas, &[id1, id2])
            .unwrap();

        // Center of bounds: (0 + 300) / 2 = 150
        // Shape 1 center: 0 + 50 = 50, so x should be 150 - 50 = 100
        // Shape 2 center: 200 + 50 = 250, so x should be 150 - 50 = 100
        // Actually, let's verify the expected behavior:
        // The center of the combined bounds should be at (0 + 300) / 2 = 150
        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();

        // Centers should be the same
        let center1 = shape1.x + shape1.width / 2.0;
        let center2 = shape2.x + shape2.width / 2.0;
        assert!((center1 - center2).abs() < 0.01);
    }

    #[test]
    fn test_align_right() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 150.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(300.0, 200.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager.align_right(&mut canvas, &[id1, id2, id3]).unwrap();

        // All shapes should be aligned to the rightmost edge (300 + 50 = 350)
        // Shape 1: x + 50 = 350 => x = 300
        assert_eq!(canvas.get_shape(id1).unwrap().x, 300.0);
        assert_eq!(canvas.get_shape(id2).unwrap().x, 300.0);
        assert_eq!(canvas.get_shape(id3).unwrap().x, 300.0);
    }

    #[test]
    fn test_align_top() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(150.0, 200.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(200.0, 300.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager.align_top(&mut canvas, &[id1, id2, id3]).unwrap();

        // All shapes should be aligned to the topmost Y (100.0)
        assert_eq!(canvas.get_shape(id1).unwrap().y, 100.0);
        assert_eq!(canvas.get_shape(id2).unwrap().y, 100.0);
        assert_eq!(canvas.get_shape(id3).unwrap().y, 100.0);
    }

    #[test]
    fn test_align_middle() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 0.0, 50.0, 100.0);
        let id2 = canvas.create_rectangle(150.0, 200.0, 50.0, 100.0);

        let manager = AlignmentManager::new();
        manager.align_middle(&mut canvas, &[id1, id2]).unwrap();

        // Centers should be the same
        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();

        let center1 = shape1.y + shape1.height / 2.0;
        let center2 = shape2.y + shape2.height / 2.0;
        assert!((center1 - center2).abs() < 0.01);
    }

    #[test]
    fn test_align_bottom() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(150.0, 200.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(200.0, 300.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager.align_bottom(&mut canvas, &[id1, id2, id3]).unwrap();

        // All shapes should be aligned to the bottommost edge (300 + 50 = 350)
        assert_eq!(canvas.get_shape(id1).unwrap().y, 300.0);
        assert_eq!(canvas.get_shape(id2).unwrap().y, 300.0);
        assert_eq!(canvas.get_shape(id3).unwrap().y, 300.0);
    }

    #[test]
    fn test_distribute_horizontal() {
        let mut canvas = Canvas::new(800.0, 600.0);
        // Create 3 shapes with gaps
        let id1 = canvas.create_rectangle(0.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(200.0, 100.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager
            .distribute_horizontal(&mut canvas, &[id1, id2, id3])
            .unwrap();

        // Total width = 250 (0 to 250), shapes take 150, spacing = (250-150)/2 = 50
        // id1 at 0, id2 at 50+50=100, id3 at 100+50+50=200
        // After distribute: evenly spaced
        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();
        let shape3 = canvas.get_shape(id3).unwrap();

        // Check that spacing is equal
        let gap1 = shape2.x - (shape1.x + shape1.width);
        let gap2 = shape3.x - (shape2.x + shape2.width);
        assert!((gap1 - gap2).abs() < 0.01);
    }

    #[test]
    fn test_distribute_vertical() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 0.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(100.0, 200.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        manager
            .distribute_vertical(&mut canvas, &[id1, id2, id3])
            .unwrap();

        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();
        let shape3 = canvas.get_shape(id3).unwrap();

        // Check that vertical spacing is equal
        let gap1 = shape2.y - (shape1.y + shape1.height);
        let gap2 = shape3.y - (shape2.y + shape2.height);
        assert!((gap1 - gap2).abs() < 0.01);
    }

    #[test]
    fn test_align_requires_min_2_selections() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        let result = manager.align_left(&mut canvas, &[id1]);

        assert!(matches!(
            result,
            Err(AlignmentError::InsufficientShapes(2, 1))
        ));
    }

    #[test]
    fn test_distribute_requires_min_2_selections() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let manager = AlignmentManager::new();
        let result = manager.distribute_horizontal(&mut canvas, &[id1]);

        assert!(matches!(
            result,
            Err(AlignmentError::InsufficientShapes(2, 1))
        ));
    }

    #[test]
    fn test_align_command_undo_redo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 100.0, 50.0, 50.0);

        let original_x1 = canvas.get_shape(id1).unwrap().x;
        let original_x2 = canvas.get_shape(id2).unwrap().x;

        let mut cmd = AlignCommand::new(
            vec![id1, id2],
            AlignmentType::Horizontal(HorizontalAlign::Left),
        )
        .unwrap();

        // Execute
        cmd.execute(&mut canvas).unwrap();
        assert_eq!(
            canvas.get_shape(id1).unwrap().x,
            canvas.get_shape(id2).unwrap().x
        );

        // Undo
        cmd.undo(&mut canvas).unwrap();
        assert_eq!(canvas.get_shape(id1).unwrap().x, original_x1);
        assert_eq!(canvas.get_shape(id2).unwrap().x, original_x2);
    }

    #[test]
    fn test_align_preserves_other_properties() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 200.0, 50.0, 75.0);
        let id2 = canvas.create_rectangle(300.0, 150.0, 100.0, 50.0);

        let original_y1 = canvas.get_shape(id1).unwrap().y;
        let original_y2 = canvas.get_shape(id2).unwrap().y;
        let original_width1 = canvas.get_shape(id1).unwrap().width;
        let original_height1 = canvas.get_shape(id1).unwrap().height;

        let manager = AlignmentManager::new();
        manager.align_left(&mut canvas, &[id1, id2]).unwrap();

        // Y, width, and height should be preserved
        assert_eq!(canvas.get_shape(id1).unwrap().y, original_y1);
        assert_eq!(canvas.get_shape(id2).unwrap().y, original_y2);
        assert_eq!(canvas.get_shape(id1).unwrap().width, original_width1);
        assert_eq!(canvas.get_shape(id1).unwrap().height, original_height1);
    }

    #[test]
    fn test_alignment_type_display() {
        let h_left = AlignmentType::Horizontal(HorizontalAlign::Left);
        let h_center = AlignmentType::Horizontal(HorizontalAlign::Center);
        let h_right = AlignmentType::Horizontal(HorizontalAlign::Right);
        let v_top = AlignmentType::Vertical(VerticalAlign::Top);
        let v_middle = AlignmentType::Vertical(VerticalAlign::Middle);
        let v_bottom = AlignmentType::Vertical(VerticalAlign::Bottom);

        // Just verify they can be created and are distinct
        assert_ne!(h_left, h_center);
        assert_ne!(h_center, h_right);
        assert_ne!(v_top, v_middle);
        assert_ne!(v_middle, v_bottom);
        assert_ne!(h_left, v_top);
    }

    #[test]
    fn test_alignment_manager_creation() {
        let manager = AlignmentManager::new();
        let _ = manager; // Just verify it can be created
    }

    #[test]
    fn test_create_align_command() {
        let manager = AlignmentManager::new();
        let ids = vec![EntityId::new(), EntityId::new()];

        let cmd =
            manager.create_align_command(ids, AlignmentType::Horizontal(HorizontalAlign::Center));

        assert!(cmd.is_ok());
    }

    #[test]
    fn test_create_distribute_command() {
        let manager = AlignmentManager::new();
        let ids = vec![EntityId::new(), EntityId::new()];

        let cmd = manager.create_distribute_command(ids, DistributionAxis::Horizontal);

        assert!(cmd.is_ok());
    }

    #[test]
    fn test_distribute_command_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(0.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 100.0, 50.0, 50.0);

        let original_x1 = canvas.get_shape(id1).unwrap().x;
        let original_x2 = canvas.get_shape(id2).unwrap().x;

        let mut cmd = DistributeCommand::new(vec![id1, id2], DistributionAxis::Horizontal).unwrap();

        // Execute
        cmd.execute(&mut canvas).unwrap();

        // Undo
        cmd.undo(&mut canvas).unwrap();

        // Positions should be restored
        assert_eq!(canvas.get_shape(id1).unwrap().x, original_x1);
        assert_eq!(canvas.get_shape(id2).unwrap().x, original_x2);
    }

    #[test]
    fn test_align_command_with_nonexistent_shape() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let nonexistent_id = EntityId::new();

        let manager = AlignmentManager::new();
        let result = manager.align_left(&mut canvas, &[id1, nonexistent_id]);

        assert!(matches!(result, Err(AlignmentError::ShapeNotFound(_))));
    }
}
