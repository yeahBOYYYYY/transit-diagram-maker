//! Grid geometry: cells, 8-way directions, path routing and SVG path building.
//!
//! All document geometry lives in *cell units*. A cell `(x, y)` covers the square
//! `[x, x+1) × [y, y+1)` and its center is `(x + 0.5, y + 0.5)`. Lines travel
//! through cell centers, either axis-aligned or along 45° diagonals.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub const fn new(x: i32, y: i32) -> Self {
        Cell { x, y }
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x as f64 + 0.5, self.y as f64 + 0.5)
    }

    /// The cell containing a point given in cell units.
    pub fn containing(p: Vec2) -> Self {
        Cell::new(p.x.floor() as i32, p.y.floor() as i32)
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Cell::new(self.x + dx, self.y + dy)
    }

    pub fn step(self, dir: Dir8) -> Self {
        let (dx, dy) = dir.delta();
        self.offset(dx, dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[allow(clippy::should_implement_trait)]
impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }
    pub fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
    pub fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
    pub fn scale(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
    pub fn len(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub fn dot(self, o: Vec2) -> f64 {
        self.x * o.x + self.y * o.y
    }
    pub fn cross(self, o: Vec2) -> f64 {
        self.x * o.y - self.y * o.x
    }
    pub fn normalized(self) -> Vec2 {
        let l = self.len();
        if l == 0.0 {
            self
        } else {
            self.scale(1.0 / l)
        }
    }
    /// Left-hand normal in screen coordinates (y grows downward).
    pub fn perp(self) -> Vec2 {
        Vec2::new(self.y, -self.x)
    }
    /// Round both coordinates to the document precision of 0.1 cell.
    pub fn snapped(self) -> Vec2 {
        Vec2::new(snap_tenth(self.x), snap_tenth(self.y))
    }
    /// Rotate by `deg` degrees around `pivot` (clockwise on screen for positive angles).
    pub fn rotate_around(self, pivot: Vec2, deg: f64) -> Vec2 {
        let (s, c) = deg.to_radians().sin_cos();
        let d = self.sub(pivot);
        let r = Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c);
        // Kill floating noise so 90° rotations land exactly on the grid.
        let clean = |v: f64| {
            let r = (v * 1e9).round() / 1e9;
            if r == 0.0 {
                0.0
            } else {
                r
            }
        };
        let p = pivot.add(r);
        Vec2::new(clean(p.x), clean(p.y))
    }
}

/// Positions are stored with a precision of one decimal (0.1 cell).
pub fn snap_tenth(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    let r = (v * 10.0).round() / 10.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

/// One of the eight grid directions, numbered clockwise starting from east.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dir8(pub u8);

impl Dir8 {
    pub const E: Dir8 = Dir8(0);
    pub const SE: Dir8 = Dir8(1);
    pub const S: Dir8 = Dir8(2);
    pub const SW: Dir8 = Dir8(3);
    pub const W: Dir8 = Dir8(4);
    pub const NW: Dir8 = Dir8(5);
    pub const N: Dir8 = Dir8(6);
    pub const NE: Dir8 = Dir8(7);

    const DELTAS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

    pub fn delta(self) -> (i32, i32) {
        Self::DELTAS[(self.0 % 8) as usize]
    }

    /// Direction from a delta whose components are each in -1..=1 (not both 0).
    pub fn from_delta(dx: i32, dy: i32) -> Option<Dir8> {
        let d = (dx.signum(), dy.signum());
        Self::DELTAS.iter().position(|&x| x == d).map(|i| Dir8(i as u8))
    }

    /// Direction between two cells if they are aligned on one of the 8 directions.
    pub fn between(a: Cell, b: Cell) -> Option<Dir8> {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        if (dx, dy) == (0, 0) {
            return None;
        }
        if dx == 0 || dy == 0 || dx.abs() == dy.abs() {
            Dir8::from_delta(dx, dy)
        } else {
            None
        }
    }

    pub fn is_diagonal(self) -> bool {
        self.0 % 2 == 1
    }

    pub fn rotated(self, steps45: i32) -> Dir8 {
        Dir8(((self.0 as i32 + steps45).rem_euclid(8)) as u8)
    }

    pub fn unit(self) -> Vec2 {
        let (dx, dy) = self.delta();
        Vec2::new(dx as f64, dy as f64).normalized()
    }

    pub fn opposite(self) -> Dir8 {
        self.rotated(4)
    }
}

/// How the pen routes between two cells that are not on a common 8-way direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteMode {
    /// Travel straight first, then finish with a 45° diagonal.
    #[default]
    StraightFirst,
    /// Travel diagonally first, then finish straight.
    DiagonalFirst,
}

/// Route from `a` to `b` using at most one bend, returning the intermediate vertex if needed.
pub fn route(a: Cell, b: Cell, mode: RouteMode) -> Vec<Cell> {
    if a == b {
        return vec![a];
    }
    if Dir8::between(a, b).is_some() {
        return vec![a, b];
    }
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let diag = dx.abs().min(dy.abs());
    let (sx, sy) = (dx.signum(), dy.signum());
    let bend = match mode {
        RouteMode::DiagonalFirst => a.offset(sx * diag, sy * diag),
        RouteMode::StraightFirst => b.offset(-sx * diag, -sy * diag),
    };
    vec![a, bend, b]
}

/// Turn an arbitrary list of vertices into a valid line polyline: every pair of
/// consecutive vertices is 8-way aligned, there are no duplicates and no
/// redundant collinear vertices.
pub fn normalize_path(points: &[Cell], mode: RouteMode) -> Vec<Cell> {
    let mut routed: Vec<Cell> = Vec::with_capacity(points.len());
    for &p in points {
        match routed.last().copied() {
            None => routed.push(p),
            Some(last) if last == p => {}
            Some(last) => routed.extend(route(last, p, mode).into_iter().skip(1)),
        }
    }
    compress(&expand(&routed))
}

/// Expand a polyline into the full sequence of cells it passes through.
/// Consecutive vertices must be 8-way aligned (unaligned pairs are routed).
pub fn expand(points: &[Cell]) -> Vec<Cell> {
    let mut out = Vec::new();
    if let Some(&first) = points.first() {
        out.push(first);
    }
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let pts = if Dir8::between(a, b).is_some() || a == b {
            vec![a, b]
        } else {
            route(a, b, RouteMode::StraightFirst)
        };
        for seg in pts.windows(2) {
            let (s, e) = (seg[0], seg[1]);
            if let Some(d) = Dir8::between(s, e) {
                let mut c = s;
                while c != e {
                    c = c.step(d);
                    out.push(c);
                }
            }
        }
    }
    out.dedup();
    out
}

