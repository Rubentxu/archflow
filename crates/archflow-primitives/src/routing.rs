//! Connection Routing - Algoritmos de enrutamiento de conexiones
//!
//! Proporciona:
//! - Routing ortogonal con heurísticas L-shape y Z-shape
//! - Routing curvo con curvas de Bézier
//! - Routing spline con interpolación
//! - Routing inteligente que evita obstáculos
//! - Marcadores de flecha configurables

use crate::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipo de routing visual
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoutingType {
    /// Línea recta directa
    Straight,
    /// Líneas ortogonales (esquina)
    Orthogonal {
        /// Radio de la esquina
        corner_radius: f32,
        /// Estilo de esquina (Chamfer, Round, Miter)
        corner_style: CornerStyle,
    },
    /// Curvas de Bézier cuadráticas
    Curved {
        /// Factor de curvatura (0.0-1.0)
        curvature: f32,
        /// Posición de los puntos de control
        control_point_mode: ControlPointMode,
    },
    /// Spline Catmull-Rom o similar
    Spline {
        /// Tensión de la spline (0.0-1.0)
        tension: f32,
        /// Resolución de la discretización
        resolution: u32,
    },
    /// Routing inteligente que evita obstáculos
    Smart {
        /// Máxima profundidad de búsqueda
        max_depth: u32,
        /// Timeout en milisegundos
        timeout_ms: u32,
        /// Prioridad del algoritmo
        priority: RoutingPriority,
    },
}

impl Default for RoutingType {
    fn default() -> Self {
        RoutingType::Orthogonal {
            corner_radius: 10.0,
            corner_style: CornerStyle::Round,
        }
    }
}

impl RoutingType {
    /// Crear routing ortogonal con estilo específico
    pub fn orthogonal_with_style(corner_style: CornerStyle, radius: f32) -> Self {
        RoutingType::Orthogonal {
            corner_radius: radius,
            corner_style,
        }
    }

    /// Crear routing curvo
    pub fn curved(curvature: f32) -> Self {
        RoutingType::Curved {
            curvature: curvature.clamp(0.1, 0.9),
            control_point_mode: ControlPointMode::Auto,
        }
    }
}

/// Estilo de esquina para routing ortogonal
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CornerStyle {
    /// Esquina aguda (45 grados)
    Chamfer,
    /// Esquina redondeada
    Round,
    /// Esquina en ángulo recto
    Miter,
}

/// Modo de cálculo de puntos de control
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ControlPointMode {
    /// Calcular automáticamente basado en los puntos
    Auto,
    /// Usar punto medio
    Midpoint,
    /// Personalizado
    Custom(Vec2),
}

/// Prioridad del algoritmo de routing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RoutingPriority {
    /// Minimizar longitud total
    MinimizeLength,
    /// Minimizar número de esquinas
    MinimizeCorners,
    /// Evitar obstáculos primero
    AvoidObstacles,
    /// Balance entre longitud y esquinas
    Balanced,
}

/// Configuración del router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Padding alrededor de los obstáculos
    pub obstacle_padding: f32,
    /// Resolución para discretización de curvas
    pub curve_resolution: u32,
    /// Tolerancia para simplificación de path
    pub simplification_tolerance: f32,
    /// Ancho de paso para el algoritmo A*
    pub step_size: f32,
    /// Número máximo de iteraciones para el pathfinder
    pub max_iterations: u32,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            obstacle_padding: 5.0,
            curve_resolution: 20,
            simplification_tolerance: 1.0,
            step_size: 5.0,
            max_iterations: 1000,
        }
    }
}

/// Resultado del routing
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingResult {
    /// Puntos del path generado
    pub points: Vec<Vec2>,
    /// Longitud total del path
    pub length: f32,
    /// Número de esquinas
    pub corner_count: u32,
    /// Tiempo de cálculo en microsegundos
    pub compute_time_us: u128,
}

/// Obstáculo para el routing inteligente
#[derive(Debug, Clone, PartialEq)]
pub struct Obstacle {
    /// ID del obstáculo
    pub id: EntityId,
    /// Rectángulo delimitador
    pub bounds: crate::Rect,
    /// Prioridad (mayor = más difícil de atravesar)
    pub priority: u32,
}

