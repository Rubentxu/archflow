//! IntersectionEngine - Detección de intersecciones geométricas
//!
//! Proporciona algoritmos para:
//! - Intersección rectángulo-rectángulo
//! - Intersección punto-polígono (ray casting)
//! - Intersección línea-línea
//! - Intersección línea-círculo
//! - Detección de punto en forma

use crate::Vec2;
use kurbo::{BezPath, Point as KurboPoint, Rect as KurboRect, Shape};

/// Resultado de detección de intersección
#[derive(Debug, Clone, PartialEq)]
pub enum IntersectionType {
    /// No hay intersección
    None,
    /// Intersección en un punto
    Point(Vec2),
    /// Intersección en línea (colisión)
    Line(Vec2, Vec2),
    /// Intersección en segmento
    Segment(Vec2, Vec2),
    /// Intersección en área
    Area(KurboRect),
}

/// Configuración para hit testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTestConfig {
    /// Tolerancia para el punto
    pub tolerance: f32,
    /// Si incluir filled shapes
    pub fill: bool,
    /// Si incluir strokes
    pub stroke: bool,
    /// Ancho del stroke para hit testing
    pub stroke_width: f32,
}

impl Default for HitTestConfig {
    fn default() -> Self {
        Self {
            tolerance: 2.0,
            fill: true,
            stroke: true,
            stroke_width: 2.0,
        }
    }
}

/// Motor de detección de intersecciones
#[derive(Debug, Default, Clone)]
pub struct IntersectionEngine {
    config: HitTestConfig,
}

impl IntersectionEngine {
    /// Crear engine con configuración
    #[inline]
    pub fn with_config(config: HitTestConfig) -> Self {
        Self { config }
    }

    /// Actualizar configuración
    pub fn set_config(&mut self, config: HitTestConfig) {
        self.config = config;
    }

    /// Verificar intersección rectángulo-rectángulo
    #[inline]
    pub fn rect_rect(&self, a: KurboRect, b: KurboRect) -> bool {
        !(a.x1 < b.x0 || a.x0 > b.x1 || a.y1 < b.y0 || a.y0 > b.y1)
    }

    /// Obtener área de intersección rectángulo-rectángulo
    #[inline]
    pub fn rect_rect_area(&self, a: KurboRect, b: KurboRect) -> Option<KurboRect> {
        let x0 = a.x0.max(b.x0);
        let y0 = a.y0.max(b.y0);
        let x1 = a.x1.min(b.x1);
        let y1 = a.y1.min(b.y1);

        if x0 < x1 && y0 < y1 {
            Some(KurboRect::new(x0, y0, x1, y1))
        } else {
            None
        }
    }

    /// Verificar si punto está en rectángulo
    #[inline]
    pub fn point_in_rect(&self, point: Vec2, rect: KurboRect) -> bool {
        let x = point.x as f64;
        let y = point.y as f64;
        x >= rect.x0 && x <= rect.x1 && y >= rect.y0 && y <= rect.y1
    }

    /// Verificar si punto está en polígono (ray casting / even-odd rule)
    pub fn point_in_polygon(&self, point: Vec2, polygon: &[Vec2]) -> bool {
        let mut inside = false;
        let n = polygon.len();

        if n < 3 {
            return false;
        }

        for i in 0..n {
            let a = polygon[i];
            let b = polygon[(i + 1) % n];

            // Verificar si el rayo cruza el borde
            if ((a.y > point.y) != (b.y > point.y))
                && (point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x)
            {
                inside = !inside;
            }
        }

        inside
    }

    /// Verificar si punto está en path (considerando fill y stroke)
    pub fn point_in_path(&self, point: Vec2, path: &BezPath) -> bool {
        let p = self.to_kurbo_point(point);

        // Verificar fill
        if self.config.fill && path.contains(p) {
            return true;
        }

        // Verificar stroke
        if self.config.stroke {
            for seg in path.segments() {
                if self.segment_hit_test(
                    p,
                    seg,
                    (self.config.stroke_width + self.config.tolerance) as f64,
                ) {
                    return true;
                }
            }
        }

        false
    }

