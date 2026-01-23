//! GeometryEngine - Motor de geometría con kurbo para 2D graphics
//!
//! Proporciona operaciones geométricas incluyendo:
//! - Cálculo de bounding boxes
//! - Conversión a paths de kurbo
//! - Transformaciones geométricas

use crate::Vec2;
use kurbo::{BezPath, Rect as KurboRect};
use std::f32::consts::PI;

/// Resultado de intersección entre formas
#[derive(Debug, Clone, PartialEq)]
pub struct IntersectionResult {
    /// Si hay intersección
    pub intersects: bool,
    /// Punto de intersección más cercano (si aplica)
    pub closest_point: Option<Vec2>,
    /// Lista de puntos de intersección
    pub points: Vec<Vec2>,
    /// Distancia desde un punto de referencia
    pub distance: f32,
}

impl Default for IntersectionResult {
    fn default() -> Self {
        Self {
            intersects: false,
            closest_point: None,
            points: Vec::new(),
            distance: f32::INFINITY,
        }
    }
}

/// Configuración para discretización de curvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiscretizeConfig {
    /// Número de segmentos para curvas
    pub curve_segments: usize,
    /// Tolerancia para simplificación
    pub tolerance: f32,
    /// Si usar adaptive sampling
    pub adaptive: bool,
}

impl Default for DiscretizeConfig {
    fn default() -> Self {
        Self {
            curve_segments: 32,
            tolerance: 0.5,
            adaptive: true,
        }
    }
}

/// Motor de operaciones geométricas
#[derive(Debug, Default, Clone)]
pub struct GeometryEngine {
    config: DiscretizeConfig,
}

impl GeometryEngine {
    /// Crear engine con configuración personalizada
    #[inline]
    pub fn with_config(config: DiscretizeConfig) -> Self {
        Self { config }
    }

    /// Actualizar configuración
    pub fn set_config(&mut self, config: DiscretizeConfig) {
        self.config = config;
    }

    /// Calcular distancia entre dos puntos
    #[inline]
    pub fn distance(&self, a: Vec2, b: Vec2) -> f32 {
        (b - a).length()
    }

    /// Calcular distancia al cuadrado (más eficiente)
    #[inline]
    pub fn distance_sq(&self, a: Vec2, b: Vec2) -> f32 {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        dx * dx + dy * dy
    }

    /// Calcular ángulo entre dos puntos
    #[inline]
    pub fn angle(&self, a: Vec2, b: Vec2) -> f32 {
        (b.y - a.y).atan2(b.x - a.x)
    }

    /// Calcular punto en línea dado parámetro t (0-1)
    #[inline]
    pub fn point_on_segment(&self, start: Vec2, end: Vec2, t: f32) -> Vec2 {
        start + (end - start) * t.clamp(0.0, 1.0)
    }

    /// Verificar si punto está en línea (con tolerancia)
    pub fn point_on_line(
        &self,
        point: Vec2,
        line_start: Vec2,
        line_end: Vec2,
        tolerance: f32,
    ) -> bool {
        let d1 = self.distance(point, line_start);
        let d2 = self.distance(point, line_end);
        let line_len = self.distance(line_start, line_end);
        if line_len < f32::EPSILON {
            return d1 <= tolerance;
        }
        let min_dist = (d1 + d2 - line_len).abs();
        min_dist <= tolerance
    }

    /// Calcular normal de segmento (vector perpendicular)
    #[inline]
    pub fn segment_normal(&self, start: Vec2, end: Vec2) -> Vec2 {
        let d = end - start;
        let len = d.length();
        if len < f32::EPSILON {
            Vec2::new(0.0, 1.0)
        } else {
            Vec2::new(-d.y / len, d.x / len)
        }
    }