impl Obstacle {
    pub fn new(id: EntityId, bounds: crate::Rect) -> Self {
        Self {
            id,
            bounds,
            priority: 1,
        }
    }

    /// Obtener rectángulo expandido con padding
    pub fn expanded(&self, padding: f32) -> crate::Rect {
        crate::Rect::from_min_max(
            self.bounds.min - Vec2::splat(padding),
            self.bounds.max + Vec2::splat(padding),
        )
    }
}

/// Motor de routing de conexiones
#[derive(Debug, Clone)]
pub struct ConnectionRouter {
    /// Configuración del router
    config: RouterConfig,
    /// Cache de paths calculados
    path_cache: HashMap<(u64, u64), RoutingResult>,
}

impl ConnectionRouter {
    /// Crear nuevo router
    pub fn new() -> Self {
        Self {
            config: RouterConfig::default(),
            path_cache: HashMap::new(),
        }
    }

    /// Crear con configuración personalizada
    #[inline]
    pub fn with_config(config: RouterConfig) -> Self {
        Self {
            config,
            path_cache: HashMap::new(),
        }
    }

    /// Obtener referencia a la configuración
    #[inline]
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// Obtener referencia mutable a la configuración
    #[inline]
    pub fn config_mut(&mut self) -> &mut RouterConfig {
        &mut self.config
    }

    /// Calcular path para una conexión
    pub fn route(
        &mut self,
        start: Vec2,
        end: Vec2,
        routing_type: &RoutingType,
        obstacles: &[Obstacle],
    ) -> RoutingResult {
        // Verificar cache
        let cache_key = (
            (start.x.to_bits() as u64) << 32 | (start.y.to_bits() as u64) & 0xFFFFFFFF,
            (end.x.to_bits() as u64) << 32 | (end.y.to_bits() as u64) & 0xFFFFFFFF,
        );
        if let Some(cached) = self.path_cache.get(&cache_key) {
            return cached.clone();
        }

        let start_time = std::time::Instant::now();
        let result = match routing_type {
            RoutingType::Straight => self.route_straight(start, end),
            RoutingType::Orthogonal {
                corner_radius,
                corner_style,
            } => self.route_orthogonal(start, end, *corner_radius, *corner_style),
            RoutingType::Curved {
                curvature,
                control_point_mode,
            } => self.route_curved(start, end, *curvature, *control_point_mode),
            RoutingType::Spline {
                tension,
                resolution,
            } => self.route_spline(start, end, *tension, *resolution),
            RoutingType::Smart {
                max_depth,
                timeout_ms: _,
                priority: _,
            } => self.route_smart(start, end, obstacles, *max_depth),
        };

        let compute_time = start_time.elapsed().as_micros();
        let mut result_with_time = result;
        result_with_time.compute_time_us = compute_time;

        // Cachear resultado
        self.path_cache.insert(cache_key, result_with_time.clone());

        result_with_time
    }

    /// Routing recto
    fn route_straight(&self, start: Vec2, end: Vec2) -> RoutingResult {
        let points = vec![start, end];
        let length = (end - start).length();

        RoutingResult {
            points,
            length,
            corner_count: 0,
            compute_time_us: 0,
        }
    }

