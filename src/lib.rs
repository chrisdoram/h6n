//! Hexagonal grid coordinates and geometry.
//!
//! Hexagons are addressed in the [cube coordinate] system `(q, r, s)`, where
//! the constraint `q + r + s = 0` always holds. The building blocks:
//!
//! - [`Point`]: a location on the grid.
//! - [`Vector`]: a displacement between locations.
//! - [`Hex`]: a hexagon at a [`Point`], with its orientation ([`Flat`] or
//!   [`Pointy`]) encoded at the type level.
//!
//! ```
//! use h6n::{Direction, Hex, Pointy};
//!
//! let hex: Hex<Pointy> = Hex::new(2, -1);
//!
//! // Step to an adjacent hex and measure the distance back.
//! let neighbor = hex.neighbor(Direction::QS);
//! assert_eq!(hex.distance(neighbor), 1);
//!
//! // Convert to pixel space and back.
//! let (x, y) = hex.center(32.0);
//! assert_eq!(Hex::from_pixel(x, y, 32.0), hex);
//! ```
//!
//! All methods return new values and never modify in place; the only
//! mutation in the API is the explicit `+=`/`-=` assignment operators.
//!
//! Coordinates are `i32` and arithmetic inherits the primitive overflow
//! semantics: panic in debug builds, wrapping in release builds.
//!
//! [cube coordinate]: https://www.redblobgames.com/grids/hexagons/#coordinates-cube

#![warn(missing_docs)]

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// √3 as an f64 literal, since `f64::sqrt` is not a `const fn`.
const SQRT_3: f64 = 1.732_050_807_568_877_2;
/// √3 / 2
const SQRT_3_2: f64 = SQRT_3 / 2.0;

/// A location on the hexagonal grid, in cube coordinates `(q, r, s)`.
/// The constraint `q + r + s = 0` always holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    q: i32,
    r: i32,
    s: i32,
}

/// A displacement between two locations on the hexagonal grid, in cube
/// coordinates `(q, r, s)`. The constraint `q + r + s = 0` always holds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vector {
    q: i32,
    r: i32,
    s: i32,
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Flat {}
    impl Sealed for super::Pointy {}
}

/// The orientation of a hexagon: [`Flat`] topped or [`Pointy`] topped.
///
/// This trait is sealed and cannot be implemented outside this crate.
pub trait Orientation: sealed::Sealed {
    /// Forward hex-to-pixel matrix `[f0, f1, f2, f3]` for a hex of size 1,
    /// mapping `(q, r)` to a pixel `(x, y)`.
    const FORWARD: [f64; 4];
    /// Inverse pixel-to-hex matrix `[b0, b1, b2, b3]` for a hex of size 1,
    /// mapping a pixel `(x, y)` to a fractional `(q, r)`.
    const INVERSE: [f64; 4];
    /// The angle of the first corner, in multiples of 60 degrees.
    const START_ANGLE: f64;
    /// Aligns neighbor directions with corners: the edge shared with the
    /// neighbor in [`Vector::DIRECTIONS`]`[d]` runs from corner
    /// `(d + EDGE_CORNER_OFFSET) % 6` to the corner after it. Exposed
    /// through [`Hex::edge_corner_indices`].
    const EDGE_CORNER_OFFSET: usize;
}

/// Marker type for flat-top hexagons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Flat;

impl Orientation for Flat {
    const FORWARD: [f64; 4] = [1.5, 0.0, SQRT_3_2, SQRT_3];
    const INVERSE: [f64; 4] = [2.0 / 3.0, 0.0, -1.0 / 3.0, SQRT_3 / 3.0];
    const START_ANGLE: f64 = 0.0;
    const EDGE_CORNER_OFFSET: usize = 0;
}

/// Marker type for pointy-top hexagons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pointy;

impl Orientation for Pointy {
    const FORWARD: [f64; 4] = [SQRT_3, SQRT_3_2, 0.0, 1.5];
    const INVERSE: [f64; 4] = [SQRT_3 / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0];
    const START_ANGLE: f64 = 0.5;
    const EDGE_CORNER_OFFSET: usize = 5;
}