/// Compress a cell sequence (unit steps) into its turning vertices.
pub fn compress(cells: &[Cell]) -> Vec<Cell> {
    let mut out: Vec<Cell> = Vec::new();
    for &c in cells {
        if out.last() == Some(&c) {
            continue;
        }
        if out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            if let (Some(d1), Some(d2)) = (Dir8::between(a, b), Dir8::between(b, c)) {
                if d1 == d2 {
                    *out.last_mut().unwrap() = c;
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// Direction of the line at the given index of an expanded cell sequence.
pub fn direction_at(cells: &[Cell], index: usize) -> Option<Dir8> {
    if cells.len() < 2 {
        return None;
    }
    if index + 1 < cells.len() {
        Dir8::between(cells[index], cells[index + 1])
    } else {
        Dir8::between(cells[index - 1], cells[index])
    }
}

/// Every cell on the straight 8-connected (Bresenham style) run between two cells,
/// used to interpolate fast mouse drags so no cell is skipped.
pub fn cells_between(a: Cell, b: Cell) -> Vec<Cell> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let n = dx.abs().max(dy.abs());
    if n == 0 {
        return vec![a];
    }
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            Cell::new(
                (a.x as f64 + dx as f64 * t).round() as i32,
                (a.y as f64 + dy as f64 * t).round() as i32,
            )
        })
        .collect()
}

/// Axis-aligned rectangle (in whatever unit the caller uses).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn from_points(pts: impl IntoIterator<Item = Vec2>) -> Option<Rect> {
        let mut it = pts.into_iter();
        let first = it.next()?;
        let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
        for p in it {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        Some(Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 })
    }

    pub fn union(self, o: Rect) -> Rect {
        let x0 = self.x.min(o.x);
        let y0 = self.y.min(o.y);
        let x1 = (self.x + self.w).max(o.x + o.w);
        let y1 = (self.y + self.h).max(o.y + o.h);
        Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
    }

    pub fn inflate(self, m: f64) -> Rect {
        Rect { x: self.x - m, y: self.y - m, w: self.w + 2.0 * m, h: self.h + 2.0 * m }
    }

    pub fn scale(self, s: f64) -> Rect {
        Rect { x: self.x * s, y: self.y * s, w: self.w * s, h: self.h * s }
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn intersects(self, o: Rect) -> bool {
        self.x <= o.x + o.w && o.x <= self.x + self.w && self.y <= o.y + o.h && o.y <= self.y + self.h
    }
}