    /// Routing ortogonal (L-shape, Z-shape)
    fn route_orthogonal(
        &self,
        start: Vec2,
        end: Vec2,
        corner_radius: f32,
        corner_style: CornerStyle,
    ) -> RoutingResult {
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        // Determinar dirección basada en la distancia
        let horizontal_first = dx.abs() > dy.abs();

        // Calcular puntos intermedios
        let (p1, p2, corner_point) = if horizontal_first {
            let mid_x = start.x + dx / 2.0;
            (
                Vec2::new(mid_x, start.y),
                Vec2::new(mid_x, end.y),
                if corner_radius > 0.0 {
                    Some(Self::calculate_corner(
                        Vec2::new(mid_x, start.y),
                        Vec2::new(mid_x, end.y),
                        corner_style,
                        corner_radius,
                    ))
                } else {
                    None
                },
            )
        } else {
            let mid_y = start.y + dy / 2.0;
            (
                Vec2::new(start.x, mid_y),
                Vec2::new(end.x, mid_y),
                if corner_radius > 0.0 {
                    Some(Self::calculate_corner(
                        Vec2::new(start.x, mid_y),
                        Vec2::new(end.x, mid_y),
                        corner_style,
                        corner_radius,
                    ))
                } else {
                    None
                },
            )
        };

        let mut points = vec![start];
        if let Some(cp) = corner_point {
            points.push(cp);
        }
        points.push(p1);
        points.push(p2);
        points.push(end);

        let length = Self::calculate_path_length(&points);

        RoutingResult {
            points,
            length,
            corner_count: if corner_radius > 0.0 { 1 } else { 2 },
            compute_time_us: 0,
        }
    }

    /// Calcular punto de esquina
    fn calculate_corner(prev: Vec2, next: Vec2, style: CornerStyle, radius: f32) -> Vec2 {
        match style {
            CornerStyle::Chamfer => {
                // Punto medio entre prev y next, movido hacia la esquina
                let corner = Vec2::new(prev.x, next.y);
                let dir = corner - prev;
                if dir.length() > radius {
                    prev + dir.normalize() * radius
                } else {
                    prev
                }
            }
            CornerStyle::Round => {
                // Esquina redondeada
                Vec2::new(prev.x, next.y)
            }
            CornerStyle::Miter => {
                // Esquina aguda
                Vec2::new(prev.x, next.y)
            }
        }
    }

    /// Routing curvo con Bézier
    fn route_curved(
        &self,
        start: Vec2,
        end: Vec2,
        curvature: f32,
        control_point_mode: ControlPointMode,
    ) -> RoutingResult {
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        let control1 = match control_point_mode {
            ControlPointMode::Auto => {
                let (cx, cy) = if dx.abs() > dy.abs() {
                    (start.x + dx * curvature, start.y)
                } else {
                    (start.x, start.y + dy * curvature)
                };
                Vec2::new(cx, cy)
            }
            ControlPointMode::Midpoint => start + (end - start) / 3.0,
            ControlPointMode::Custom(cp) => cp,
        };

        let control2 = match control_point_mode {
            ControlPointMode::Auto => {
                let (cx, cy) = if dx.abs() > dy.abs() {
                    (end.x - dx * curvature, end.y)
                } else {
                    (end.x, end.y - dy * curvature)
                };
                Vec2::new(cx, cy)
            }
            ControlPointMode::Midpoint => start + (end - start) * 2.0 / 3.0,
            ControlPointMode::Custom(cp) => cp,
        };

        // Discretizar la curva
        let points = Self::bezier_cubic(
            &[start, control1, control2, end],
            self.config.curve_resolution,
        );

        let length = Self::calculate_path_length(&points);

        RoutingResult {
            points,
            length,
            corner_count: 0, // Las curvas no tienen esquinas
            compute_time_us: 0,
        }
    }

    /// Routing spline
    fn route_spline(&self, start: Vec2, end: Vec2, tension: f32, resolution: u32) -> RoutingResult {
        // Catmull-Rom spline con 4 puntos de control
        let _mid = (start + end) / 2.0;
        let dir = end - start;
        let _perp = Vec2::new(-dir.y, dir.x).normalize() * dir.length() * tension;

        let p0 = start - dir * tension;
        let p1 = start;
        let p2 = end;
        let p3 = end + dir * tension;

        let points = Self::catmull_rom(&[p0, p1, p2, p3], resolution);

        let length = Self::calculate_path_length(&points);

        RoutingResult {
            points,
            length,
            corner_count: 0,
            compute_time_us: 0,
        }
    }