    /// Calcular elipse discretizada
    pub fn ellipse_points(&self, center: Vec2, rx: f32, ry: f32, segments: usize) -> Vec<Vec2> {
        if segments == 0 {
            return Vec::new();
        }
        let mut points = Vec::with_capacity(segments);
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            points.push(Vec2::new(
                center.x + rx * angle.cos(),
                center.y + ry * angle.sin(),
            ));
        }
        points
    }

    /// Aproximar curva de Bézier cuadrática
    pub fn quadratic_bezier(
        &self,
        start: Vec2,
        control: Vec2,
        end: Vec2,
        segments: usize,
    ) -> Vec<Vec2> {
        if segments == 0 {
            return Vec::new();
        }
        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let t2 = t * t;
            points.push(start * mt2 + control * (2.0 * mt * t) + end * t2);
        }
        points
    }

    /// Aproximar curva de Bézier cúbica
    pub fn cubic_bezier(
        &self,
        start: Vec2,
        c1: Vec2,
        c2: Vec2,
        end: Vec2,
        segments: usize,
    ) -> Vec<Vec2> {
        if segments == 0 {
            return Vec::new();
        }
        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;
            points.push(start * mt3 + c1 * (3.0 * mt2 * t) + c2 * (3.0 * mt * t2) + end * t3);
        }
        points
    }

    /// Calcular bounding box de un conjunto de puntos
    pub fn bounds_of_points(&self, points: &[Vec2]) -> Option<KurboRect> {
        if points.is_empty() {
            return None;
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Some(KurboRect::new(
            min_x as f64,
            min_y as f64,
            max_x as f64,
            max_y as f64,
        ))
    }

    /// Convertir Vec2 a kurbo::Point
    #[inline]
    pub fn to_kurbo_point(&self, v: Vec2) -> kurbo::Point {
        kurbo::Point::new(v.x as f64, v.y as f64)
    }

    /// Convertir kurbo::Point a Vec2
    #[inline]
    pub fn from_kurbo_point(&self, p: kurbo::Point) -> Vec2 {
        Vec2::new(p.x as f32, p.y as f32)
    }

    /// Crear kurbo::Rect desde Vec2 min y max
    #[inline]
    pub fn to_kurbo_rect(&self, min: Vec2, max: Vec2) -> KurboRect {
        KurboRect::new(min.x as f64, min.y as f64, max.x as f64, max.y as f64)
    }

    /// Calcular centro de un rectángulo
    #[inline]
    pub fn rect_center(&self, rect: KurboRect) -> Vec2 {
        Vec2::new(
            (rect.x0 + rect.x1) as f32 / 2.0,
            (rect.y0 + rect.y1) as f32 / 2.0,
        )
    }

    /// Verificar si punto está dentro de rectángulo
    #[inline]
    pub fn point_in_rect(&self, point: Vec2, rect: KurboRect) -> bool {
        let x = point.x as f64;
        let y = point.y as f64;
        x >= rect.x0 && x <= rect.x1 && y >= rect.y0 && y <= rect.y1
    }

    /// Calcular área de polígono (para signed area y orientación)
    pub fn polygon_area(&self, points: &[Vec2]) -> f32 {
        if points.len() < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            area += points[i].x * points[j].y;
            area -= points[j].x * points[i].y;
        }
        area / 2.0
    }

    /// Verificar si polígono es convexo
    pub fn is_polygon_convex(&self, points: &[Vec2]) -> bool {
        if points.len() < 4 {
            return true;
        }
        let mut prev_cross: f32 = 0.0;
        for i in 0..points.len() {
            let p0 = points[i];
            let p1 = points[(i + 1) % points.len()];
            let p2 = points[(i + 2) % points.len()];

            let v1 = p1 - p0;
            let v2 = p2 - p1;
            let cross = v1.x * v2.y - v1.y * v2.x;

            if cross.abs() > f32::EPSILON {
                if prev_cross.abs() > f32::EPSILON && cross.signum() != prev_cross.signum() {
                    return false;
                }
                prev_cross = cross;
            }
        }
        true
    }

    /// Encontrar el punto más cercano en un polígono
    pub fn closest_point_on_polygon(&self, point: Vec2, polygon: &[Vec2]) -> Vec2 {
        let mut closest = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut min_dist = f32::INFINITY;

        for i in 0..polygon.len() {
            let p1 = polygon[i];
            let p2 = polygon[(i + 1) % polygon.len()];
            let closest_on_seg = self.closest_point_on_segment(point, p1, p2);
            let dist = self.distance(point, closest_on_seg);
            if dist < min_dist {
                min_dist = dist;
                closest = closest_on_seg;
            }
        }
        closest
    }

    /// Encontrar punto más cercano en un segmento
    pub fn closest_point_on_segment(&self, point: Vec2, seg_start: Vec2, seg_end: Vec2) -> Vec2 {
        let seg_vec = seg_end - seg_start;
        let pt_vec = point - seg_start;
        let seg_len_sq = seg_vec.length_squared();

        if seg_len_sq < f32::EPSILON {
            return seg_start;
        }

        let t = (pt_vec.dot(seg_vec) / seg_len_sq).clamp(0.0, 1.0);
        seg_start + seg_vec * t
    }

    /// Calcular distancia de punto a segmento
    pub fn distance_to_segment(&self, point: Vec2, seg_start: Vec2, seg_end: Vec2) -> f32 {
        let closest = self.closest_point_on_segment(point, seg_start, seg_end);
        self.distance(point, closest)
    }

    /// Discretizar un BezPath a puntos
    pub fn discretize_path(&self, path: &BezPath) -> Vec<Vec2> {
        let mut points = Vec::new();
        self.discretize_path_into(path, &mut points);
        points
    }

    /// Discretizar un BezPath a un vector existente
    pub fn discretize_path_into(&self, path: &BezPath, output: &mut Vec<Vec2>) {
        for seg in path.segments() {
            match seg {
                kurbo::PathSeg::Line(line) => {
                    output.push(self.from_kurbo_point(line.p0));
                    output.push(self.from_kurbo_point(line.p1));
                }
                kurbo::PathSeg::Quad(quad) => {
                    let start = self.from_kurbo_point(quad.p0);
                    let control = self.from_kurbo_point(quad.p1);
                    let end = self.from_kurbo_point(quad.p2);
                    let bezier_points =
                        self.quadratic_bezier(start, control, end, self.config.curve_segments);
                    output.extend_from_slice(&bezier_points);
                }
                kurbo::PathSeg::Cubic(cubic) => {
                    let start = self.from_kurbo_point(cubic.p0);
                    let c1 = self.from_kurbo_point(cubic.p1);
                    let c2 = self.from_kurbo_point(cubic.p2);
                    let end = self.from_kurbo_point(cubic.p3);
                    let bezier_points =
                        self.cubic_bezier(start, c1, c2, end, self.config.curve_segments);
                    output.extend_from_slice(&bezier_points);
                }
            }
        }
    }

    /// Crear path desde puntos (polígono cerrado)
    pub fn path_from_points(&self, points: &[Vec2], closed: bool) -> BezPath {
        let mut path = BezPath::new();
        if points.is_empty() {
            return path;
        }

        path.move_to(self.to_kurbo_point(points[0]));
        for point in &points[1..] {
            path.line_to(self.to_kurbo_point(*point));
        }
        if closed {
            path.close_path();
        }
        path
    }

    /// Verificar si dos rectángulos se intersectan
    #[inline]
    pub fn rects_intersect(&self, r1: KurboRect, r2: KurboRect) -> bool {
        !(r1.x1 < r2.x0 || r1.x0 > r2.x1 || r1.y1 < r2.y0 || r1.y0 > r2.y1)
    }

    /// Calcular unión de dos rectángulos
    #[inline]
    pub fn rect_union(&self, r1: KurboRect, r2: KurboRect) -> KurboRect {
        KurboRect::new(
            r1.x0.min(r2.x0),
            r1.y0.min(r2.y0),
            r1.x1.max(r2.x1),
            r1.y1.max(r2.y1),
        )
    }

    /// Calcular intersección de dos rectángulos
    #[inline]
    pub fn rect_intersection(&self, r1: KurboRect, r2: KurboRect) -> Option<KurboRect> {
        let x0 = r1.x0.max(r2.x0);
        let y0 = r1.y0.max(r2.y0);
        let x1 = r1.x1.min(r2.x1);
        let y1 = r1.y1.min(r2.y1);

        if x0 < x1 && y0 < y1 {
            Some(KurboRect::new(x0, y0, x1, y1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let engine = GeometryEngine::default();
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(engine.distance(a, b), 5.0);
    }

    #[test]
    fn test_point_on_segment() {
        let engine = GeometryEngine::default();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(10.0, 0.0);

        assert_eq!(engine.point_on_segment(start, end, 0.0), start);
        assert_eq!(
            engine.point_on_segment(start, end, 0.5),
            Vec2::new(5.0, 0.0)
        );
        assert_eq!(engine.point_on_segment(start, end, 1.0), end);
    }

    #[test]
    fn test_ellipse_points() {
        let engine = GeometryEngine::default();
        let center = Vec2::new(100.0, 100.0);
        let points = engine.ellipse_points(center, 50.0, 30.0, 4);

        assert_eq!(points.len(), 4);
        // Verificar que los puntos están en la elipse
        assert!((points[0].x - 150.0).abs() < 0.001);
        assert!((points[0].y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_quadratic_bezier() {
        let engine = GeometryEngine::default();
        let start = Vec2::new(0.0, 0.0);
        let control = Vec2::new(50.0, 100.0);
        let end = Vec2::new(100.0, 0.0);

        let points = engine.quadratic_bezier(start, control, end, 4);
        assert_eq!(points.len(), 5);
        assert_eq!(points[0], start);
        assert_eq!(points[4], end);
    }

    #[test]
    fn test_polygon_area() {
        let engine = GeometryEngine::default();
        // Square with side 2, area = 4
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        assert!((engine.polygon_area(&square).abs() - 4.0) < 0.001);
    }

    #[test]
    fn test_is_polygon_convex() {
        let engine = GeometryEngine::default();

        // Square - convex
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        assert!(engine.is_polygon_convex(&square));

        // L-shape - concave
        let l_shape = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        assert!(!engine.is_polygon_convex(&l_shape));
    }

    #[test]
    fn test_point_in_rect() {
        let engine = GeometryEngine::default();
        let rect = KurboRect::new(0.0, 0.0, 100.0, 100.0);

        assert!(engine.point_in_rect(Vec2::new(50.0, 50.0), rect));
        assert!(engine.point_in_rect(Vec2::new(0.0, 0.0), rect));
        assert!(!engine.point_in_rect(Vec2::new(101.0, 50.0), rect));
        assert!(!engine.point_in_rect(Vec2::new(50.0, 101.0), rect));
    }

    #[test]
    fn test_rects_intersect() {
        let engine = GeometryEngine::default();
        let r1 = KurboRect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = KurboRect::new(50.0, 50.0, 150.0, 150.0);
        let r3 = KurboRect::new(200.0, 200.0, 300.0, 300.0);

        assert!(engine.rects_intersect(r1, r2));
        assert!(!engine.rects_intersect(r1, r3));
    }

    #[test]
    fn test_discretize_path() {
        let engine = GeometryEngine::default();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));
        path.quad_to((50.0, 50.0), (100.0, 100.0));

        let points = engine.discretize_path(&path);
        assert!(!points.is_empty());
        assert_eq!(points[0], Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_closest_point_on_segment() {
        let engine = GeometryEngine::default();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(10.0, 0.0);

        // Point on segment
        let closest = engine.closest_point_on_segment(Vec2::new(5.0, 5.0), start, end);
        assert_eq!(closest, Vec2::new(5.0, 0.0));

        // Point before segment
        let closest = engine.closest_point_on_segment(Vec2::new(-5.0, 0.0), start, end);
        assert_eq!(closest, start);

        // Point after segment
        let closest = engine.closest_point_on_segment(Vec2::new(15.0, 0.0), start, end);
        assert_eq!(closest, end);
    }

    #[test]
    fn test_path_from_points() {
        let engine = GeometryEngine::default();
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
        ];

        let path = engine.path_from_points(&points, false);
        let elements: Vec<_> = path.iter().collect();
        assert_eq!(elements.len(), 3);

        let closed_path = engine.path_from_points(&points, true);
        let closed_elements: Vec<_> = closed_path.iter().collect();
        assert_eq!(closed_elements.len(), 4); // Includes ClosePath
    }
}
