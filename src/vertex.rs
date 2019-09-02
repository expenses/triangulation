use ggez::graphics::{Point2, Color};
use spade::{PointN, TwoDimensional};
use image::RgbaImage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, y
        }
    }

    pub fn as_point2(&self) -> Point2 {
        Point2::new(self.x, self.y)
    }

    pub fn get_colour(&self, image: &RgbaImage) -> Color {
        let x = self.x.round() as u32;
        let y = self.y.round() as u32;

        if x < image.width() && y < image.height() {
            let pixel = image.get_pixel(x, y).data;
            Color::from_rgba(pixel[0], pixel[1], pixel[2], pixel[3])
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn add(&self, point: &Point2) -> Self {
        Self {
            x: self.x + point.coords.x,
            y: self.y + point.coords.y
        }
    }

    pub fn scale(&self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar
        }
    }
}

impl PointN for Vertex {
    type Scalar = f32;

    fn dimensions() -> usize {
        2
    }

    fn from_value(value: Self::Scalar) -> Self {
        Self {
            x: value,
            y: value
        }
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.x,
            _ => &self.y,
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            _ => &mut self.y
        }
    }
}

impl TwoDimensional for Vertex {}