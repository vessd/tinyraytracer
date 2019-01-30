use crate::geometry::Vec3f;
use png::HasParameters;
use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Index, IndexMut};

mod geometry;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug, Default, Clone, Copy)]
struct Material {
    diffuse_color: Vec3f,
}

impl Material {
    fn new(color: Vec3f) -> Self {
        Self {
            diffuse_color: color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Sphere {
    center: Vec3f,
    radius: f32,
    material: Material,
}

impl Sphere {
    fn new(center: Vec3f, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
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
}

#[derive(Debug)]
struct Image {
    framebuffer: Vec<Vec3f>,
    width: usize,
    fov: f32,
    spheres: Vec<Sphere>,
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
            spheres: Vec::new(),
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.framebuffer.len() / self.width
    }

    fn scene_intersect(&self, orig: Vec3f, direction: Vec3f) -> Option<Vec3f> {
        let mut spheres_dist = std::f32::MAX;
        let mut material = Material::default();
        let mut dist_i = 0f32;
        for i in 0..self.spheres.len() {
            if self.spheres[i].ray_intersect(orig, direction, &mut dist_i) && dist_i < spheres_dist
            {
                spheres_dist = dist_i;
                material = self.spheres[i].material;
            }
        }
        if spheres_dist < 1000f32 {
            Some(material.diffuse_color)
        } else {
            None
        }
    }

    fn cast_ray(&self, orig: Vec3f, direction: Vec3f) -> Vec3f {
        self.scene_intersect(orig, direction)
            .unwrap_or(Vec3f::new(0.2, 0.7, 0.8))
    }

    fn render(&mut self) -> Result<()> {
        let w = self.width() as f32;
        let h = self.height() as f32;

        for i in 0..self.height() {
            for j in 0..self.width() {
                let dir_x = (j as f32 + 0.5) - w / 2f32;
                let dir_y = -(i as f32 + 0.5) + h / 2f32;
                let dir_z = -h / (2f32 * (self.fov / 2f32).tan());
                self[i][j] = self.cast_ray(
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

    fn add_sphere(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }
}

fn main() {
    let width = 1024;
    let height = 768;
    let mut image = Image::new(width, height);
    let ivory = Material::new(Vec3f::new(0.4, 0.4, 0.3));
    let red_rubber = Material::new(Vec3f::new(0.3, 0.1, 0.1));
    image.add_sphere(Sphere::new(Vec3f::new(-3.0, 0.0, -16.0), 2.0, ivory));
    image.add_sphere(Sphere::new(Vec3f::new(-1.0, -1.5, -12.0), 2.0, red_rubber));
    image.add_sphere(Sphere::new(Vec3f::new(1.5, -0.5, -18.0), 3.0, red_rubber));
    image.add_sphere(Sphere::new(Vec3f::new(7.0, 5.0, -18.0), 4.0, ivory));
    image.render().expect("render");
}
