//! PathEngine - Operaciones sobre paths usando kurbo
//!
//! Proporciona conversión entre diferentes representaciones de paths:
//! - Elementos de path (MoveTo, LineTo, QuadTo, etc.)
//! - Segmentos de kurbo
//! - Puntos discretizados

use crate::Vec2;
use kurbo::{BezPath, PathSeg, Point as KurboPoint, Shape};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Tipo de elemento en un path (representación de alto nivel)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathElement {
    /// Mover a posición sin dibujar
    MoveTo(Vec2),
    /// Línea recta
    LineTo(Vec2),
    /// Bézier cuadrática (punto de control, punto final)
    QuadTo(Vec2, Vec2),
    /// Bézier cúbica (dos puntos de control, punto final)
    CurveTo(Vec2, Vec2, Vec2),
    /// Cerrar path
    Close,
}

/// Tipo de arco
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArcType {
    /// Arco elíptico estándar
    Elliptical,
    /// Arco circular
    Circular,
}

/// Arco con parámetros
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArcElement {
    /// Punto de inicio
    pub start: Vec2,
    /// Punto final
    pub end: Vec2,
    /// Radio en eje X
    pub rx: f32,
    /// Radio en eje Y
    pub ry: f32,
    /// Rotación del eje X
    pub x_axis_rotation: f32,
    /// Tipo de arco (para SVG compatibility)
    pub large_arc: bool,
    /// Dirección del arco
    pub sweep: bool,
}

/// Configuración para simplificación de paths
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimplifyConfig {
    /// Tolerancia máxima para puntos consecutivos
    pub tolerance: f32,
    /// Si preservar esquinas (puntos de inflexión)
    pub preserve_corners: bool,
}

impl Default for SimplifyConfig {
    fn default() -> Self {
        Self {
            tolerance: 1.0,
            preserve_corners: true,
        }
    }
}

/// Engine para operaciones sobre paths
#[derive(Debug, Default, Clone)]
pub struct PathEngine {
    simplify_config: SimplifyConfig,
}

impl PathEngine {
    /// Crear engine con configuración de simplificación
    #[inline]
    pub fn with_config(config: SimplifyConfig) -> Self {
        Self {
            simplify_config: config,
        }
    }

    /// Actualizar configuración
    pub fn set_simplify_config(&mut self, config: SimplifyConfig) {
        self.simplify_config = config;
    }

    /// Crear path desde elementos
    pub fn from_elements(&self, elements: &[PathElement]) -> BezPath {
        let mut path = BezPath::new();
        for elem in elements {
            match elem {
                PathElement::MoveTo(p) => {
                    path.move_to(self.to_kurbo_point(*p));
                }
                PathElement::LineTo(p) => {
                    path.line_to(self.to_kurbo_point(*p));
                }
                PathElement::QuadTo(cp, end) => {
                    path.quad_to(self.to_kurbo_point(*cp), self.to_kurbo_point(*end));
                }
                PathElement::CurveTo(c1, c2, end) => {
                    path.curve_to(
                        self.to_kurbo_point(*c1),
                        self.to_kurbo_point(*c2),
                        self.to_kurbo_point(*end),
                    );
                }
                PathElement::Close => {
                    path.close_path();
                }
            }
        }
        path
    }

    /// Extraer elementos del path (representación canónica)
    pub fn to_elements(&self, path: &BezPath) -> Vec<PathElement> {
        let mut elements = Vec::new();

        for elem in path.iter() {
            match elem {
                kurbo::PathEl::MoveTo(p) => {
                    elements.push(PathElement::MoveTo(self.from_kurbo_point(p)));
                }
                kurbo::PathEl::LineTo(p) => {
                    elements.push(PathElement::LineTo(self.from_kurbo_point(p)));
                }
                kurbo::PathEl::QuadTo(c, p) => {
                    elements.push(PathElement::QuadTo(
                        self.from_kurbo_point(c),
                        self.from_kurbo_point(p),
                    ));
                }
                kurbo::PathEl::CurveTo(c1, c2, p) => {
                    elements.push(PathElement::CurveTo(
                        self.from_kurbo_point(c1),
                        self.from_kurbo_point(c2),
                        self.from_kurbo_point(p),
                    ));
                }
                kurbo::PathEl::ClosePath => {
                    elements.push(PathElement::Close);
                }
            }
        }

        elements
    }

    /// Convertir a segmentos de kurbo
    pub fn to_segments(&self, path: &BezPath) -> Vec<PathSeg> {
        path.segments().collect()
    }

