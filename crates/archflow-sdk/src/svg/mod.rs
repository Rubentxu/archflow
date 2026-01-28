//! SVG Import/Export module for ArchFlow SDK
//!
//! This module provides functionality to import and export diagrams
//! in SVG format for compatibility with other tools.

use crate::canvas::{Canvas, Shape, ShapeType};
use crate::layers::C4Level;
use archflow_core::{Color, EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for SVG export
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SvgExportOptions {
    /// Width of the exported SVG
    pub width: u32,
    /// Height of the exported SVG
    pub height: u32,
    /// Include background
    pub include_background: bool,
    /// Scale factor
    pub scale: f32,
    /// Include grid
    pub include_grid: bool,
    /// Custom CSS styles
    pub styles: HashMap<String, String>,
}

impl SvgExportOptions {
    /// Creates default export options
    #[inline]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            include_background: true,
            scale: 1.0,
            include_grid: false,
            styles: HashMap::new(),
        }
    }

    /// Sets the scale factor
    #[inline]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Enables background
    #[inline]
    pub fn with_background(mut self, include: bool) -> Self {
        self.include_background = include;
        self
    }

    /// Enables grid
    #[inline]
    pub fn with_grid(mut self, include: bool) -> Self {
        self.include_grid = include;
        self
    }
}

/// Result of SVG import operation
#[derive(Debug, thiserror::Error)]
pub enum SvgImportError {
    #[error("Invalid SVG content: {0}")]
    InvalidContent(String),
    #[error("Unsupported SVG feature: {0}")]
    UnsupportedFeature(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// SVG exporter for ArchFlow documents
#[derive(Debug, Default)]
pub struct SvgExporter;

impl SvgExporter {
    /// Creates a new SVG exporter
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Exports a canvas to SVG format
    ///
    /// # Arguments
    ///
    /// * `canvas` - The canvas to export
    /// * `options` - Export options
    ///
    /// # Returns
    ///
    /// SVG string representation
    pub fn export_to_svg(&self, canvas: &Canvas, options: &SvgExportOptions) -> String {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            options.width,
            options.height,
            options.width,
            options.height
        ));

        // CSS styles
        svg.push_str("<style>");
        for (class, style) in &options.styles {
            svg.push_str(&format!(".{} {{ {} }}", class, style));
        }
        svg.push_str("</style>");

        // Background
        if options.include_background {
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="white"/>"#,
            ));
        }

        // Grid (if enabled)
        if options.include_grid {
            self.render_grid(&mut svg, canvas, options);
        }

        // Export shapes by C4 level
        for level in [
            C4Level::Context,
            C4Level::Container,
            C4Level::Component,
            C4Level::Code,
        ] {
            let shapes = self.get_shapes_for_level(canvas, level);
            for shape in shapes {
                svg.push_str(&self.shape_to_svg_element(shape));
            }
        }

        // SVG footer
        svg.push_str("</svg>");

        svg
    }

    /// Renders grid as SVG pattern
    fn render_grid(&self, svg: &mut String, canvas: &Canvas, options: &SvgExportOptions) {
        let grid_config = canvas.background_renderer().grid_config();
        let spacing = grid_config.spacing * options.scale;

        // Create pattern definition
        svg.push_str(&format!(
            r#"<defs>
                <pattern id="grid" width="{}" height="{}" patternUnits="userSpaceOnUse">
                    <circle cx="{}" cy="{}" r="{}" fill="{}"/>
                </pattern>
            </defs>"#,
            spacing,
            spacing,
            spacing / 2.0,
            spacing / 2.0,
            grid_config.dot_radius,
            Self::color_to_hex(grid_config.dot_color),
        ));

        // Apply pattern
        svg.push_str(r#"<rect width="100%" height="100%" fill="url(#grid)"/>"#);
    }

    /// Gets shapes for a specific C4 level
    fn get_shapes_for_level(&self, canvas: &Canvas, level: C4Level) -> Vec<&Shape> {
        canvas
            .all_shapes()
            .iter()
            .filter(|s| {
                // Filter by layer C4 level if layer info is available
                true // TODO: Implement layer-based filtering
            })
            .cloned()
            .collect()
    }

    /// Converts a shape to an SVG element
    fn shape_to_svg_element(&self, shape: &Shape) -> String {
        let fill = Self::color_to_hex(shape.fill_color);
        let stroke = shape.stroke_color.map_or(String::new(), |c| {
            format!("stroke=\"{}\"", Self::color_to_hex(c))
        });
        let stroke_width = if shape.stroke_width > 0.0 {
            format!("stroke-width=\"{}\"", shape.stroke_width)
        } else {
            String::new()
        };
        let opacity = if shape.opacity < 1.0 {
            format!("opacity=\"{}\"", shape.opacity)
        } else {
            String::new()
        };
        let transform = if shape.rotation != 0.0 {
            let cx = shape.x + shape.width / 2.0;
            let cy = shape.y + shape.height / 2.0;
            format!(r#" transform="rotate({}, {}, {})""#, shape.rotation, cx, cy)
        } else {
            String::new()
        };

        match shape.shape_type {
            ShapeType::Rectangle => {
                format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" {} {} {}{}"/>"#,
                    shape.x,
                    shape.y,
                    shape.width,
                    shape.height,
                    fill,
                    stroke,
                    stroke_width,
                    opacity,
                    transform
                )
            }
            ShapeType::Ellipse => {
                let cx = shape.x + shape.width / 2.0;
                let cy = shape.y + shape.height / 2.0;
                let rx = shape.width / 2.0;
                let ry = shape.height / 2.0;
                format!(
                    r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" {} {} {}{}"/>"#,
                    cx, cy, rx, ry, fill, stroke, stroke_width, opacity, transform
                )
            }
            ShapeType::Line => {
                // For lines, use the bounds to determine endpoints
                format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {} {}{}"/>"#,
                    shape.x,
                    shape.y,
                    shape.x + shape.width,
                    shape.y + shape.height,
                    stroke,
                    stroke_width,
                    transform
                )
            }
            ShapeType::Path => {
                // Simple path representation
                format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-dasharray="5,5"{}{}"/>"#,
                    shape.x, shape.y, shape.width, shape.height, fill, stroke_width, transform
                )
            }
            ShapeType::Text => {
                format!(
                    r#"<text x="{}" y="{}" fill="{}">Shape</text>"#,
                    shape.x,
                    shape.y + shape.height / 2.0,
                    fill
                )
            }
            ShapeType::Image | ShapeType::Group => {
                // Fallback to rectangle
                format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" {} {}{}"/>"#,
                    shape.x,
                    shape.y,
                    shape.width,
                    shape.height,
                    fill,
                    stroke,
                    stroke_width,
                    transform
                )
            }
        }
    }

    /// Converts a Color to hex string
    fn color_to_hex(color: Color) -> String {
        let (r, g, b, _) = color.to_rgba();
        format!(
            "#{:02x}{:02x}{:02x}",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        )
    }
}

