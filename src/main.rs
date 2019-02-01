use crate::geometry::Vec3f;
use png::HasParameters;
use std::f32::consts::FRAC_PI_2;
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Index, IndexMut};

mod geometry;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug, Clone, Copy)]
struct Light {
    position: Vec3f,
    intensity: f32,
}

impl Light {
    fn new(position: Vec3f, intensity: f32) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Material {
    albedo: Vec3f,
    diffuse_color: Vec3f,
    specular_exponent: f32,
}

impl Material {
    fn new(albedo: Vec3f, color: Vec3f, spec: f32) -> Self {
        Self {
            albedo,
            diffuse_color: color,
            specular_exponent: spec,
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
    lights: Vec<Light>,
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
            lights: Vec::new(),
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.framebuffer.len() / self.width
    }

    fn scene_intersect(&self, orig: Vec3f, direction: Vec3f) -> Option<(Vec3f, Vec3f, Material)> {
        let mut spheres_dist = std::f32::MAX;
        let mut hit = Vec3f::default();
        let mut n = Vec3f::default();
        let mut material = Material::default();
        let mut dist_i = 0f32;
        for sphere in &self.spheres {
            if sphere.ray_intersect(orig, direction, &mut dist_i) && dist_i < spheres_dist {
                spheres_dist = dist_i;
                hit = orig + direction * dist_i;
                n = (hit - sphere.center).normalize();
                material = sphere.material;
            }
        }
        if spheres_dist < 1000f32 {
            Some((hit, n, material))
        } else {
            None
        }
    }

    fn cast_ray(&self, orig: Vec3f, dir: Vec3f, depth: Option<usize>) -> Vec3f {
        if let Some((point, n, material)) = depth.and_then(|_| self.scene_intersect(orig, dir)) {
            let reflect_dir = dir.reflect(n).normalize();
            let reflect_orig = if reflect_dir * n < 0f32 {
                point - n * 1e-3
            } else {
                point + n * 1e-3
            };
            let reflect_color = self.cast_ray(
                reflect_orig,
                reflect_dir,
                depth.map(|d| d + 1).filter(|d| *d <= 4),
            );
            let mut diffuse_light_intensity = 0f32;
            let mut specular_light_intensity = 0f32;
            for light in &self.lights {
                let light_dir = (light.position - point).normalize();
                let light_distance = (light.position - point).norm();
                let shadow_orig = if light_dir * n < 0f32 {
                    point - n * 1e-3
                } else {
                    point + n * 1e-3
                };
                if let Some((shadow_pt, _, _)) = self.scene_intersect(shadow_orig, light_dir) {
                    if (shadow_pt - shadow_orig).norm() < light_distance {
                        continue;
                    }
                }
                diffuse_light_intensity += light.intensity * 0f32.max(light_dir * n);
                specular_light_intensity += 0f32
                    .max(-(-light_dir).reflect(n) * dir)
                    .powf(material.specular_exponent)
                    * light.intensity;
            }
            material.diffuse_color * diffuse_light_intensity * material.albedo.0[0]
                + Vec3f::new(1.0, 1.0, 1.0) * specular_light_intensity * material.albedo.0[1]
                + reflect_color * material.albedo.0[2]
        } else {
            Vec3f::new(0.2, 0.7, 0.8) // background color
        }
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
                    Some(0),
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
                .iter_mut()
                .flat_map(|p| {
                    let max = p.0[0].max(p.0[1].max(p.0[2]));
                    if max > 1f32 {
                        *p = *p * (1f32 / max);
                    }
                    p.as_bytes()
                })
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }

    fn add_sphere(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }

    fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }
}

fn main() {
    let mut image = Image::new(1024, 768);
    let ivory = Material::new(Vec3f::new(0.6, 0.3, 0.1), Vec3f::new(0.4, 0.4, 0.3), 50.0);
    let red_rubber = Material::new(Vec3f::new(0.9, 0.1, 0.0), Vec3f::new(0.3, 0.1, 0.1), 10.0);
    let mirror = Material::new(
        Vec3f::new(0.0, 10.0, 0.8),
        Vec3f::new(1.0, 1.0, 1.0),
        1425.0,
    );

    image.add_sphere(Sphere::new(Vec3f::new(-3.0, 0.0, -16.0), 2.0, ivory));
    image.add_sphere(Sphere::new(Vec3f::new(-1.0, -1.5, -12.0), 2.0, mirror));
    image.add_sphere(Sphere::new(Vec3f::new(1.5, -0.5, -18.0), 3.0, red_rubber));
    image.add_sphere(Sphere::new(Vec3f::new(7.0, 5.0, -18.0), 4.0, mirror));
    image.add_light(Light::new(Vec3f::new(-20.0, 20.0, 20.0), 1.5));
    image.add_light(Light::new(Vec3f::new(30.0, 50.0, -25.0), 1.8));
    image.add_light(Light::new(Vec3f::new(30.0, 20.0, 30.0), 1.7));
    image.render().expect("render");
}