    /// Routing inteligente con evitación de obstáculos
    fn route_smart(
        &self,
        start: Vec2,
        end: Vec2,
        obstacles: &[Obstacle],
        max_depth: u32,
    ) -> RoutingResult {
        // Usar una variante simplificada de A* con waypoints
        let direct_path = self.route_straight(start, end);

        // Verificar si hay colisión directa
        let has_collision = obstacles.iter().any(|obs| {
            Self::line_intersects_rect(start, end, obs.expanded(self.config.obstacle_padding))
        });

        if !has_collision {
            return direct_path;
        }

        // Generar waypoints potenciales
        let waypoints = self.generate_waypoints(start, end, obstacles);

        // Buscar el mejor path usando los waypoints
        let best_path = self.find_best_path(start, end, &waypoints, obstacles, max_depth);

        if best_path.is_empty() {
            // Fallback al path directo
            direct_path
        } else {
            let mut points = vec![start];
            points.extend(best_path.clone());
            points.push(end);

            let length = Self::calculate_path_length(&points);
            let corner_count = (points.len() as u32).saturating_sub(2);

            RoutingResult {
                points,
                length,
                corner_count,
                compute_time_us: 0,
            }
        }
    }

    /// Generar waypoints potenciales alrededor de obstáculos
    fn generate_waypoints(&self, _start: Vec2, _end: Vec2, obstacles: &[Obstacle]) -> Vec<Vec2> {
        let mut waypoints = Vec::new();
        let padding = self.config.obstacle_padding;

        for obs in obstacles {
            let expanded = obs.expanded(padding);
            // Waypoints alrededor del obstáculo
            waypoints.push(Vec2::new(expanded.min.x - padding, expanded.center().y));
            waypoints.push(Vec2::new(expanded.max.x + padding, expanded.center().y));
            waypoints.push(Vec2::new(expanded.center().x, expanded.min.y - padding));
            waypoints.push(Vec2::new(expanded.center().x, expanded.max.y + padding));
        }

        waypoints
    }

    /// Encontrar el mejor path entre waypoints
    fn find_best_path(
        &self,
        start: Vec2,
        end: Vec2,
        _waypoints: &[Vec2],
        obstacles: &[Obstacle],
        _max_depth: u32,
    ) -> Vec<Vec2> {
        let mut best_path: Vec<Vec2> = Vec::new();
        let mut best_length = f32::MAX;

        // Simplificación: intentar path horizontal-vertical
        let path1 = vec![Vec2::new(start.x, end.y), end];
        let path2 = vec![Vec2::new(end.x, start.y), end];

        for path in [path1, path2] {
            let has_collision = path.windows(2).any(|segment| {
                obstacles.iter().any(|obs| {
                    Self::line_intersects_rect(
                        segment[0],
                        segment[1],
                        obs.expanded(self.config.obstacle_padding),
                    )
                })
            });

            if !has_collision {
                let length = Self::calculate_path_length(&path);
                if length < best_length {
                    best_length = length;
                    best_path = path[..path.len() - 1].to_vec();
                }
            }
        }

        best_path
    }

    /// Verificar si una línea intersecta un rectángulo
    fn line_intersects_rect(p1: Vec2, p2: Vec2, rect: crate::Rect) -> bool {
        // Verificar si la línea intersecta cualquiera de los lados del rectángulo
        let left = rect.min.x;
        let right = rect.max.x;
        let top = rect.min.y;
        let bottom = rect.max.y;

        // Si ambos puntos están a un lado del rectángulo, no hay intersección
        if (p1.x < left && p2.x < left) || (p1.x > right && p2.x > right) {
            return false;
        }
        if (p1.y < top && p2.y < top) || (p1.y > bottom && p2.y > bottom) {
            return false;
        }

        // Si algún punto está dentro del rectángulo, hay intersección
        if rect.contains(p1) || rect.contains(p2) {
            return true;
        }

        // Verificar intersección con los lados del rectángulo
        Self::line_intersects_line(p1, p2, Vec2::new(left, top), Vec2::new(right, top))
            || Self::line_intersects_line(p1, p2, Vec2::new(left, bottom), Vec2::new(right, bottom))
            || Self::line_intersects_line(p1, p2, Vec2::new(left, top), Vec2::new(left, bottom))
            || Self::line_intersects_line(p1, p2, Vec2::new(right, top), Vec2::new(right, bottom))
    }