/// A hexagon defined by its canonical coordinate on the grid.
///
/// The orientation `O` ([`Flat`] or [`Pointy`]) is encoded at the type level
/// and only affects pixel-space geometry; grid coordinates and arithmetic are
/// orientation-independent.
pub struct Hex<O> {
    coordinate: Point,
    // `fn() -> O` keeps `Hex` `Copy`/`Send`/`Sync` regardless of `O`.
    _phantom: PhantomData<fn() -> O>,
}

impl<O> Clone for Hex<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O> Copy for Hex<O> {}

impl<O> fmt::Debug for Hex<O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Hex")
            .field("coordinate", &self.coordinate)
            .finish()
    }
}

impl<O> PartialEq for Hex<O> {
    fn eq(&self, other: &Self) -> bool {
        self.coordinate == other.coordinate
    }
}

impl<O> Eq for Hex<O> {}

impl<O> Hash for Hex<O> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.coordinate.hash(state);
    }
}

/// Error returned when a cube coordinate violates the constraint `q + r + s = 0`.
///
/// Carries the offending coordinate so callers can report what was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCoordinate {
    /// The `q` element of the rejected coordinate.
    pub q: i32,
    /// The `r` element of the rejected coordinate.
    pub r: i32,
    /// The `s` element of the rejected coordinate.
    pub s: i32,
}

impl fmt::Display for InvalidCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid cube coordinate (q: {}, r: {}, s: {}): q + r + s = 0 must hold",
            self.q, self.r, self.s
        )
    }
}

impl std::error::Error for InvalidCoordinate {}

/// The conversion from a tuple containing all three elements of a coordinate
/// is fallible as the constraint `q + r + s = 0` must hold.
impl TryFrom<(i32, i32, i32)> for Point {
    type Error = InvalidCoordinate;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        if value.0 + value.1 + value.2 != 0 {
            Err(InvalidCoordinate {
                q: value.0,
                r: value.1,
                s: value.2,
            })
        } else {
            Ok(Self {
                q: value.0,
                r: value.1,
                s: value.2,
            })
        }
    }
}

impl From<(i32, i32)> for Point {
    fn from(value: (i32, i32)) -> Self {
        Self::new(value.0, value.1)
    }
}

/// The conversion from a tuple containing all three elements of a coordinate
/// is fallible as the constraint `q + r + s = 0` must hold.
impl TryFrom<(i32, i32, i32)> for Vector {
    type Error = InvalidCoordinate;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        if value.0 + value.1 + value.2 != 0 {
            Err(InvalidCoordinate {
                q: value.0,
                r: value.1,
                s: value.2,
            })
        } else {
            Ok(Self {
                q: value.0,
                r: value.1,
                s: value.2,
            })
        }
    }
}

impl From<(i32, i32)> for Vector {
    fn from(value: (i32, i32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        Vector {
            q: value.q,
            r: value.r,
            s: value.s,
        }
    }
}

/// The conversion from a tuple containing all three elements of a coordinate
/// is fallible as the constraint `q + r + s = 0` must hold.
impl<O> TryFrom<(i32, i32, i32)> for Hex<O> {
    type Error = InvalidCoordinate;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        Ok(Point::try_from(value)?.into())
    }
}

impl<O> From<(i32, i32)> for Hex<O> {
    fn from(value: (i32, i32)) -> Self {
        Point::from(value).into()
    }
}

impl<O> From<Point> for Hex<O> {
    fn from(value: Point) -> Self {
        Self {
            coordinate: value,
            _phantom: PhantomData,
        }
    }
}

impl<O> From<Hex<O>> for Point {
    fn from(value: Hex<O>) -> Self {
        value.coordinate
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.q, self.r, self.s)
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.q, self.r, self.s)
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::Output {
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
        }
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::Output {
            q: self.q - other.q,
            r: self.r - other.r,
            s: self.s - other.s,
        }
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            q: -self.q,
            r: -self.r,
            s: -self.s,
        }
    }
}

