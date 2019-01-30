use std::ops::{Mul, Sub};

#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3f([f32; 3]);

impl Sub for Vec3f {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

impl Mul for Vec3f {
    type Output = f32;

    fn mul(self, other: Self) -> f32 {
        self.0.iter().zip(other.0.iter()).map(|(l, r)| l * r).sum()
    }
}

impl Mul<f32> for Vec3f {
    type Output = Vec3f;

    fn mul(self, other: f32) -> Self {
        Self([self.0[0] * other, self.0[1] * other, self.0[2] * other])
    }
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    pub fn as_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.0
            .iter()
            .map(|b| (b.min(1f32).max(0f32) * 255f32) as u8)
    }

    fn norm(&self) -> f32 {
        (self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]).sqrt()
    }

    pub fn normalize(&self) -> Self {
        *self * (1f32 / self.norm())
    }
}
