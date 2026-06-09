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

pub trait Orientation {}

#[derive(Debug, Clone)]
pub struct Flat;

impl Orientation for Flat {}

#[derive(Debug, Clone)]
pub struct Pointy;

impl Orientation for Pointy {}

/// A hexagon defined by its canonical coordinate in space.
/// Orientation of the hexagon is encoded at the type level.
#[derive(Copy, Clone, Debug)]
pub struct Hex<O: Orientation> {
    coordinate: Point,
    _phantom: PhantomData<O>,
}

// Basic error unit struct returned for an invalid coordinate.
pub struct Invalid;

/// The conversion from a tuple containing all three elements of a coordinate
/// is fallable as the contraint `q + r + s = 0` must hold.
impl TryFrom<(i32, i32, i32)> for Point {
    type Error = Invalid;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        if value.0 + value.1 + value.2 != 0 {
            Err(Invalid)
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
    type Error = Invalid;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        if value.0 + value.1 + value.2 != 0 {
            Err(Invalid)
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
    type Error = Invalid;

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
    pub fn neighbours(&self) -> [Hex<O>; 6] {
        Vector::UNIT_VECTORS.map(|x| (self.coordinate + x).into())
    }
}

impl<O: Orientation> Hex<O> {
    pub fn reflect_q(self) -> Self {
        Self {
            coordinate: Point {
                q: self.coordinate.q,
                r: self.coordinate.s,
                s: self.coordinate.r,
            },
            _phantom: PhantomData,
        }
    }

    pub fn reflect_r(self) -> Self {
        Self {
            coordinate: Point {
                q: self.coordinate.s,
                r: self.coordinate.r,
                s: self.coordinate.q,
            },
            _phantom: PhantomData,
        }
    }

    pub fn reflect_s(self) -> Self {
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
    pub fn distance(self, other: Self) -> i32 {
        let diff = self - other;
        (diff.q.abs() + diff.r.abs() + diff.s.abs()) / 2
    }
}