/// Scaling the vector by a factor; equivalent to [`Vector::scale`].
impl Mul<i32> for Vector {
    type Output = Self;

    fn mul(self, factor: i32) -> Self::Output {
        self.scale(factor)
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

/// Can subtract two points in space to return a vector representing the displacement
/// between the points.
impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Self::Output {
            q: self.q - other.q,
            r: self.r - other.r,
            s: self.s - other.s,
        }
    }
}

/// Can add a vector displacement to a point, returning a new point.
impl Add<Vector> for Point {
    type Output = Self;

    fn add(self, other: Vector) -> Self::Output {
        Self::Output {
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
        }
    }
}

/// Can subtract a vector displacement from a point, returning a new point.
impl Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, other: Vector) -> Self::Output {
        self + -other
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, other: Vector) {
        *self = *self + other;
    }
}

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, other: Vector) {
        *self = *self - other;
    }
}

/// Can add a vector displacement to a hex, returning the hex at the
/// displaced coordinate.
impl<O> Add<Vector> for Hex<O> {
    type Output = Self;

    fn add(self, other: Vector) -> Self::Output {
        (self.coordinate + other).into()
    }
}

/// Can subtract a vector displacement from a hex, returning the hex at the
/// displaced coordinate.
impl<O> Sub<Vector> for Hex<O> {
    type Output = Self;

    fn sub(self, other: Vector) -> Self::Output {
        (self.coordinate - other).into()
    }
}

/// Can subtract two hexes to return a vector representing the displacement
/// between them.
impl<O> Sub for Hex<O> {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        self.coordinate - other.coordinate
    }
}

impl<O> AddAssign<Vector> for Hex<O> {
    fn add_assign(&mut self, other: Vector) {
        *self = *self + other;
    }
}

impl<O> SubAssign<Vector> for Hex<O> {
    fn sub_assign(&mut self, other: Vector) {
        *self = *self - other;
    }
}

impl<O> Hex<O> {
    /// Creates the hex at axial coordinate `(q, r)`, deriving `s = -q - r`.
    pub fn new(q: i32, r: i32) -> Self {
        Point::new(q, r).into()
    }

    /// The canonical coordinate of this hex.
    pub fn coordinate(&self) -> Point {
        self.coordinate
    }

    /// The six adjacent hexes, in the order of [`Direction::ALL`].
    pub fn neighbors(&self) -> [Hex<O>; 6] {
        Vector::DIRECTIONS.map(|d| *self + d)
    }

    /// The adjacent hex in `direction`.
    pub fn neighbor(self, direction: Direction) -> Self {
        self + direction.vector()
    }

    /// The number of hexes between `self` and `other`.
    pub fn distance(self, other: Self) -> i32 {
        self.coordinate.distance(other.coordinate)
    }

    /// All hexes within `radius` rings of `self` — a filled hexagonal disc,
    /// including `self`. Yields nothing for a negative `radius`.
    ///
    /// ```
    /// use h6n::{Hex, Pointy};
    ///
    /// let center: Hex<Pointy> = Hex::new(0, 0);
    /// assert_eq!(center.range(2).count(), 19);
    /// ```
    pub fn range(self, radius: i32) -> impl Iterator<Item = Self> {
        (-radius..=radius).flat_map(move |q| {
            let lo = (-radius).max(-q - radius);
            let hi = radius.min(-q + radius);
            (lo..=hi).map(move |r| self + Vector::new(q, r))
        })
    }

    /// The hexes at exactly `radius` rings from `self` — a hollow ring,
    /// walked once around: starting from the corner toward
    /// [`Direction::SR`] and stepping `radius` hexes per side in
    /// [`Direction::ALL`] order. A `radius` of 0 yields just `self`;
    /// negative yields nothing.
    ///
    /// ```
    /// use h6n::{Hex, Pointy};
    ///
    /// let center: Hex<Pointy> = Hex::new(0, 0);
    /// assert_eq!(center.ring(2).count(), 12);
    /// ```
    pub fn ring(self, radius: i32) -> impl Iterator<Item = Self> {
        let center = (radius == 0).then_some(self);
        let walk = (0..6).flat_map(move |side| {
            let corner = self + Vector::DIRECTIONS[(side + 4) % 6].scale(radius);
            (0..radius).map(move |step| corner + Vector::DIRECTIONS[side].scale(step))
        });
        center.into_iter().chain(walk)
    }

