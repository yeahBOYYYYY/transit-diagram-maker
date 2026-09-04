use serde::{Deserialize, Serialize};

/// 2D vector with f64 precision for high-zoom fidelity and infinite grid calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DVec2 {
    pub x: f64,
    pub y: f64,
}

impl DVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    pub fn distance_squared(self, other: Self) -> f64 {
        (self - other).length_squared()
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 1e-12 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn lerp(self, other: Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl std::ops::Add for DVec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for DVec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for DVec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f64> for DVec2 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Neg for DVec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// Axis-Aligned Bounding Box (AABB) in f64 space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: DVec2,
    pub max: DVec2,
}

impl Aabb {
    pub fn new(min: DVec2, max: DVec2) -> Self {
        Self {
            min: DVec2::new(min.x.min(max.x), min.y.min(max.y)),
            max: DVec2::new(min.x.max(max.x), min.y.max(max.y)),
        }
    }

    pub fn from_point(p: DVec2) -> Self {
        Self { min: p, max: p }
    }

    pub fn from_points<I: IntoIterator<Item = DVec2>>(points: I) -> Option<Self> {
        let mut iter = points.into_iter();
        let first = iter.next()?;
        let mut aabb = Self::from_point(first);
        for p in iter {
            aabb = aabb.expand(p);
        }
        Some(aabb)
    }

    pub fn expand(&self, p: DVec2) -> Self {
        Self {
            min: DVec2::new(self.min.x.min(p.x), self.min.y.min(p.y)),
            max: DVec2::new(self.max.x.max(p.x), self.max.y.max(p.y)),
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: DVec2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: DVec2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    pub fn expand_by_margin(&self, margin: f64) -> Self {
        Self {
            min: DVec2::new(self.min.x - margin, self.min.y - margin),
            max: DVec2::new(self.max.x + margin, self.max.y + margin),
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn contains_point(&self, p: DVec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn center(&self) -> DVec2 {
        (self.min + self.max) * 0.5
    }

    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
}

/// Clamps coordinate to scene bounds [-100_000.0, 100_000.0] to prevent catastrophic cancellation.
pub const MAX_BOUND_EXTENT: f64 = 100_000.0;

pub fn clamp_to_world_bounds(coord: DVec2) -> DVec2 {
    DVec2::new(
        coord.x.clamp(-MAX_BOUND_EXTENT, MAX_BOUND_EXTENT),
        coord.y.clamp(-MAX_BOUND_EXTENT, MAX_BOUND_EXTENT),
    )
}

/// Snap coordinate to integer grid subdivisions.
pub fn snap_to_grid(coord: DVec2, grid_spacing: f64) -> DVec2 {
    if grid_spacing <= 0.0 {
        return coord;
    }
    DVec2::new(
        (coord.x / grid_spacing).round() * grid_spacing,
        (coord.y / grid_spacing).round() * grid_spacing,
    )
}

/// Snaps the ray from `from` towards `to` to the nearest 45-degree octolinear transit angle
/// (0, 45, 90, 135, 180, 225, 270, 315 degrees).
pub fn snap_octolinear_angle(from: DVec2, to: DVec2) -> DVec2 {
    let delta = to - from;
    if delta.length_squared() < 1e-8 {
        return to;
    }

    let angle = delta.y.atan2(delta.x);
    let step = std::f64::consts::FRAC_PI_4; // 45 degrees in radians
    let snapped_angle = (angle / step).round() * step;

    // Preserve the dominant distance along the chosen direction
    let dir = DVec2::new(snapped_angle.cos(), snapped_angle.sin());
    let projected_dist = delta.dot(dir);

    from + dir * projected_dist.max(0.0)
}

/// Strictly locks a transit ray onto integer grid units along orthogonal (0°, 90°)
/// or diagonal (45°, 135°) channels.
/// Guarantees that horizontal lines lie strictly on a single grid line (y = y0),
/// vertical lines lie strictly on a single grid line (x = x0),
/// and diagonal lines connect grid points with equal dx and dy.
pub fn snap_transit_octolinear_to_grid(from: DVec2, target: DVec2, grid_spacing: f64) -> DVec2 {
    let from_grid = snap_to_grid(from, grid_spacing);
    let delta = target - from_grid;

    if delta.length_squared() < (grid_spacing * 0.25).powi(2) {
        return from_grid;
    }

    let angle = delta.y.atan2(delta.x);
    let step = std::f64::consts::FRAC_PI_4;
    // Octant 0..8
    let octant = ((angle / step).round() as i64).rem_euclid(8);

    let grid_x = (delta.x.abs() / grid_spacing).round() * grid_spacing;
    let grid_y = (delta.y.abs() / grid_spacing).round() * grid_spacing;

    match octant {
        0 => DVec2::new(from_grid.x + grid_x.max(grid_spacing), from_grid.y),
        1 => {
            let d = grid_x.max(grid_y).max(grid_spacing);
            DVec2::new(from_grid.x + d, from_grid.y + d)
        }
        2 => DVec2::new(from_grid.x, from_grid.y + grid_y.max(grid_spacing)),
        3 => {
            let d = grid_x.max(grid_y).max(grid_spacing);
            DVec2::new(from_grid.x - d, from_grid.y + d)
        }
        4 => DVec2::new(from_grid.x - grid_x.max(grid_spacing), from_grid.y),
        5 => {
            let d = grid_x.max(grid_y).max(grid_spacing);
            DVec2::new(from_grid.x - d, from_grid.y - d)
        }
        6 => DVec2::new(from_grid.x, from_grid.y - grid_y.max(grid_spacing)),
        7 => {
            let d = grid_x.max(grid_y).max(grid_spacing);
            DVec2::new(from_grid.x + d, from_grid.y - d)
        }
        _ => from_grid,
    }
}

/// Projects a point `p` onto the line segment `[a, b]`.
/// Returns `(projected_point, parameter_t, distance)`.
pub fn project_point_to_segment(p: DVec2, a: DVec2, b: DVec2) -> (DVec2, f64, f64) {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq < 1e-12 {
        return (a, 0.0, p.distance(a));
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (proj, t, p.distance(proj))
}
