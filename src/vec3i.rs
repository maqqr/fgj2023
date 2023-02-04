use std::ops::{Add, AddAssign, Sub, SubAssign, Neg};

use bevy::{prelude::{Vec3, Transform}, reflect::Reflect};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Reflect)]
pub struct Vec3i(i64, i64, i64);

impl Vec3i {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self(x, y, z)
    }

    #[inline]
    pub fn x(self) -> i64 {
        self.0
    }

    #[inline]
    pub fn y(self) -> i64 {
        self.1
    }

    #[inline]
    pub fn z(self) -> i64 {
        self.2
    }

    #[inline]
    pub fn x_mut(&mut self) -> &mut i64 {
        &mut self.0
    }

    #[inline]
    pub fn y_mut(&mut self) -> &mut i64 {
        &mut self.1
    }

    #[inline]
    pub fn z_mut(&mut self) -> &mut i64 {
        &mut self.2
    }

    #[inline]
    pub fn set_x(&mut self, x: i64) {
        self.0 = x;
    }

    #[inline]
    pub fn set_y(&mut self, y: i64) {
        self.1 = y;
    }

    #[inline]
    pub fn set_z(&mut self, z: i64) {
        self.2 = z;
    }
}

impl Add for Vec3i {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl AddAssign for Vec3i {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
        self.2 += other.2;
    }
}

impl Sub for Vec3i {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl SubAssign for Vec3i {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
        self.1 -= other.1;
        self.2 -= other.2;
    }
}

impl Neg for Vec3i {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0, -self.1, -self.2)
    }
}

impl From<Vec3i> for Vec3 {
    #[inline]
    fn from(t: Vec3i) -> Self {
        Self::new(t.0 as f32, t.1 as f32, t.2 as f32)
    }
}

impl From<Vec3i> for Transform {
    #[inline]
    fn from(t: Vec3i) -> Self {
        Self::from_xyz(t.0 as f32, t.1 as f32, t.2 as f32)
    }
}

impl From<(i64, i64, i64)> for Vec3i {
    #[inline]
    fn from((x, y, z): (i64, i64, i64)) -> Self {
        Self::new(x, y, z)
    }
}