    /// Reflects the hex across the q-axis, swapping `r` and `s`.
    #[must_use]
    pub fn reflect_q(self) -> Self {
        Point {
            q: self.coordinate.q,
            r: self.coordinate.s,
            s: self.coordinate.r,
        }
        .into()
    }

    /// Reflects the hex across the r-axis, swapping `q` and `s`.
    #[must_use]
    pub fn reflect_r(self) -> Self {
        Point {
            q: self.coordinate.s,
            r: self.coordinate.r,
            s: self.coordinate.q,
        }
        .into()
    }

    /// Reflects the hex across the s-axis, swapping `q` and `r`.
    #[must_use]
    pub fn reflect_s(self) -> Self {
        Point {
            q: self.coordinate.r,
            r: self.coordinate.q,
            s: self.coordinate.s,
        }
        .into()
    }
}

impl<O: Orientation> Hex<O> {
    /// The pixel coordinate of the hex's center for the given `size`,
    /// taking the orientation into account.
    pub fn center(&self, size: f64) -> (f64, f64) {
        let [f0, f1, f2, f3] = O::FORWARD;
        let q = f64::from(self.coordinate.q);
        let r = f64::from(self.coordinate.r);
        (size * (f0 * q + f1 * r), size * (f2 * q + f3 * r))
    }

    /// The six pixel corners of the hex for the given `size`, in order.
    pub fn corners(&self, size: f64) -> [(f64, f64); 6] {
        let (cx, cy) = self.center(size);
        std::array::from_fn(|i| {
            let angle = std::f64::consts::FRAC_PI_3 * (i as f64 + O::START_ANGLE);
            (cx + size * angle.cos(), cy + size * angle.sin())
        })
    }

    /// The indices into [`Hex::corners`] of the two corners bounding the
    /// edge shared with the neighbor in `direction`. The edge runs from
    /// the first returned corner to the second.
    ///
    /// Useful for drawing region boundaries: walk a cell's directions, and
    /// for each neighbor outside the region, stroke this edge.
    pub const fn edge_corner_indices(direction: Direction) -> (usize, usize) {
        let first = (direction as usize + O::EDGE_CORNER_OFFSET) % 6;
        (first, (first + 1) % 6)
    }

    /// The hex containing the pixel coordinate `(x, y)` for the given `size`.
    /// This is the inverse of [`Hex::center`].
    pub fn from_pixel(x: f64, y: f64, size: f64) -> Self {
        let [b0, b1, b2, b3] = O::INVERSE;
        let q = (b0 * x + b1 * y) / size;
        let r = (b2 * x + b3 * y) / size;
        let s = -q - r;

        // Round each cube coordinate, then recompute the one that moved
        // furthest from the others so the constraint q + r + s = 0 holds.
        let (mut rq, mut rr) = (q.round(), r.round());
        let rs = s.round();
        let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
        if dq > dr && dq > ds {
            rq = -rr - rs;
        } else if dr > ds {
            rr = -rq - rs;
        }
        Self::new(rq as i32, rr as i32)
    }
}

impl Hex<Flat> {
    /// The pixel width of a flat-top hexagon of the given `size`.
    pub const fn width(size: f64) -> f64 {
        2.0 * size
    }

    /// The pixel height of a flat-top hexagon of the given `size`.
    pub const fn height(size: f64) -> f64 {
        SQRT_3 * size
    }
}

impl Hex<Pointy> {
    /// The pixel width of a pointy-top hexagon of the given `size`.
    pub const fn width(size: f64) -> f64 {
        SQRT_3 * size
    }

    /// The pixel height of a pointy-top hexagon of the given `size`.
    pub const fn height(size: f64) -> f64 {
        2.0 * size
    }
}