    /// Crear path desde segmentos
    pub fn from_segments(&self, segments: &[PathSeg]) -> BezPath {
        let mut path = BezPath::new();
        for seg in segments {
            match seg {
                PathSeg::Line(line) => {
                    path.move_to(line.p0);
                    path.line_to(line.p1);
                }
                PathSeg::Quad(quad) => {
                    path.move_to(quad.p0);
                    path.quad_to(quad.p1, quad.p2);
                }
                PathSeg::Cubic(cubic) => {
                    path.move_to(cubic.p0);
                    path.curve_to(cubic.p1, cubic.p2, cubic.p3);
                }
            }
        }
        path
    }

    /// Extraer todos los puntos del path (para hit testing)
    pub fn extract_points(&self, path: &BezPath, segments: usize) -> Vec<Vec2> {
        let mut points = Vec::new();
        self.extract_points_into(path, segments, &mut points);
        points
    }

    /// Extraer puntos a un vector existente
    pub fn extract_points_into(&self, path: &BezPath, segments: usize, output: &mut Vec<Vec2>) {
        for seg in path.segments() {
            match seg {
                PathSeg::Line(line) => {
                    output.push(self.from_kurbo_point(line.p0));
                    output.push(self.from_kurbo_point(line.p1));
                }
                PathSeg::Quad(quad) => {
                    self.quad_bezier_points(
                        self.from_kurbo_point(quad.p0),
                        self.from_kurbo_point(quad.p1),
                        self.from_kurbo_point(quad.p2),
                        segments,
                        output,
                    );
                }
                PathSeg::Cubic(cubic) => {
                    self.cubic_bezier_points(
                        self.from_kurbo_point(cubic.p0),
                        self.from_kurbo_point(cubic.p1),
                        self.from_kurbo_point(cubic.p2),
                        self.from_kurbo_point(cubic.p3),
                        segments,
                        output,
                    );
                }
            }
        }
    }

