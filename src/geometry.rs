use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3f(pub [f32; 3]);

impl Add for Vec3f {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl Neg for Vec3f {
    type Output = Self;

    fn neg(self) -> Self {
        self * -1f32
    }
}

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

    pub fn norm(self) -> f32 {
        (self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]).sqrt()
    }

    pub fn normalize(self) -> Self {
        self * (1f32 / self.norm())
    }

    pub fn reflect(self, p: Self) -> Self {
        self - p * 2f32 * (self * p)
    }

    pub fn refract(self, p: Self, mut etat: f32) -> Self {
        let mut cosi = -(self * p).min(1f32).max(-1f32);
        let mut etai = 1f32;
        let n = if cosi < 0f32 {
            cosi = -cosi;
            std::mem::swap(&mut etai, &mut etat);
            -p
        } else {
            p
        };
        let eta = etai / etat;
        let k = 1f32 - eta * eta * (1f32 - cosi * cosi);
        if k < 0f32 {
            Vec3f::new(0.0, 0.0, 0.0)
        } else {
            self * eta + n * (eta * cosi - k.sqrt())
        }
    }
}