/// SVG importer for ArchFlow documents
#[derive(Debug, Default)]
pub struct SvgImporter;

impl SvgImporter {
    /// Creates a new SVG importer
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Imports shapes from SVG content
    ///
    /// # Arguments
    ///
    /// * `svg_content` - The SVG content to parse
    ///
    /// # Returns
    ///
    /// Result containing vector of imported shapes or an error
    pub fn import_from_svg(&self, svg_content: &str) -> Result<Vec<Shape>, SvgImportError> {
        let mut shapes = Vec::new();

        // Simple SVG parsing - look for shape elements
        let elements = [
            ("rect", Self::parse_rect),
            ("circle", Self::parse_circle),
            ("ellipse", Self::parse_ellipse),
            ("line", Self::parse_line),
            ("path", Self::parse_path),
        ];

        for (tag, parser) in elements {
            if let Some(matches) = Self::find_elements(svg_content, tag) {
                for mat in matches {
                    if let Some(shape) = parser(&mat) {
                        shapes.push(shape);
                    }
                }
            }
        }

        Ok(shapes)
    }

    /// Finds all elements of a given tag
    fn find_elements(content: &str, tag: &str) -> Option<std::vec::Vec<std::string::String>> {
        let pattern = format!(r#"<{}[^>]*>"#, tag);
        let regex = regex::Regex::new(&pattern).ok()?;
        let matches: Vec<String> = regex
            .find_iter(content)
            .map(|m| m.as_str().to_string())
            .collect();
        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    /// Parses a rect element
    fn parse_rect(element: &str) -> Option<Shape> {
        let x = Self::extract_float(element, "x")?;
        let y = Self::extract_float(element, "y")?;
        let width = Self::extract_float(element, "width")?;
        let height = Self::extract_float(element, "height")?;

        Some(Shape::new_rectangle(x, y, width, height))
    }

    /// Parses a circle element
    fn parse_circle(element: &str) -> Option<Shape> {
        let cx = Self::extract_float(element, "cx")?;
        let cy = Self::extract_float(element, "cy")?;
        let r = Self::extract_float(element, "r")?;

        Some(Shape::new_ellipse(cx, cy, r, r))
    }

    /// Parses an ellipse element
    fn parse_ellipse(element: &str) -> Option<Shape> {
        let cx = Self::extract_float(element, "cx")?;
        let cy = Self::extract_float(element, "cy")?;
        let rx = Self::extract_float(element, "rx")?;
        let ry = Self::extract_float(element, "ry")?;

        Some(Shape::new_ellipse(cx, cy, rx, ry))
    }

    /// Parses a line element
    fn parse_line(element: &str) -> Option<Shape> {
        let x1 = Self::extract_float(element, "x1")?;
        let y1 = Self::extract_float(element, "y1")?;
        let x2 = Self::extract_float(element, "x2")?;
        let y2 = Self::extract_float(element, "y2")?;

        Some(Shape::new_line(x1, y1, x2, y2))
    }

    /// Parses a path element (simplified)
    fn parse_path(_element: &str) -> Option<Shape> {
        // Path parsing is complex - for now return a placeholder
        Some(Shape::new_rectangle(0.0, 0.0, 100.0, 100.0))
    }

    /// Extracts a float attribute from an element
    fn extract_float(element: &str, attr: &str) -> Option<f32> {
        let pattern = format!(r#"{}=["']?([^"'>\s]+)["']?"#, attr);
        let re = regex::Regex::new(&pattern).ok()?;
        re.captures(element)?.get(1)?.as_str().parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_export_empty() {
        let canvas = Canvas::new(800.0, 600.0);
        let exporter = SvgExporter::new();
        let options = SvgExportOptions::new(800, 600);

        let svg = exporter.export_to_svg(&canvas, &options);

        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("width=\"800\""));
        assert!(svg.contains("height=\"600\""));
    }

    #[test]
    fn test_svg_export_with_shapes() {
        let mut canvas = Canvas::new(800.0, 600.0);
        canvas.create_rectangle(100.0, 100.0, 200.0, 150.0);
        canvas.create_ellipse(400.0, 300.0, 50.0, 75.0);

        let exporter = SvgExporter::new();
        let options = SvgExportOptions::new(800, 600);

        let svg = exporter.export_to_svg(&canvas, &options);

        assert!(svg.contains("<rect"));
        assert!(svg.contains("<ellipse"));
    }

    #[test]
    fn test_svg_import_empty() {
        let importer = SvgImporter::new();
        let svg_content = r#"<svg></svg>"#;

        let shapes = importer.import_from_svg(svg_content);

        assert!(shapes.is_ok());
        assert!(shapes.unwrap().is_empty());
    }

    #[test]
    fn test_svg_import_with_rect() {
        let importer = SvgImporter::new();
        let svg_content = r#"<svg><rect x="10" y="20" width="100" height="50"/></svg>"#;

        let shapes = importer.import_from_svg(svg_content);

        assert!(shapes.is_ok());
        let shapes = shapes.unwrap();
        assert_eq!(shapes.len(), 1);
    }

    #[test]
    fn test_svg_export_options() {
        let options = SvgExportOptions::new(1024, 768)
            .with_scale(2.0)
            .with_background(true)
            .with_grid(true);

        assert_eq!(options.width, 1024);
        assert_eq!(options.height, 768);
        assert!((options.scale - 2.0).abs() < 0.01);
        assert!(options.include_background);
        assert!(options.include_grid);
    }
}