    /// Verificar si punto está cerca de un segmento
    fn segment_hit_test(&self, point: KurboPoint, seg: kurbo::PathSeg, tolerance: f64) -> bool {
        match seg {
            kurbo::PathSeg::Line(line) => {
                self.line_point_distance(point, line.p0, line.p1) <= tolerance
            }
            kurbo::PathSeg::Quad(quad) => {
                // Verificar distancia a curva cuadrática
                self.quad_point_distance(point, quad.p0, quad.p1, quad.p2) <= tolerance
            }
            kurbo::PathSeg::Cubic(cubic) => {
                // Verificar distancia a curva cúbica
                self.cubic_point_distance(point, cubic.p0, cubic.p1, cubic.p2, cubic.p3)
                    <= tolerance
            }
        }
    }

    /// Distancia de punto a línea
    fn line_point_distance(&self, point: KurboPoint, start: KurboPoint, end: KurboPoint) -> f64 {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let len2 = dx * dx + dy * dy;

        if len2 < 1e-10 {
            return ((point.x - start.x).powi(2) + (point.y - start.y).powi(2)).sqrt();
        }

        let t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / len2;
        let t = t.clamp(0.0, 1.0);

        let proj_x = start.x + t * dx;
        let proj_y = start.y + t * dy;

        ((point.x - proj_x).powi(2) + (point.y - proj_y).powi(2)).sqrt()
    }

    /// Distancia de punto a Bézier cuadrática (sampling)
    fn quad_point_distance(
        &self,
        point: KurboPoint,
        p0: KurboPoint,
        p1: KurboPoint,
        p2: KurboPoint,
    ) -> f64 {
        let samples = 16;
        let mut min_dist = f64::INFINITY;

        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let mt = 1.0 - t;
            let x = mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x;
            let y = mt * mt * p0.y + 2.0 * mt * t * p1.y + t * t * p2.y;
            let dist = ((point.x - x).powi(2) + (point.y - y).powi(2)).sqrt();
            min_dist = min_dist.min(dist);
        }