    /// Verificar intersección entre dos líneas
    fn line_intersects_line(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
        let d1 = a2 - a1;
        let d2 = b2 - b1;
        let d = d1.x * d2.y - d1.y * d2.x;

        if d.abs() < f32::EPSILON {
            return false; // Líneas paralelas
        }

        let t = ((b1.x - a1.x) * d2.y - (b1.y - a1.y) * d2.x) / d;
        let u = ((b1.x - a1.x) * d1.y - (b1.y - a1.y) * d1.x) / d;

        (t >= 0.0 && t <= 1.0) && (u >= 0.0 && u <= 1.0)
    }

    /// Discretizar curva de Bézier cúbica
    fn bezier_cubic(points: &[Vec2], resolution: u32) -> Vec<Vec2> {
        if points.len() < 4 {
            return points.to_vec();
        }

        let p0 = points[0];
        let p1 = points[1];
        let p2 = points[2];
        let p3 = points[3];

        let mut result = Vec::with_capacity(resolution as usize + 1);
        result.push(p0);

        for i in 1..=resolution {
            let t = i as f32 / resolution as f32;
            let mt = 1.0 - t;
            let mt2 = mt * mt;
            let mt3 = mt2 * mt;
            let t2 = t * t;
            let t3 = t2 * t;

            let x = mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x;
            let y = mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y;

            result.push(Vec2::new(x, y));
        }

        result
    }

    /// Catmull-Rom spline interpolation
    fn catmull_rom(points: &[Vec2], resolution: u32) -> Vec<Vec2> {
        if points.len() < 4 {
            return points.to_vec();
        }

        let mut result = Vec::new();

        for i in 0..points.len() - 3 {
            let p0 = points[i];
            let p1 = points[i + 1];
            let p2 = points[i + 2];
            let p3 = points[i + 3];

            for j in 0..resolution {
                let t = j as f32 / resolution as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5
                    * ((2.0 * p1.x)
                        + (-p0.x + p2.x) * t
                        + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                        + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

                let y = 0.5
                    * ((2.0 * p1.y)
                        + (-p0.y + p2.y) * t
                        + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                        + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

                result.push(Vec2::new(x, y));
            }
        }

        result
    }

    /// Calcular longitud de un path
    fn calculate_path_length(points: &[Vec2]) -> f32 {
        if points.len() < 2 {
            return 0.0;
        }

        points.windows(2).map(|w| (w[1] - w[0]).length()).sum()
    }

    /// Limpiar cache
    #[inline]
    pub fn clear_cache(&mut self) {
        self.path_cache.clear();
    }

    /// Obtener tamaño del cache
    #[inline]
    pub fn cache_size(&self) -> usize {
        self.path_cache.len()
    }
}

impl Default for ConnectionRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Marcadores de flecha para conexiones
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarkerType {
    /// Sin marcador
    None,
    /// Flecha simple
    Arrow {
        /// Tamaño del marcador
        size: f32,
        /// Ángulo de la punta (en grados)
        angle: f32,
        /// Relleno o outline
        fill: bool,
    },
    /// Círculo
    Circle {
        /// Radio del círculo
        radius: f32,
        /// Relleno o outline
        fill: bool,
    },
    /// Diamante
    Diamond {
        /// Tamaño del marcador
        size: f32,
        /// Relleno o outline
        fill: bool,
    },
    /// Cuadrado
    Square {
        /// Tamaño del marcador
        size: f32,
        /// Relleno o outline
        fill: bool,
    },
    /// Marcador personalizado
    Custom {
        /// Path SVG del marcador
        path_data: String,
        /// Ancho del viewBox
        viewbox_width: f32,
        /// Alto del viewBox
        viewbox_height: f32,
    },
}

impl Default for MarkerType {
    fn default() -> Self {
        MarkerType::Arrow {
            size: 12.0,
            angle: 30.0,
            fill: true,
        }
    }
}

impl MarkerType {
    /// Crear flecha simple
    pub fn arrow(size: f32) -> Self {
        MarkerType::Arrow {
            size,
            angle: 30.0,
            fill: true,
        }
    }

    /// Crear círculo
    pub fn circle(radius: f32) -> Self {
        MarkerType::Circle {
            radius,
            fill: false,
        }
    }

    /// Crear diamante
    pub fn diamond(size: f32) -> Self {
        MarkerType::Diamond { size, fill: true }
    }

    /// Calcular puntos del marcador para rendering
    pub fn to_path_points(&self, direction: Vec2, position: Vec2) -> Vec<Vec2> {
        let forward = direction.normalize();
        let right = Vec2::new(-forward.y, forward.x);

        match self {
            MarkerType::None => vec![],
            MarkerType::Arrow {
                size,
                angle,
                fill: _,
            } => {
                let angle_rad = angle.to_radians();
                let s = *size;
                let p1 = position + forward * s;
                let p2 = position + right * s * angle_rad.sin() - forward * s * angle_rad.cos();
                let p3 = position - right * s * angle_rad.sin() - forward * s * angle_rad.cos();
                vec![p2, p1, p3]
            }
            MarkerType::Circle { radius, fill: _ } => {
                // Generar círculo como polígono
                let segments = 12;
                let r = *radius;
                (0..segments)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / segments as f32;
                        position + Vec2::new(a.cos() * r, a.sin() * r)
                    })
                    .collect()
            }
            MarkerType::Diamond { size, fill: _ } => {
                let s = *size;
                vec![
                    position + forward * s,
                    position + right * s,
                    position - forward * s,
                    position - right * s,
                ]
            }
            MarkerType::Square { size, fill: _ } => {
                let half = *size / 2.0;
                vec![
                    position + Vec2::new(-half, -half),
                    position + Vec2::new(half, -half),
                    position + Vec2::new(half, half),
                    position + Vec2::new(-half, half),
                ]
            }
            MarkerType::Custom { .. } => vec![], // Los personalizados se manejan vía SVG
        }
    }

