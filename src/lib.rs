use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Sub};

/// A location in 2d space in the cube coordinate system, `(q, r, s)`.
/// The constriant `q + r + s = 0` holds.
#[derive(Copy, Clone, Debug)]
pub struct Point {
    q: i32,
    r: i32,
    s: i32,
}

/// A displacement in space in the cube coordinate system.
#[derive(Copy, Clone, Debug)]
pub struct Vector {
    q: i32,
    r: i32,
    s: i32,
}

/// √3 as an f64 literal, since `f64::sqrt` is not a `const fn`.
const SQRT_3: f64 = 1.732_050_807_568_877_2;
/// √3 / 2
const SQRT_3_2: f64 = SQRT_3 / 2.0;

pub trait Orientation {
    /// Forward hex-to-pixel matrix `[f0, f1, f2, f3]` for a hex of size 1,
    /// mapping `(q, r)` to a pixel `(x, y)`.
    const FORWARD: [f64; 4];
    /// Inverse pixel-to-hex matrix `[b0, b1, b2, b3]` for a hex of size 1,
    /// mapping a pixel `(x, y)` to a fractional `(q, r)`.
    const INVERSE: [f64; 4];
    /// The angle of the first corner, in multiples of 60 degrees.
    const START_ANGLE: f64;
}

#[derive(Debug, Clone, Copy)]
pub struct Flat;

impl Orientation for Flat {
    const FORWARD: [f64; 4] = [1.5, 0.0, SQRT_3_2, SQRT_3];
    const INVERSE: [f64; 4] = [2.0 / 3.0, 0.0, -1.0 / 3.0, SQRT_3 / 3.0];
    const START_ANGLE: f64 = 0.0;
}

#[derive(Debug, Clone, Copy)]
pub struct Pointy;

impl Orientation for Pointy {
    const FORWARD: [f64; 4] = [SQRT_3, SQRT_3_2, 0.0, 1.5];
    const INVERSE: [f64; 4] = [SQRT_3 / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0];
    const START_ANGLE: f64 = 0.5;
}

/// A hexagon defined by its canonical coordinate in space.
/// Orientation of the hexagon is encoded at the type level.
#[derive(Copy, Clone, Debug)]
pub struct Hex<O: Orientation> {
    coordinate: Point,
    _phantom: PhantomData<O>,
}

/// Error returned when a cube coordinate violates the constraint `q + r + s = 0`.
///
/// Carries the offending coordinate so callers can report what was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCoordinate {
    pub q: i32,
    pub r: i32,
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
/// is fallable as the contraint `q + r + s = 0` must hold.
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
        Self {
            q: value.0,
            r: value.1,
            s: -value.0 - value.1,
        }
    }
}

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
        Self {
            q: value.0,
            r: value.1,
            s: -value.0 - value.1,
        }
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

impl<O: Orientation> TryFrom<(i32, i32, i32)> for Hex<O> {
    type Error = InvalidCoordinate;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        Ok(Self {
            coordinate: value.try_into()?,
            _phantom: PhantomData,
        })
    }
}

impl<O: Orientation> From<(i32, i32)> for Hex<O> {
    fn from(value: (i32, i32)) -> Self {
        Self {
            coordinate: value.into(),
            _phantom: PhantomData,
        }
    }
}

impl<O: Orientation> From<Point> for Hex<O> {
    fn from(value: Point) -> Self {
        Self {
            coordinate: value,
            _phantom: PhantomData,
        }
    }
}

impl<O: Orientation> From<Hex<O>> for Point {
    fn from(value: Hex<O>) -> Self {
        value.coordinate
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "q: {}, r: {}, s: {}", self.q, self.r, self.s)
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "q: {}, r: {}, s: {}", self.q, self.r, self.s)
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

impl<O: Orientation> Hex<O> {
    /// The canonical coordinate of this hex.
    pub fn coordinate(&self) -> Point {
        self.coordinate
    }

    pub fn neighbours(&self) -> [Hex<O>; 6] {
        Vector::UNIT_VECTORS.map(|x| (self.coordinate + x).into())
    }

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

    /// The hex containing the pixel coordinate `(x, y)` for the given `size`.
    /// This is the inverse of [`Hex::center`].
    pub fn from_pixel((x, y): (f64, f64), size: f64) -> Self {
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
        (rq as i32, rr as i32).into()
    }
}

impl<O: Orientation> Hex<O> {
    pub fn reflect_q(&self) -> Self {
        Self {
            coordinate: Point {
                q: self.coordinate.q,
                r: self.coordinate.s,
                s: self.coordinate.r,
            },
            _phantom: PhantomData,
        }
    }

    pub fn reflect_r(&self) -> Self {
        Self {
            coordinate: Point {
                q: self.coordinate.s,
                r: self.coordinate.r,
                s: self.coordinate.q,
            },
            _phantom: PhantomData,
        }
    }

    pub fn reflect_s(&self) -> Self {
        Self {
            coordinate: Point {
                q: self.coordinate.r,
                r: self.coordinate.q,
                s: self.coordinate.s,
            },
            _phantom: PhantomData,
        }
    }
}

impl Hex<Flat> {
    pub fn width(size: i32) -> f64 {
        2f64 * f64::from(size)
    }

    pub fn height(size: i32) -> f64 {
        3f64.sqrt() * f64::from(size)
    }
}

impl Hex<Pointy> {
    pub fn width(size: i32) -> f64 {
        3f64.sqrt() * f64::from(size)
    }

    pub fn height(size: i32) -> f64 {
        2f64 * f64::from(size)
    }
}

impl Vector {
    const UNIT_VECTORS: [Self; 6] = [
        Self { q: 1, r: 0, s: -1 },
        Self { q: 0, r: 1, s: -1 },
        Self { q: -1, r: 1, s: 0 },
        Self { q: -1, r: 0, s: 1 },
        Self { q: 0, r: -1, s: 1 },
        Self { q: 1, r: -1, s: 0 },
    ];

    /// Scale the vector by a given magnitude
    pub fn scale(&self, factor: i32) -> Self {
        (self.q * factor, self.r * factor).into()
    }

    /// Rotate the vector 60 degrees anti-clockwise
    pub fn rotate_anticlockwise(&self) -> Self {
        (self.q + self.r, -self.q).into()
    }

    /// Rotate the vector 60 degrees clockwise
    pub fn rotate_clockwise(&self) -> Self {
        (-self.r, self.q + self.r).into()
    }

    /// The length of the vector
    pub fn len(&self) -> i32 {
        (self.q.abs() + self.r.abs() + self.s.abs()) / 2
    }

    /// Distance between the vector and other
    pub fn distance(self, other: Self) -> i32 {
        (self - other).len()
    }
}

impl Point {
    pub fn q(&self) -> i32 {
        self.q
    }

    pub fn r(&self) -> i32 {
        self.r
    }

    pub fn s(&self) -> i32 {
        self.s
    }

    pub fn distance(self, other: Self) -> i32 {
        let diff = self - other;
        (diff.q.abs() + diff.r.abs() + diff.s.abs()) / 2
    }
}