/// Format a number compactly for SVG output (max 3 decimals, no trailing zeros).
pub fn fmt_num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        let s = format!("{:.3}", r);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Build an SVG path through `pts` whose corners are rounded with `radius`
/// (all values already in output units). The radius shrinks automatically when
/// segments are too short to fit the requested curve.
pub fn rounded_polyline(pts: &[Vec2], radius: f64) -> String {
    let mut d = String::new();
    if pts.is_empty() {
        return d;
    }
    let _ = write!(d, "M{} {}", fmt_num(pts[0].x), fmt_num(pts[0].y));
    let n = pts.len();
    for i in 1..n {
        let p = pts[i];
        if i == n - 1 || radius <= 0.0 {
            let _ = write!(d, "L{} {}", fmt_num(p.x), fmt_num(p.y));
            continue;
        }
        let prev = pts[i - 1];
        let next = pts[i + 1];
        let vin = p.sub(prev);
        let vout = next.sub(p);
        let (lin, lout) = (vin.len(), vout.len());
        let (u, v) = (vin.normalized(), vout.normalized());
        let cos = u.dot(v).clamp(-1.0, 1.0);
        let theta = cos.acos(); // turning angle
        if theta < 1e-6 || (std::f64::consts::PI - theta) < 1e-6 || lin == 0.0 || lout == 0.0 {
            let _ = write!(d, "L{} {}", fmt_num(p.x), fmt_num(p.y));
            continue;
        }
        let t = (theta / 2.0).tan();
        // Segments shared with another corner may only give away half their length.
        let max_in = if i == 1 { lin } else { lin / 2.0 };
        let max_out = if i + 1 == n - 1 { lout } else { lout / 2.0 };
        let dist = (radius * t).min(max_in).min(max_out);
        let r = dist / t;
        let a = p.sub(u.scale(dist));
        let b = p.add(v.scale(dist));
        let sweep = if u.cross(v) > 0.0 { 1 } else { 0 };
        let _ = write!(
            d,
            "L{} {}A{} {} 0 0 {} {} {}",
            fmt_num(a.x),
            fmt_num(a.y),
            fmt_num(r),
            fmt_num(r),
            sweep,
            fmt_num(b.x),
            fmt_num(b.y)
        );
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: i32, y: i32) -> Cell {
        Cell::new(x, y)
    }

    #[test]
    fn dir_between_aligned_and_unaligned() {
        assert_eq!(Dir8::between(c(0, 0), c(5, 0)), Some(Dir8::E));
        assert_eq!(Dir8::between(c(0, 0), c(0, -3)), Some(Dir8::N));
        assert_eq!(Dir8::between(c(0, 0), c(2, 2)), Some(Dir8::SE));
        assert_eq!(Dir8::between(c(0, 0), c(-4, 4)), Some(Dir8::SW));
        assert_eq!(Dir8::between(c(0, 0), c(2, 1)), None);
        assert_eq!(Dir8::between(c(1, 1), c(1, 1)), None);
    }

    #[test]
    fn dir_rotation_wraps() {
        assert_eq!(Dir8::E.rotated(2), Dir8::S);
        assert_eq!(Dir8::E.rotated(-1), Dir8::NE);
        assert_eq!(Dir8::N.rotated(10), Dir8::E);
        assert_eq!(Dir8::SE.opposite(), Dir8::NW);
        assert!(Dir8::NE.is_diagonal());
        assert!(!Dir8::W.is_diagonal());
    }

    #[test]
    fn route_straight_first_and_diagonal_first() {
        assert_eq!(route(c(0, 0), c(5, 2), RouteMode::StraightFirst), vec![c(0, 0), c(3, 0), c(5, 2)]);
        assert_eq!(route(c(0, 0), c(5, 2), RouteMode::DiagonalFirst), vec![c(0, 0), c(2, 2), c(5, 2)]);
        assert_eq!(route(c(0, 0), c(-1, -4), RouteMode::StraightFirst), vec![c(0, 0), c(0, -3), c(-1, -4)]);
        assert_eq!(route(c(0, 0), c(3, 3), RouteMode::StraightFirst), vec![c(0, 0), c(3, 3)]);
    }

    #[test]
    fn expand_and_compress_roundtrip() {
        let pts = vec![c(0, 0), c(3, 0), c(5, 2), c(5, 4)];
        let cells = expand(&pts);
        assert_eq!(cells.len(), 1 + 3 + 2 + 2);
        assert_eq!(cells[3], c(3, 0));
        assert_eq!(cells[4], c(4, 1));
        assert_eq!(compress(&cells), pts);
    }

    #[test]
    fn normalize_removes_duplicates_and_collinear_points() {
        let pts = vec![c(0, 0), c(0, 0), c(2, 0), c(4, 0), c(6, 3)];
        assert_eq!(normalize_path(&pts, RouteMode::StraightFirst), vec![c(0, 0), c(4, 0), c(4, 1), c(6, 3)]);
    }

    #[test]
    fn direction_at_end_uses_incoming() {
        let cells = expand(&[c(0, 0), c(2, 0), c(2, 2)]);
        assert_eq!(direction_at(&cells, 0), Some(Dir8::E));
        assert_eq!(direction_at(&cells, 2), Some(Dir8::S));
        assert_eq!(direction_at(&cells, cells.len() - 1), Some(Dir8::S));
    }

    #[test]
    fn cells_between_has_no_gaps() {
        let run = cells_between(c(0, 0), c(5, 2));
        assert_eq!(run.first(), Some(&c(0, 0)));
        assert_eq!(run.last(), Some(&c(5, 2)));
        for w in run.windows(2) {
            assert!((w[1].x - w[0].x).abs() <= 1 && (w[1].y - w[0].y).abs() <= 1);
        }
    }

    #[test]
    fn rotate_quarter_turn_is_exact() {
        let p = Vec2::new(3.5, 0.5).rotate_around(Vec2::new(0.5, 0.5), 90.0);
        assert_eq!(p, Vec2::new(0.5, 3.5));
    }

    #[test]
    fn positions_snap_to_tenths() {
        assert_eq!(snap_tenth(1.26), 1.3);
        assert_eq!(snap_tenth(-0.04), 0.0);
        assert_eq!(Vec2::new(0.123, 4.449).snapped(), Vec2::new(0.1, 4.4));
    }

    #[test]
    fn rounded_polyline_contains_arc_for_corner() {
        let pts = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)];
        let d = rounded_polyline(&pts, 4.0);
        assert!(d.starts_with("M0 0L6 0A4 4 0 0 1 10 4L10 10"), "{d}");
    }

    #[test]
    fn rounded_polyline_clamps_radius_on_short_segments() {
        let pts = [Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(2.0, 2.0), Vec2::new(4.0, 2.0)];
        let d = rounded_polyline(&pts, 100.0);
        // Interior segment of length 2 is shared, so each corner can use at most 1.
        assert!(d.contains("A1 1 0 0 1 2 1"), "{d}");
        assert!(d.contains("A1 1 0 0 0 3 2"), "{d}");
    }

    #[test]
    fn rounded_polyline_without_radius_is_plain() {
        let pts = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)];
        assert_eq!(rounded_polyline(&pts, 0.0), "M0 0L1 0L1 1");
    }

    #[test]
    fn fmt_num_is_compact() {
        assert_eq!(fmt_num(1.0), "1");
        assert_eq!(fmt_num(1.25), "1.25");
        assert_eq!(fmt_num(-0.33333), "-0.333");
        assert_eq!(fmt_num(2.0004), "2");
    }

    #[test]
    fn rect_union_and_intersection() {
        let a = Rect { x: 0.0, y: 0.0, w: 2.0, h: 2.0 };
        let b = Rect { x: 1.0, y: 1.0, w: 3.0, h: 1.0 };
        let u = a.union(b);
        assert_eq!(u, Rect { x: 0.0, y: 0.0, w: 4.0, h: 2.0 });
        assert!(a.intersects(b));
        assert!(!a.intersects(Rect { x: 5.0, y: 5.0, w: 1.0, h: 1.0 }));
    }
}