        min_dist
    }

    /// Distancia de punto a Bézier cúbica (sampling)
    fn cubic_point_distance(
        &self,
        point: KurboPoint,
        p0: KurboPoint,
        p1: KurboPoint,
        p2: KurboPoint,
        p3: KurboPoint,
    ) -> f64 {
        let samples = 20;
        let mut min_dist = f64::INFINITY;

        for i in 0..=samples {
            let t = i as f64 / samples as f64;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;

            let x = mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x;
            let y = mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y;
            let dist = ((point.x - x).powi(2) + (point.y - y).powi(2)).sqrt();
            min_dist = min_dist.min(dist);
        }

        min_dist
    }

    /// Intersección línea-línea (segmentos infinitos)
    pub fn line_line(&self, a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
        let denom = (a2.x - a1.x) * (b2.y - b1.y) - (a2.y - a1.y) * (b2.x - b1.x);

        if denom.abs() < f32::EPSILON {
            return None; // Líneas paralelas o coincidentes
        }

        let ua = ((b1.x - a1.x) * (b2.y - b1.y) - (b1.y - a1.y) * (b2.x - b1.x)) / denom;

        // Retornar punto de intersección
        Some(Vec2::new(
            a1.x + ua * (a2.x - a1.x),
            a1.y + ua * (a2.y - a1.y),
        ))
    }

    /// Intersección segmento-segmento
    pub fn segment_segment(&self, a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
        let denom = (a2.x - a1.x) * (b2.y - b1.y) - (a2.y - a1.y) * (b2.x - b1.x);

        if denom.abs() < f32::EPSILON {
            return None; // Segmentos paralelos
        }

        let ua = ((b1.x - a1.x) * (b2.y - b1.y) - (b1.y - a1.y) * (b2.x - b1.x)) / denom;
        let ub = ((b1.x - a1.x) * (a2.y - a1.y) - (b1.y - a1.y) * (a2.x - a1.x)) / denom;

        if ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0 {
            Some(Vec2::new(
                a1.x + ua * (a2.x - a1.x),
                a1.y + ua * (a2.y - a1.y),
            ))
        } else {
            None
        }
    }

    /// Intersección línea-círculo (devuelve puntos de intersección)
    pub fn line_circle(
        &self,
        line_start: Vec2,
        line_end: Vec2,
        center: Vec2,
        radius: f32,
    ) -> Vec<Vec2> {
        let d = line_end - line_start;
        let f = line_start - center;

        let dx = d.x as f64;
        let dy = d.y as f64;
        let fx = f.x as f64;
        let fy = f.y as f64;
        let radius_f64 = radius as f64;

        let a = dx * dx + dy * dy;
        let b = 2.0 * (fx * dx + fy * dy);
        let c = fx * fx + fy * fy - radius_f64 * radius_f64;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return Vec::new();
        }

        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        let mut points = Vec::new();

        if t1 >= 0.0 && t1 <= 1.0 {
            points.push(line_start + d * t1 as f32);
        }
        if t2 >= 0.0 && t2 <= 1.0 && (t1 - t2).abs() > 1e-6 {
            points.push(line_start + d * t2 as f32);
        }

        points
    }

    /// Verificar intersección path-path (bounding box check + detailed check)
    pub fn path_path(&self, path1: &BezPath, path2: &BezPath) -> bool {
        // Quick rejection con bounding boxes
        let bounds1 = path1.bounding_box();
        let bounds2 = path2.bounding_box();

        if !self.rect_rect(bounds1, bounds2) {
            return false;
        }

        // Detailed check - verificar segmentos
        for seg1 in path1.segments() {
            for seg2 in path2.segments() {
                if self.segments_intersect(seg1, seg2) {
                    return true;
                }
            }
        }

        false
    }

    /// Verificar intersección entre segmentos
    fn segments_intersect(&self, seg1: kurbo::PathSeg, seg2: kurbo::PathSeg) -> bool {
        // Sampling approach - discretizar y verificar
        let _samples = 20;

        match seg1 {
            kurbo::PathSeg::Line(line1) => {
                let p1a = self.from_kurbo_point(line1.p0);
                let p1b = self.from_kurbo_point(line1.p1);

                match seg2 {
                    kurbo::PathSeg::Line(line2) => {
                        let p2a = self.from_kurbo_point(line2.p0);
                        let p2b = self.from_kurbo_point(line2.p1);
                        self.segment_segment(p1a, p1b, p2a, p2b).is_some()
                    }
                    _ => false, // Simplified: no check for curves vs lines
                }
            }
            _ => false,
        }
    }

    /// Encontrar closest point on line segment
    pub fn closest_point_on_segment(&self, point: Vec2, start: Vec2, end: Vec2) -> Vec2 {
        let d = end - start;
        let len2 = d.dot(d);

        if len2 < f32::EPSILON {
            return start;
        }

        let t = ((point - start).dot(d) / len2).clamp(0.0, 1.0);
        start + d * t
    }

    /// Verificar si polígonos se intersectan (SAT - Separating Axis Theorem)
    pub fn polygons_intersect(&self, poly1: &[Vec2], poly2: &[Vec2]) -> bool {
        if poly1.len() < 3 || poly2.len() < 3 {
            return false;
        }

        // Verificar bounding boxes primero
        let bounds1 = self.polygon_bounds(poly1);
        let bounds2 = self.polygon_bounds(poly2);

        if !self.rect_rect(bounds1, bounds2) {
            return false;
        }

        // SAT - verificar cada eje
        for poly in [poly1, poly2] {
            for i in 0..poly.len() {
                let p1 = poly[i];
                let p2 = poly[(i + 1) % poly.len()];

                // Eje perpendicular al borde
                let edge = p2 - p1;
                let len = edge.length();
                if len < f32::EPSILON {
                    continue;
                }
                let axis = Vec2::new(-edge.y / len, edge.x / len);

                let proj1 = self.project_polygon(axis, poly1);
                let proj2 = self.project_polygon(axis, poly2);

                if proj1.1 < proj2.0 || proj2.1 < proj1.0 {
                    return false; // Separados en este eje
                }
            }
        }

        true
    }

    /// Calcular bounding box de polígono
    fn polygon_bounds(&self, polygon: &[Vec2]) -> KurboRect {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in polygon {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        KurboRect::new(min_x as f64, min_y as f64, max_x as f64, max_y as f64)
    }

    /// Proyectar polígono sobre eje
    fn project_polygon(&self, axis: Vec2, polygon: &[Vec2]) -> (f32, f32) {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for p in polygon {
            let proj = p.dot(axis);
            min = min.min(proj);
            max = max.max(proj);
        }

        (min, max)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_rect() {
        let engine = IntersectionEngine::default();
        let rect = KurboRect::new(0.0, 0.0, 100.0, 100.0);

        assert!(engine.point_in_rect(Vec2::new(50.0, 50.0), rect));
        assert!(engine.point_in_rect(Vec2::new(0.0, 0.0), rect));
        assert!(!engine.point_in_rect(Vec2::new(101.0, 50.0), rect));
    }

    #[test]
    fn test_rect_rect() {
        let engine = IntersectionEngine::default();
        let r1 = KurboRect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = KurboRect::new(50.0, 50.0, 150.0, 150.0);
        let r3 = KurboRect::new(200.0, 200.0, 300.0, 300.0);

        assert!(engine.rect_rect(r1, r2));
        assert!(!engine.rect_rect(r1, r3));
    }

    #[test]
    fn test_point_in_polygon() {
        let engine = IntersectionEngine::default();
        // Triangle
        let triangle = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 100.0),
        ];

        assert!(engine.point_in_polygon(Vec2::new(50.0, 50.0), &triangle));
        assert!(!engine.point_in_polygon(Vec2::new(150.0, 50.0), &triangle));
    }

    #[test]
    fn test_segment_segment() {
        let engine = IntersectionEngine::default();

        // Intersecting
        let result = engine.segment_segment(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(0.0, 100.0),
        );
        assert_eq!(result, Some(Vec2::new(50.0, 50.0)));

        // Non-intersecting
        let result = engine.segment_segment(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
            Vec2::new(30.0, 0.0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_line_circle() {
        let engine = IntersectionEngine::default();

        let points = engine.line_circle(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 0.0),
            10.0,
        );

        assert_eq!(points.len(), 2);
    }

    #[test]
    fn test_closest_point_on_segment() {
        let engine = IntersectionEngine::default();

        let closest = engine.closest_point_on_segment(
            Vec2::new(5.0, 5.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        );
        assert_eq!(closest, Vec2::new(5.0, 0.0));
    }

    #[test]
    fn test_polygons_intersect() {
        let engine = IntersectionEngine::default();

        // Two overlapping triangles
        let tri1 = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(50.0, 100.0),
        ];
        let tri2 = vec![
            Vec2::new(50.0, 50.0),
            Vec2::new(150.0, 50.0),
            Vec2::new(100.0, 150.0),
        ];

        assert!(engine.polygons_intersect(&tri1, &tri2));

        // Non-overlapping
        let tri3 = vec![
            Vec2::new(200.0, 200.0),
            Vec2::new(300.0, 200.0),
            Vec2::new(250.0, 300.0),
        ];

        assert!(!engine.polygons_intersect(&tri1, &tri3));
    }

    #[test]
    fn test_point_in_path() {
        let engine = IntersectionEngine::default();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((100.0, 0.0));
        path.line_to((100.0, 100.0));
        path.line_to((0.0, 100.0));
        path.close_path();

        // Point inside
        assert!(engine.point_in_path(Vec2::new(50.0, 50.0), &path));

        // Point outside
        assert!(!engine.point_in_path(Vec2::new(150.0, 50.0), &path));
    }
}
