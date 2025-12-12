use std::ops::{Add, Div, Mul, Sub};

#[derive(Default, Clone, Copy)]
pub struct Coordinates {
    x: i32,
    y: i32,
    z: i32,
}

impl Coordinates {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn new_uniform(value: i32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    /// Creates coordinates from a linear index given the width and length dimensions.
    /// The index is assumed to be in y-major order: y * width * length + z * width + x
    pub fn from_index(index: usize, width: i32, length: i32) -> Self {
        let i = index as i32;
        Self {
            x: i % width,
            z: (i / width) % length,
            y: i / (width * length),
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn z(&self) -> i32 {
        self.z
    }
}

impl Add for Coordinates {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Coordinates {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul for Coordinates {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Add<i32> for Coordinates {
    type Output = Self;

    fn add(self, rhs: i32) -> Self {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl Mul<i32> for Coordinates {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<i32> for Coordinates {
    type Output = Self;

    fn div(self, rhs: i32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
