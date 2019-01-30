use png::HasParameters;
use std::fs::File;
use std::io::BufWriter;
use std::ops::Index;
use std::ops::IndexMut;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

#[derive(Debug, Default, Clone, Copy)]
struct Pixel([f32; 3]);

impl Pixel {
    fn new(red: f32, green: f32, blue: f32) -> Self {
        Self([red, green, blue])
    }

    fn as_bytes(&self) -> impl std::iter::Iterator<Item = u8> + '_ {
        self.0.iter().map(|b| (b * 255f32) as u8)
    }
}

#[derive(Debug)]
struct Image {
    framebuffer: Vec<Pixel>,
    width: usize,
}

impl Index<usize> for Image {
    type Output = [Pixel];

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
            framebuffer: vec![Pixel::default(); width * height],
            width,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.framebuffer.len() / self.width
    }

    fn render(&self) -> Result<()> {
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

fn main() {
    let w = 1024;
    let h = 768;
    let mut image = Image::new(w, h);
    for i in 0..h {
        for j in 0..w {
            image[i][j] = Pixel::new(i as f32 / h as f32, j as f32 / w as f32, 0f32);
        }
    }
    image.render().expect("render");
}