    /// Obtener color del marcador
    pub fn get_color(&self, connection_color: &str) -> String {
        match self {
            MarkerType::Arrow { fill: true, .. } => connection_color.to_string(),
            MarkerType::Circle { fill: true, .. } => connection_color.to_string(),
            MarkerType::Diamond { fill: true, .. } => connection_color.to_string(),
            MarkerType::Square { fill: true, .. } => connection_color.to_string(),
            _ => "transparent".to_string(),
        }
    }
}

/// Configuración de marcadores
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerConfig {
    /// Marcador al inicio de la conexión
    pub start_marker: MarkerType,
    /// Marcador al final de la conexión
    pub end_marker: MarkerType,
    /// Escala global de marcadores
    pub scale: f32,
    /// Color de override (None usa el de la conexión)
    pub override_color: Option<String>,
}

impl Default for MarkerConfig {
    fn default() -> Self {
        Self {
            start_marker: MarkerType::None,
            end_marker: MarkerType::default(),
            scale: 1.0,
            override_color: None,
        }
    }
}

impl MarkerConfig {
    /// Crear configuración con flecha al final
    pub fn with_arrow_at_end(size: f32) -> Self {
        Self {
            start_marker: MarkerType::None,
            end_marker: MarkerType::arrow(size),
            scale: 1.0,
            override_color: None,
        }
    }

    /// Crear configuración con flechas en ambos extremos
    pub fn with_arrows(size: f32) -> Self {
        Self {
            start_marker: MarkerType::arrow(size),
            end_marker: MarkerType::arrow(size),
            scale: 1.0,
            override_color: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_new() {
        let router = ConnectionRouter::new();
        assert_eq!(router.config().obstacle_padding, 5.0);
    }

    #[test]
    fn test_route_straight() {
        let mut router = ConnectionRouter::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 100.0);

        let result = router.route(start, end, &RoutingType::Straight, &[]);

        assert_eq!(result.points.len(), 2);
        assert_eq!(result.points[0], start);
        assert_eq!(result.points[1], end);
        assert!((result.length - 141.421).abs() < 0.1);
    }

    #[test]
    fn test_route_orthogonal() {
        let mut router = ConnectionRouter::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 100.0);

        let result = router.route(
            start,
            end,
            &RoutingType::Orthogonal {
                corner_radius: 0.0,
                corner_style: CornerStyle::Miter,
            },
            &[],
        );

        assert_eq!(result.points.len(), 4);
        assert_eq!(result.points[0], start);
        assert_eq!(result.points[3], end);
    }

