use crate::geometry::Vec3f;
use png::HasParameters;
use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Index, IndexMut};

mod geometry;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug)]
struct Image {
    framebuffer: Vec<Vec3f>,
    width: usize,
    fov: f32,
}

impl Index<usize> for Image {
    type Output = [Vec3f];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        let i = index * self.width;
        &self.framebuffer[i..i + self.width]
    }
}

impl IndexMut<usize> for Image {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let i = index * self.width;
        &mut self.framebuffer[i..i + self.width]
    }
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            framebuffer: vec![Vec3f::default(); width * height],
            width,
            fov: FRAC_PI_2,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.framebuffer.len() / self.width
    }

    fn render(&mut self, sphere: &Sphere) -> Result<()> {
        let w = self.width() as f32;
        let h = self.height() as f32;

        for i in 0..self.height() {
            for j in 0..self.width() {
                let dir_x = (i as f32 + 0.5) - w / 2f32;
                let dir_y = -(j as f32 + 0.5) + h / 2f32;
                let dir_z = -h / (2f32 * (self.fov / 2f32).tan());
                self[i][j] = sphere.cast_ray(
                    Vec3f::new(0f32, 0f32, 0f32),
                    Vec3f::new(dir_x, dir_y, dir_z).normalize(),
                );
            }
        }

        let w = BufWriter::new(File::create("image.png")?);
        let mut encoder = png::Encoder::new(w, self.width() as u32, self.height() as u32);
        encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(
            &self
                .framebuffer
                .iter()
                .flat_map(|p| p.as_bytes())
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }
}

struct Sphere {
    center: Vec3f,
    radius: f32,
}

impl Sphere {
    fn new(center: Vec3f, radius: f32) -> Self {
        Self { center, radius }
    }

    fn ray_intersect(&self, p: Vec3f, direction: Vec3f, t0: &mut f32) -> bool {
        let vcp = self.center - p;
        let tca = vcp * direction;
        let d2 = vcp * vcp - tca * tca;
        if d2 > self.radius * self.radius {
            return false;
        }
        let thc = (self.radius * self.radius - d2).sqrt();
        *t0 = tca - thc;
        let t1 = tca + thc;
        if *t0 < 0f32 {
            *t0 = t1;
        }
        if *t0 < 0f32 {
            return false;
        }
        true
    }

    fn cast_ray(&self, p: Vec3f, direction: Vec3f) -> Vec3f {
        let mut sphere_dist = std::f32::MAX;
        if !self.ray_intersect(p, direction, &mut sphere_dist) {
            Vec3f::new(0.2, 0.7, 0.8) // background color
        } else {
            Vec3f::new(0.4, 0.4, 0.3)
        }
    }
}

fn main() {
    let width = 1024;
    let height = 768;
    let mut image = Image::new(width, height);
    let sphere = Sphere::new(Vec3f::new(-3f32, 0f32, -16f32), 2f32);
    image.render(&sphere).expect("render");
}
