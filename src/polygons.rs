use vertex::Vertex;

use image::{RgbaImage, Rgba};
use spade::delaunay::FaceHandle;
use svg::node::element;

pub struct Polygon {
    pub vertices: Vec<Vertex>,
    pub colour: Rgba<u8>
}

fn colour(r: u8, b: u8, g: u8) -> Rgba<u8> {
    Rgba {
        data: [r, b, g, 255]
    }
}

impl Polygon {
    pub fn new(face: FaceHandle<Vertex>, image: &RgbaImage) -> Self {
        let vertices: Vec<_> = face.as_triangle().iter().map(|vertex| **vertex).collect();

        Self {
            colour: Self::average_colour(&Self::rasterise(&vertices), image),
            vertices
        }
    }

    pub fn average_colour(points: &[(u32, u32)], image: &RgbaImage) -> Rgba<u8> {
       if !points.is_empty() {
            let (r, g, b) = points.iter()
                .filter(|&&(x, y)| x < image.width() && y < image.height())
                .map(|&(x, y)| image.get_pixel(x, y).data)
                .fold((0, 0, 0), |mut total, pixel| {
                    total.0 += pixel[0] as usize;
                    total.1 += pixel[1] as usize;
                    total.2 += pixel[2] as usize;
                    total
                });

            colour(
                (r / points.len()) as u8,
                (g / points.len()) as u8,
                (b / points.len()) as u8
            )
        } else {
            colour(0, 0, 0)
        }
    }

    pub fn rasterise(vertices: &[Vertex]) -> Vec<(u32, u32)> {
        let min_x = vertices.iter().map(|vertex| vertex.x.floor() as i32).min().unwrap();
        let min_y = vertices.iter().map(|vertex| vertex.y.floor() as i32).min().unwrap();
        let max_x = vertices.iter().map(|vertex| vertex.x.ceil()  as i32).max().unwrap();
        let max_y = vertices.iter().map(|vertex| vertex.y.ceil()  as i32).max().unwrap();

        let mut vec = Vec::new();

        for x in min_x .. max_x {
            for y in min_y .. max_y {
                if Self::contains(vertices, x as f32, y as f32) {
                    vec.push((x as u32, y as u32));
                }
            }
        }

        vec
    }

    fn contains(vertices: &[Vertex], x: f32, y: f32) -> bool {
        let mut inside = false;
        let num_vertices = vertices.len();

        let mut j = num_vertices - 1;

        for i in 0 .. num_vertices {
            let ip = vertices[i];
            let jp = vertices[j];

            if  ((ip.y > y) != (jp.y > y)) &&
                (x < (jp.x - ip.x) * (y - ip.y) / (jp.y - ip.y) + ip.x)
            {
                inside = !inside;
            }

            j = i;
        }

        inside
    }

    fn to_string(&self) -> String {
        self.vertices.iter()
            .map(|vertex| format!("{},{} ", vertex.x, vertex.y))
            .collect()
    }

    pub fn to_svg(&self) -> element::Polygon {
        element::Polygon::new()
            .set("points", self.to_string())
            .set("fill", format!("rgb({}, {}, {})", self.colour.data[0], self.colour.data[1], self.colour.data[2]))
    }
}