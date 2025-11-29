use std::fmt;
use std::ops::{Add, Sub};

/// A location in 2d space in the cube coordinate system, `(q, r, s)`.
/// The constriant q + r + s = 0 holds.
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

/// A pixel represented by a cartesian point
#[derive(Copy, Clone, Debug)]
pub struct Pixel {
    x: f64,
    y: f64,
}

type Mat2 = [f64; 4];

pub struct BasisVector {
    /// 2x2 forward matrix
    pub f: Mat2,
    /// 2x2 inverse matrix
    pub b: Mat2,
    pub start_angle: f64, // these increase in 60 degree or PI/3 increments
    /// angle of left edge from start corner
    pub start_angle_l: f64,
    /// angle of right edge from start corner
    pub start_angle_r: f64,
}

/// Marker const representing a hexagon configured in a flat orientation
pub const FLAT: bool = true;
/// Marker const representing a hexagon configured in a pointy orientation
pub const POINTY: bool = false;

/// A hexagon defined by its canonical coordinate in space.
/// Orientation of the hexagon is encoded at the type level by the boolean const generic `IS_FLAT`.
#[derive(Copy, Clone, Debug)]
pub struct Hex<const IS_FLAT: bool>(Point);

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

impl<const IS_FLAT: bool> TryFrom<(i32, i32, i32)> for Hex<IS_FLAT> {
    type Error = Invalid;

    fn try_from(value: (i32, i32, i32)) -> Result<Self, Self::Error> {
        Ok(Hex(value.try_into()?))
    }
}

impl<const IS_FLAT: bool> From<(i32, i32)> for Hex<IS_FLAT> {
    fn from(item: (i32, i32)) -> Self {
        Hex(item.into())
    }
}

impl<const IS_FLAT: bool> From<Point> for Hex<IS_FLAT> {
    fn from(value: Point) -> Self {
        Hex(value)
    }
}

impl<const IS_FLAT: bool> From<Hex<IS_FLAT>> for Point {
    fn from(value: Hex<IS_FLAT>) -> Self {
        value.0
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

impl<const IS_FLAT: bool> Hex<IS_FLAT> {
    pub fn neighbours(&self) -> [Hex<IS_FLAT>; 6] {
        Vector::UNIT_VECTORS.map(|x| (self.0 + x).into())
    }
}

impl Hex<FLAT> {
    // Need to calculate upfront. Cannot be const without removing the call to sqrt().
    //
    // const FLAT_BASIS_VECTOR = BasisVector {
    //     f: [3. / 2., 0., 3.0f64.sqrt() / 2., 3.0f64.sqrt()],
    //     b: [2. / 3., 0., -1. / 3., 3.0f64.sqrt() / 3.],
    //     start_angle: 0.,
    //     start_angle_l: (4. / 3.) * PI,
    //     start_angle_r: (2. / 3.) * PI,
    // };

    pub fn width(size: i32) -> f64 {
        2f64 * f64::from(size)
    }

    pub fn height(size: i32) -> f64 {
        3f64.sqrt() * f64::from(size)
    }
}

impl Hex<POINTY> {
    // Need to calculate upfront. Cannot be const without removing the call to sqrt().
    //
    // const POINTY_BASIS_VECTOR = BasisVector {
    //     f: [3. / 2., 0., 3.0f64.sqrt() / 2., 3.0f64.sqrt()],
    //     b: [2. / 3., 0., -1. / 3., 3.0f64.sqrt() / 3.],
    //     start_angle: 0.,
    //     start_angle_l: (4. / 3.) * PI,
    //     start_angle_r: (2. / 3.) * PI,
    // };

    pub fn width(size: i32) -> f64 {
        3f64.sqrt() * f64::from(size)
    }

    pub fn height(size: i32) -> f64 {
        2f64 * f64::from(size)
    }
}

impl<const IS_FLAT: bool> From<Hex<IS_FLAT>> for Pixel {
    fn from(_: Hex<IS_FLAT>) -> Self {
        Pixel { x: 0f64, y: 0f64 } // FIX ME
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