/// The six neighbor directions on the hexagonal grid, in the order of
/// [`Vector::DIRECTIONS`].
///
/// Each name lists the cube axis that increases, then the axis that
/// decreases: `QS` is the displacement `(1, 0, -1)`. The screen headings
/// in the variant docs assume pixel `y` increasing downward.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// `(1, 0, -1)` — [`Pointy`]: east; [`Flat`]: south-east.
    QS,
    /// `(0, 1, -1)` — [`Pointy`]: south-east; [`Flat`]: south.
    RS,
    /// `(-1, 1, 0)` — [`Pointy`]: south-west; [`Flat`]: south-west.
    RQ,
    /// `(-1, 0, 1)` — [`Pointy`]: west; [`Flat`]: north-west.
    SQ,
    /// `(0, -1, 1)` — [`Pointy`]: north-west; [`Flat`]: north.
    SR,
    /// `(1, -1, 0)` — [`Pointy`]: north-east; [`Flat`]: north-east.
    QR,
}

impl Direction {
    /// All six directions, in [`Vector::DIRECTIONS`] order; the same order
    /// as [`Hex::neighbors`].
    pub const ALL: [Self; 6] = [
        Self::QS,
        Self::RS,
        Self::RQ,
        Self::SQ,
        Self::SR,
        Self::QR,
    ];

    /// The unit displacement vector of this direction.
    pub const fn vector(self) -> Vector {
        Vector::DIRECTIONS[self as usize]
    }

    /// The direction rotated 60 degrees clockwise (with pixel `y`
    /// increasing downward); matches [`Vector::rotate_clockwise`].
    #[must_use]
    pub const fn rotate_clockwise(self) -> Self {
        Self::ALL[(self as usize + 1) % 6]
    }

    /// The direction rotated 60 degrees counterclockwise (with pixel `y`
    /// increasing downward); matches [`Vector::rotate_counterclockwise`].
    #[must_use]
    pub const fn rotate_counterclockwise(self) -> Self {
        Self::ALL[(self as usize + 5) % 6]
    }

    /// The opposite direction.
    #[must_use]
    pub const fn opposite(self) -> Self {
        Self::ALL[(self as usize + 3) % 6]
    }
}

/// The unit displacement vector of the direction; equivalent to
/// [`Direction::vector`].
impl From<Direction> for Vector {
    fn from(value: Direction) -> Self {
        value.vector()
    }
}

impl Vector {
    /// The six displacement vectors between a hex and its neighbors, in the
    /// order of [`Direction::ALL`].
    pub const DIRECTIONS: [Self; 6] = [
        Self { q: 1, r: 0, s: -1 },
        Self { q: 0, r: 1, s: -1 },
        Self { q: -1, r: 1, s: 0 },
        Self { q: -1, r: 0, s: 1 },
        Self { q: 0, r: -1, s: 1 },
        Self { q: 1, r: -1, s: 0 },
    ];

    /// Creates the vector with axial elements `(q, r)`, deriving `s = -q - r`.
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r, s: -q - r }
    }

    /// The `q` element of the vector.
    pub fn q(&self) -> i32 {
        self.q
    }

    /// The `r` element of the vector.
    pub fn r(&self) -> i32 {
        self.r
    }

    /// The `s` element of the vector.
    pub fn s(&self) -> i32 {
        self.s
    }

    /// Scales the vector by a given magnitude.
    #[must_use]
    pub fn scale(self, factor: i32) -> Self {
        Self::new(self.q * factor, self.r * factor)
    }

    /// Rotates the vector 60 degrees counterclockwise.
    #[must_use]
    pub fn rotate_counterclockwise(self) -> Self {
        Self::new(self.q + self.r, -self.q)
    }

    /// Rotates the vector 60 degrees clockwise.
    #[must_use]
    pub fn rotate_clockwise(self) -> Self {
        Self::new(-self.r, self.q + self.r)
    }

    /// The length of the vector, in hexes.
    pub fn magnitude(self) -> i32 {
        (self.q.abs() + self.r.abs() + self.s.abs()) / 2
    }

    /// The distance between the tips of `self` and `other`, in hexes.
    pub fn distance(self, other: Self) -> i32 {
        (self - other).magnitude()
    }
}