    /// Puntos de Bézier cuadrática
    fn quad_bezier_points(
        &self,
        start: Vec2,
        control: Vec2,
        end: Vec2,
        segments: usize,
        output: &mut Vec<Vec2>,
    ) {
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let t2 = t * t;
            output.push(start * mt2 + control * (2.0 * mt * t) + end * t2);
        }
    }

    /// Puntos de Bézier cúbica
    fn cubic_bezier_points(
        &self,
        start: Vec2,
        c1: Vec2,
        c2: Vec2,
        end: Vec2,
        segments: usize,
        output: &mut Vec<Vec2>,
    ) {
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;
            output.push(start * mt3 + c1 * (3.0 * mt2 * t) + c2 * (3.0 * mt * t2) + end * t3);
        }
    }

    /// Simplificar path usando algoritmo Ramer-Douglas-Peucker
    pub fn simplify(&self, path: &BezPath, config: Option<SimplifyConfig>) -> BezPath {
        let cfg = config.unwrap_or(self.simplify_config);
        let points = self.extract_points(path, 10);

        if points.len() < 3 {
            return path.clone();
        }

        let simplified = self.rdp_simplify(&points, cfg.tolerance, cfg.preserve_corners);
        self.from_elements(&simplified)
    }

    /// Algoritmo Ramer-Douglas-Peucker
    fn rdp_simplify(
        &self,
        points: &[Vec2],
        tolerance: f32,
        preserve_corners: bool,
    ) -> Vec<PathElement> {
        if points.len() < 2 {
            return vec![PathElement::MoveTo(
                points.get(0).copied().unwrap_or_default(),
            )];
        }

        let mut result = Vec::new();
        self.rdp_impl(points, 0, points.len() - 1, tolerance, &mut result);

        // Preservar puntos de esquina (ángulos pronunciados)
        if preserve_corners {
            result = self.preserve_corners(&result);
        }

        result
    }

    fn rdp_impl(
        &self,
        points: &[Vec2],
        start: usize,
        end: usize,
        tolerance: f32,
        result: &mut Vec<PathElement>,
    ) {
        if end <= start + 1 {
            return;
        }

        let mut max_dist = 0.0;
        let mut max_idx = start;

        let p1 = points[start];
        let p2 = points[end];

        for i in (start + 1)..end {
            let dist = self.perpendicular_distance(points[i], p1, p2);
            if dist > max_dist {
                max_dist = dist;
                max_idx = i;
            }
        }

        if max_dist > tolerance {
            self.rdp_impl(points, start, max_idx, tolerance, result);
            result.push(PathElement::LineTo(points[max_idx]));
            self.rdp_impl(points, max_idx, end, tolerance, result);
        }
    }

    fn perpendicular_distance(&self, point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
        let dx = line_end.x - line_start.x;
        let dy = line_end.y - line_start.y;
        let len_sq = dx * dx + dy * dy;

        if len_sq < f32::EPSILON {
            let dx = point.x - line_start.x;
            let dy = point.y - line_start.y;
            return (dx * dx + dy * dy).sqrt();
        }

        let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        let proj_x = line_start.x + t * dx;
        let proj_y = line_start.y + t * dy;

        let dx = point.x - proj_x;
        let dy = point.y - proj_y;
        (dx * dx + dy * dy).sqrt()
    }

    fn preserve_corners(&self, elements: &[PathElement]) -> Vec<PathElement> {
        if elements.len() < 3 {
            return elements.to_vec();
        }

        let mut result = Vec::with_capacity(elements.len());
        result.push(elements[0].clone());

        for i in 1..elements.len() {
            let prev = &elements[i - 1];
            let curr = &elements[i];
            let next = elements.get(i + 1);

            let prev_line = match prev {
                PathElement::LineTo(p) => Some(p),
                _ => None,
            };
            let curr_line = match curr {
                PathElement::LineTo(p) => Some(p),
                _ => None,
            };
            let next_line = match next {
                Some(PathElement::LineTo(p)) => Some(p),
                _ => None,
            };

            let is_corner = if let (Some(p1), Some(p2)) = (prev_line, curr_line) {
                if let Some(p3) = next_line {
                    let v1 = Vec2::new(p1.x - p2.x, p1.y - p2.y);
                    let v2 = Vec2::new(p3.x - p2.x, p3.y - p2.y);
                    let len1 = v1.length();
                    let len2 = v2.length();
                    if len1 < f32::EPSILON || len2 < f32::EPSILON {
                        false
                    } else {
                        let v1 = v1 / len1;
                        let v2 = v2 / len2;
                        let dot = v1.dot(v2);
                        dot < 0.95 // Ángulo mayor a ~18 grados
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if is_corner {
                result.push(curr.clone());
            }
        }

        result.push(elements.last().unwrap().clone());
        result
    }

    /// Obtener bounding box del path
    pub fn bounds(&self, path: &BezPath) -> kurbo::Rect {
        path.bounding_box()
    }

    /// Verificar si path está vacío
    pub fn is_empty(&self, path: &BezPath) -> bool {
        path.is_empty()
    }

    /// Obtener longitud del path
    pub fn length(&self, path: &BezPath) -> f64 {
        path.perimeter(1e-2)
    }

    /// Convertir kurbo::Point a Vec2
    #[inline]
    pub fn from_kurbo_point(&self, p: KurboPoint) -> Vec2 {
        Vec2::new(p.x as f32, p.y as f32)
    }

    /// Convertir Vec2 a kurbo::Point
    #[inline]
    pub fn to_kurbo_point(&self, v: Vec2) -> KurboPoint {
        KurboPoint::new(v.x as f64, v.y as f64)
    }

    /// Crear rectángulo como path
    pub fn rect_path(&self, x: f32, y: f32, width: f32, height: f32) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(self.to_kurbo_point(Vec2::new(x, y)));
        path.line_to(self.to_kurbo_point(Vec2::new(x + width, y)));
        path.line_to(self.to_kurbo_point(Vec2::new(x + width, y + height)));
        path.line_to(self.to_kurbo_point(Vec2::new(x, y + height)));
        path.close_path();
        path
    }

    /// Crear elipse como path
    pub fn ellipse_path(&self, cx: f32, cy: f32, rx: f32, ry: f32, segments: usize) -> BezPath {
        let mut path = BezPath::new();
        let two_pi = 2.0 * PI as f64;

        for i in 0..segments {
            let angle = (i as f64 / segments as f64) * two_pi;
            let x = cx as f64 + rx as f64 * angle.cos();
            let y = cy as f64 + ry as f64 * angle.sin();

            if i == 0 {
                path.move_to(KurboPoint::new(x, y));
            } else {
                path.line_to(KurboPoint::new(x, y));
            }
        }
        path.close_path();
        path
    }

    /// Crear línea como path
    pub fn line_path(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(KurboPoint::new(x1 as f64, y1 as f64));
        path.line_to(KurboPoint::new(x2 as f64, y2 as f64));
        path
    }

    /// Crear arco como path (aproximación)
    pub fn arc_path(
        &self,
        start: Vec2,
        end: Vec2,
        _rx: f32,
        _ry: f32,
        _x_axis_rotation: f32,
        _large_arc: bool,
        _sweep: bool,
    ) -> BezPath {
        let p0 = self.to_kurbo_point(start);
        let p1 = self.to_kurbo_point(end);

        let mut path = BezPath::new();
        path.move_to(p0);
        // Aproximación simple - usar línea para MVP
        path.line_to(p1);

        path
    }

    /// Aplicar transformación afín al path
    pub fn transform(&self, path: &BezPath, transform: &kurbo::Affine) -> BezPath {
        let transform = *transform; // Dereference to get owned value
        let mut new_path = BezPath::new();
        for elem in path.iter() {
            match elem {
                kurbo::PathEl::MoveTo(p) => {
                    let transformed = transform * p;
                    new_path.move_to(transformed);
                }
                kurbo::PathEl::LineTo(p) => {
                    let transformed = transform * p;
                    new_path.line_to(transformed);
                }
                kurbo::PathEl::QuadTo(c, p) => {
                    let tc = transform * c;
                    let tp = transform * p;
                    new_path.quad_to(tc, tp);
                }
                kurbo::PathEl::CurveTo(c1, c2, p) => {
                    let tc1 = transform * c1;
                    let tc2 = transform * c2;
                    let tp = transform * p;
                    new_path.curve_to(tc1, tc2, tp);
                }
                kurbo::PathEl::ClosePath => {
                    new_path.close_path();
                }
            }
        }
        new_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_elements() {
        let engine = PathEngine::default();
        let elements = vec![
            PathElement::MoveTo(Vec2::new(0.0, 0.0)),
            PathElement::LineTo(Vec2::new(100.0, 0.0)),
            PathElement::QuadTo(Vec2::new(100.0, 50.0), Vec2::new(100.0, 100.0)),
            PathElement::Close,
        ];

        let path = engine.from_elements(&elements);
        assert!(!engine.is_empty(&path));
    }

    #[test]
    fn test_to_elements() {
        let engine = PathEngine::default();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));
        path.quad_to((50.0, 50.0), (100.0, 100.0));
        path.close_path();

        let elements = engine.to_elements(&path);
        assert_eq!(elements.len(), 4);
        assert_eq!(elements[0], PathElement::MoveTo(Vec2::new(0.0, 0.0)));
        assert_eq!(elements[3], PathElement::Close);
    }

    #[test]
    fn test_rect_path() {
        let engine = PathEngine::default();
        let path = engine.rect_path(0.0, 0.0, 100.0, 50.0);
        let bounds = engine.bounds(&path);

        assert_eq!(bounds.x0, 0.0);
        assert_eq!(bounds.y0, 0.0);
        assert_eq!(bounds.x1, 100.0);
        assert_eq!(bounds.y1, 50.0);
    }

    #[test]
    fn test_ellipse_path() {
        let engine = PathEngine::default();
        let path = engine.ellipse_path(50.0, 50.0, 40.0, 30.0, 32);
        let bounds = engine.bounds(&path);

        assert!(bounds.x0 < 50.0);
        assert!(bounds.x1 > 50.0);
        assert!(bounds.y0 < 50.0);
        assert!(bounds.y1 > 50.0);
    }

    #[test]
    fn test_extract_points() {
        let engine = PathEngine::default();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));

        let points = engine.extract_points(&path, 10);
        assert!(points.len() >= 2);
    }

    #[test]
    fn test_simplify() {
        let engine = PathEngine::default();
        // Crear path con muchos puntos en línea recta
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        for i in 1..=100 {
            path.line_to((i as f64, 0.0));
        }

        let simplified = engine.simplify(
            &path,
            Some(SimplifyConfig {
                tolerance: 5.0,
                preserve_corners: true,
            }),
        );

        // Debería tener menos elementos
        let original_elements: Vec<_> = path.iter().collect();
        let simplified_elements: Vec<_> = simplified.iter().collect();
        assert!(simplified_elements.len() < original_elements.len());
    }

    #[test]
    fn test_path_length() {
        let engine = PathEngine::default();
        let path = engine.line_path(0.0, 0.0, 100.0, 0.0);
        let length = engine.length(&path);

        assert!((length - 100.0).abs() < 1.0);
    }
}
