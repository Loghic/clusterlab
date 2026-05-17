//! 2D and 3D point types with basic arithmetic.

use std::ops::{Add, Div, Mul, Sub};

/// A 2D point (or vector) with f32 components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Squared Euclidean distance (avoids sqrt for comparisons).
    pub fn distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Euclidean distance.
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

impl Add for Point2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Point2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Div<f32> for Point2 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}

/// A 3D point (or vector) with f32 components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Squared Euclidean distance (avoids sqrt for comparisons).
    pub fn distance_squared(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    /// Euclidean distance.
    pub fn distance(&self, other: &Self) -> f32 {
        self.distance_squared(other).sqrt()
    }
}

impl Add for Point3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Point3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Point3 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn point2_distance_zero_to_unit() {
        let a = Point2::ZERO;
        let b = Point2::new(3.0, 4.0);
        assert_relative_eq!(a.distance(&b), 5.0);
    }

    #[test]
    fn point2_distance_is_symmetric() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(4.0, 6.0);
        assert_relative_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn point2_distance_squared_no_sqrt() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert_relative_eq!(a.distance_squared(&b), 25.0);
    }

    #[test]
    fn point2_add_sub() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(3.0, 4.0);
        assert_eq!(a + b, Point2::new(4.0, 6.0));
        assert_eq!(b - a, Point2::new(2.0, 2.0));
    }

    #[test]
    fn point2_scalar_ops() {
        let a = Point2::new(2.0, 4.0);
        assert_eq!(a * 2.0, Point2::new(4.0, 8.0));
        assert_eq!(a / 2.0, Point2::new(1.0, 2.0));
    }

    #[test]
    fn point3_distance_pythagorean() {
        let a = Point3::ZERO;
        let b = Point3::new(2.0, 3.0, 6.0);
        assert_relative_eq!(a.distance(&b), 7.0);
    }

    #[test]
    fn point3_distance_squared() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(1.0, 2.0, 2.0);
        assert_relative_eq!(a.distance_squared(&b), 9.0);
    }

    #[test]
    fn point3_arithmetic() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Point3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Point3::new(3.0, 3.0, 3.0));
        assert_eq!(a * 2.0, Point3::new(2.0, 4.0, 6.0));
        assert_eq!(b / 2.0, Point3::new(2.0, 2.5, 3.0));
    }
}