impl Point {
    /// Creates the point at axial coordinate `(q, r)`, deriving `s = -q - r`.
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r, s: -q - r }
    }

    /// The `q` element of the coordinate.
    pub fn q(&self) -> i32 {
        self.q
    }

    /// The `r` element of the coordinate.
    pub fn r(&self) -> i32 {
        self.r
    }

    /// The `s` element of the coordinate.
    pub fn s(&self) -> i32 {
        self.s
    }

    /// The number of hexes between `self` and `other`.
    pub fn distance(self, other: Self) -> i32 {
        (self - other).magnitude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_rejects_invalid_coordinates() {
        let err = Point::try_from((1, 2, 3)).unwrap_err();
        assert_eq!(err, InvalidCoordinate { q: 1, r: 2, s: 3 });
        assert!(Vector::try_from((1, 2, 3)).is_err());
        assert!(Hex::<Pointy>::try_from((1, 2, -3)).is_ok());
    }

    #[test]
    fn distance_counts_hexes() {
        let a = Point::new(-2, 0);
        let b = Point::new(2, -1);
        assert_eq!(a.distance(b), 4);
        assert_eq!(b.distance(a), 4);
        assert_eq!(a.distance(a), 0);
    }

    #[test]
    fn assign_operators_match_their_operators() {
        let d = Vector::new(1, -1);
        let mut hex: Hex<Flat> = Hex::new(2, 0);
        hex += d;
        assert_eq!(hex, Hex::new(2, 0) + d);
        hex -= d;
        assert_eq!(hex, Hex::new(2, 0));

        let mut p = Point::new(2, 0);
        p += d;
        assert_eq!(p, Point::new(2, 0) + d);

        let mut v = Vector::new(2, 0);
        v -= d;
        assert_eq!(v, Vector::new(2, 0) - d);
    }

    #[test]
    fn neighbors_are_all_adjacent() {
        let hex: Hex<Flat> = Hex::new(3, -1);
        for n in hex.neighbors() {
            assert_eq!(hex.distance(n), 1);
        }
    }

    #[test]
    fn rotations_compose_to_identity() {
        let v = Vector::new(2, -1);
        assert_eq!(v.rotate_clockwise().rotate_counterclockwise(), v);
        let mut w = v;
        for _ in 0..6 {
            w = w.rotate_clockwise();
        }
        assert_eq!(w, v);
    }

    #[test]
    fn reflections_are_involutions() {
        let hex: Hex<Pointy> = Hex::new(3, -1);
        assert_eq!(hex.reflect_q().reflect_q(), hex);
        assert_eq!(hex.reflect_r().reflect_r(), hex);
        assert_eq!(hex.reflect_s().reflect_s(), hex);
    }

    #[test]
    fn range_is_a_filled_disc() {
        let center: Hex<Pointy> = Hex::new(2, -3);
        for radius in 0..4 {
            let cells: Vec<_> = center.range(radius).collect();
            assert_eq!(cells.len() as i32, 3 * radius * radius + 3 * radius + 1);
            assert!(cells.iter().all(|&h| center.distance(h) <= radius));
            assert!(cells.contains(&center));
        }
        assert_eq!(center.range(-1).count(), 0);
    }

    #[test]
    fn ring_is_a_hollow_ring() {
        let center: Hex<Flat> = Hex::new(-1, 4);
        assert_eq!(center.ring(0).collect::<Vec<_>>(), vec![center]);
        assert_eq!(center.ring(-1).count(), 0);
        for radius in 1..4 {
            let cells: Vec<_> = center.ring(radius).collect();
            assert_eq!(cells.len() as i32, 6 * radius);
            assert!(cells.iter().all(|&h| center.distance(h) == radius));
            let unique: std::collections::HashSet<_> = cells.iter().copied().collect();
            assert_eq!(unique.len(), cells.len(), "ring revisits a hex");
        }
    }

    #[test]
    fn rings_partition_range() {
        let sort_key = |h: &Hex<Pointy>| (h.coordinate().q(), h.coordinate().r());
        let center: Hex<Pointy> = Hex::new(1, 1);
        let mut from_rings: Vec<_> = (0..=3).flat_map(|r| center.ring(r)).collect();
        let mut from_range: Vec<_> = center.range(3).collect();
        from_rings.sort_by_key(sort_key);
        from_range.sort_by_key(sort_key);
        assert_eq!(from_rings, from_range);
    }

    #[test]
    fn edge_corner_indices_face_their_neighbor() {
        // The midpoint of the claimed shared edge must be the midpoint
        // between the two hex centers — that is what "shared edge" means.
        fn check<O: Orientation>() {
            let hex: Hex<O> = Hex::new(1, -2);
            let size = 10.0;
            let (cx, cy) = hex.center(size);
            let corners = hex.corners(size);
            for (direction, neighbor) in Direction::ALL.into_iter().zip(hex.neighbors()) {
                let (nx, ny) = neighbor.center(size);
                let (a, b) = Hex::<O>::edge_corner_indices(direction);
                let edge_mid_x = (corners[a].0 + corners[b].0) / 2.0;
                let edge_mid_y = (corners[a].1 + corners[b].1) / 2.0;
                assert!(
                    (edge_mid_x - (cx + nx) / 2.0).abs() < 1e-9
                        && (edge_mid_y - (cy + ny) / 2.0).abs() < 1e-9,
                    "edge for {direction:?} does not face its neighbor"
                );
            }
        }
        check::<Flat>();
        check::<Pointy>();
    }

    #[test]
    fn directions_align_with_vectors() {
        for (direction, vector) in Direction::ALL.into_iter().zip(Vector::DIRECTIONS) {
            assert_eq!(direction.vector(), vector);
            assert_eq!(Vector::from(direction), vector);
            assert_eq!(direction.opposite().vector(), -vector);
            assert_eq!(
                direction.rotate_clockwise().vector(),
                vector.rotate_clockwise()
            );
            assert_eq!(
                direction.rotate_counterclockwise().vector(),
                vector.rotate_counterclockwise()
            );
        }
    }

    #[test]
    fn direction_rotations_compose_to_identity() {
        for direction in Direction::ALL {
            assert_eq!(
                direction.rotate_clockwise().rotate_counterclockwise(),
                direction
            );
            assert_eq!(direction.opposite().opposite(), direction);
            let mut d = direction;
            for _ in 0..6 {
                d = d.rotate_clockwise();
            }
            assert_eq!(d, direction);
        }
    }

    #[test]
    fn neighbor_matches_neighbors_order() {
        let hex: Hex<Pointy> = Hex::new(-2, 3);
        for (direction, expected) in Direction::ALL.into_iter().zip(hex.neighbors()) {
            assert_eq!(hex.neighbor(direction), expected);
        }
    }

    fn roundtrip<O: Orientation>(size: f64) {
        for q in -10..=10 {
            for r in -10..=10 {
                let hex: Hex<O> = Hex::new(q, r);
                let (x, y) = hex.center(size);
                assert_eq!(
                    Hex::<O>::from_pixel(x, y, size),
                    hex,
                    "center roundtrip for ({q}, {r})"
                );

                // Points partway from the center towards each corner must
                // still resolve to the same hex.
                for (cx, cy) in hex.corners(size) {
                    let (ix, iy) = (x + (cx - x) * 0.99, y + (cy - y) * 0.99);
                    assert_eq!(
                        Hex::<O>::from_pixel(ix, iy, size),
                        hex,
                        "near-corner roundtrip for ({q}, {r})"
                    );
                }
            }
        }
    }

    #[test]
    fn from_pixel_inverts_center_flat() {
        roundtrip::<Flat>(32.0);
        roundtrip::<Flat>(1.0);
    }

    #[test]
    fn from_pixel_inverts_center_pointy() {
        roundtrip::<Pointy>(32.0);
        roundtrip::<Pointy>(1.0);
    }
}