    #[test]
    fn test_route_curved() {
        let mut router = ConnectionRouter::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 0.0);

        let result = router.route(
            start,
            end,
            &RoutingType::Curved {
                curvature: 0.5,
                control_point_mode: ControlPointMode::Auto,
            },
            &[],
        );

        assert!(result.points.len() > 2);
        assert_eq!(result.points[0], start);
        assert_eq!(result.points.last().unwrap(), &end);
    }

    #[test]
    fn test_route_spline() {
        let mut router = ConnectionRouter::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 0.0);

        let result = router.route(
            start,
            end,
            &RoutingType::Spline {
                tension: 0.5,
                resolution: 10,
            },
            &[],
        );

        assert!(result.points.len() > 2);
    }

    #[test]
    fn test_marker_type_arrow() {
        let marker = MarkerType::arrow(10.0);
        let direction = Vec2::new(1.0, 0.0);
        let position = Vec2::new(0.0, 0.0);

        let points = marker.to_path_points(direction, position);

        assert_eq!(points.len(), 3);
        assert!(points[0].y != 0.0 || points[2].y != 0.0); // Puntas a los lados
    }

    #[test]
    fn test_marker_type_diamond() {
        let marker = MarkerType::diamond(10.0);
        let direction = Vec2::new(1.0, 0.0);
        let position = Vec2::new(0.0, 0.0);

        let points = marker.to_path_points(direction, position);

        assert_eq!(points.len(), 4);
    }

    #[test]
    fn test_marker_config_default() {
        let config = MarkerConfig::default();
        assert!(matches!(config.end_marker, MarkerType::Arrow { .. }));
    }

    #[test]
    fn test_path_cache() {
        let mut router = ConnectionRouter::new();
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(100.0, 100.0);

        let _ = router.route(start, end, &RoutingType::Straight, &[]);
        assert_eq!(router.cache_size(), 1);

        // La segunda llamada debería usar cache
        let _ = router.route(start, end, &RoutingType::Straight, &[]);
        assert_eq!(router.cache_size(), 1);
    }

    #[test]
    fn test_router_config() {
        let config = RouterConfig {
            obstacle_padding: 10.0,
            curve_resolution: 30,
            simplification_tolerance: 2.0,
            step_size: 3.0,
            max_iterations: 500,
        };

        let router = ConnectionRouter::with_config(config);
        assert_eq!(router.config().obstacle_padding, 10.0);
        assert_eq!(router.config().curve_resolution, 30);
    }

    #[test]
    fn test_bezier_cubic_discretization() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(33.3, 0.0),
            Vec2::new(66.6, 100.0),
            Vec2::new(100.0, 100.0),
        ];

        let result = ConnectionRouter::bezier_cubic(&points, 10);

        assert_eq!(result.len(), 11);
        assert_eq!(result[0], Vec2::new(0.0, 0.0));
        assert_eq!(result[10], Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_catmull_rom() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 50.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(150.0, 100.0),
        ];

        let result = ConnectionRouter::catmull_rom(&points, 5);

        assert!(result.len() > 4);
    }

    #[test]
    fn test_line_intersects_line() {
        // Líneas que se intersectan
        assert!(ConnectionRouter::line_intersects_line(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(10.0, 0.0)
        ));

        // Líneas que no se intersectan
        assert!(!ConnectionRouter::line_intersects_line(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(15.0, 15.0)
        ));
    }

    #[test]
    fn test_routing_type_variants() {
        let straight = RoutingType::Straight;
        let orthogonal = RoutingType::orthogonal_with_style(CornerStyle::Round, 15.0);
        let curved = RoutingType::curved(0.3);

        assert!(matches!(straight, RoutingType::Straight));
        assert!(matches!(orthogonal, RoutingType::Orthogonal { .. }));
        assert!(matches!(curved, RoutingType::Curved { .. }));
    }
}
