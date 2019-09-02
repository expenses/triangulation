extern crate ggez;
extern crate spade;
extern crate image;
#[macro_use]
extern crate error_chain;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate svg;

mod polygons;
mod vertex;
use vertex::Vertex;
use polygons::Polygon;

use ggez::*;
use ggez::graphics::{line, Image, Drawable, set_color, Color, WHITE, DrawParam, Point2, polygon, DrawMode};
use ggez::event::{EventHandler, MouseButton, Keycode, Mod};
use spade::delaunay::{FloatDelaunayTriangulation, DelaunayTreeLocate};
use image::RgbaImage;
use structopt::StructOpt;
use svg::Document;

use std::str::FromStr;

mod err {
    use ggez;
    use image;

    error_chain!{
        foreign_links {
            Ggez(ggez::GameError);
            Image(image::ImageError);
        }
    }
}

#[derive(Default)]
struct Keys {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    zoom_out: bool,
    zoom_in: bool
}

struct MainState {
    delaunay: FloatDelaunayTriangulation<Vertex, DelaunayTreeLocate<Vertex>>,
    image: Image,
    rgba_image: RgbaImage,
    scale: f32,
    x: f32,
    y: f32,
    keys: Keys,
    show_faces: bool,
    show_edges: bool,
    save: bool,
    polygon_cache: Vec<Polygon>
}

impl MainState {
    fn new(rgba_image: RgbaImage, ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            image: Image::from_rgba8(ctx, rgba_image.width() as u16, rgba_image.height() as u16, &rgba_image.to_vec())?,
            delaunay: FloatDelaunayTriangulation::new(),
            rgba_image,
            scale: 1.0,
            x: 0.0,
            y: 0.0,
            keys: Keys::default(),
            show_faces: false,
            show_edges: true,
            save: false,
            polygon_cache: Vec::new()
        })
    }

    fn write_svg(&self, filename: &str) -> Result<(), std::io::Error> {
        let doc = self.polygon_cache.iter()
            .fold(Document::new(), |document, poly| document.add(poly.to_svg()));

        svg::save(filename, &doc)
    }

    fn handle_key_press(&mut self, key: Keycode, pressed: bool) {
        match key {
            Keycode::Equals => self.keys.zoom_in  = pressed,
            Keycode::Minus  => self.keys.zoom_out = pressed,
            Keycode::W      => self.keys.up       = pressed,
            Keycode::A      => self.keys.left     = pressed,
            Keycode::S      => self.keys.down     = pressed,
            Keycode::D      => self.keys.right    = pressed,
            _ => {}
        }
    }

    fn set_polygon_cache(&mut self) {
        self.polygon_cache = self.delaunay.triangles()
            .map(|face| Polygon::new(face, &self.rgba_image))
            .collect();
    }

    fn visible(&self, point: Point2) -> bool {
        let x = point.coords.x;
        let y = point.coords.y;

        x > self.x && y > self.y && x < 1280.0 && y < 960.0
    }
}

fn mix_colours(colour_a: &Color, colour_b: &Color) -> Color {
    Color::new(
        1.0 - (colour_a.r + colour_b.r) / 2.0,
        1.0 - (colour_a.b + colour_b.g) / 2.0,
        1.0 - (colour_a.g + colour_b.b) / 2.0,
        1.0
    )
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.keys.zoom_in  { self.scale *= 1.02; }
        if self.keys.zoom_out { self.scale /= 1.02; }
        if self.keys.left     { self.x += 5.0 / self.scale; }
        if self.keys.right    { self.x -= 5.0 / self.scale; }
        if self.keys.up       { self.y += 5.0 / self.scale; }
        if self.keys.down     { self.y -= 5.0 / self.scale; }

        if self.save {
            self.write_svg("filename.svg")?;
            self.save = false;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        set_color(ctx, WHITE)?;
        graphics::clear(ctx);

        let pos = Point2::new(self.x, self.y);

        self.image.draw_ex(ctx, DrawParam {
            dest: pos * self.scale,
            scale: Point2::new(self.scale, self.scale),
            .. Default::default()
        })?;
        
        if self.show_faces {
            for poly in &self.polygon_cache {
                let colour = poly.colour.data;
                set_color(ctx, Color::from_rgba(colour[0], colour[1], colour[2], colour[3]))?;
                let points: Vec<_> = poly.vertices.iter().map(|vert| vert.add(&pos).scale(self.scale).as_point2()).collect();

                polygon(ctx, DrawMode::Fill, &points)?;
            }
        }

        if self.show_edges {
            for edge in self.delaunay.edges() {
                let from = edge.from();
                let to = edge.to();

                let from_point = from.add(&pos).scale(self.scale).as_point2();
                let to_point   = to.add(&pos).scale(self.scale).as_point2();

                if self.visible(from_point) || self.visible(to_point) {
                    let from_col = from.get_colour(&self.rgba_image);
                    let to_col = to.get_colour(&self.rgba_image);
                    set_color(ctx, mix_colours(&from_col, &to_col))?;

                    line(ctx, &[from_point, to_point], 1.0)?;
                }
            }
        }

        graphics::present(ctx);

        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, x: i32, y: i32) {
        let x = x as f32 / self.scale - self.x;
        let y = y as f32 / self.scale - self.y;

        self.delaunay.insert(Vertex::new(x, y));
        self.set_polygon_cache();

        //self.svg("wow.svg");
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: Keycode, _mod: Mod, _repeat: bool) {
        match keycode {
            Keycode::F => self.show_faces = !self.show_faces,
            Keycode::E => self.show_edges = !self.show_edges,
            Keycode::C => self.save = true,
            _ => self.handle_key_press(keycode, true)
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: Keycode, _mod: Mod, _repeat: bool) {
        self.handle_key_press(keycode, false);
    }
}

#[derive(StructOpt)]
struct Options {
    #[structopt(help = "The input image")]
    image: ArgImage
}

struct ArgImage(RgbaImage);

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);

        for cause in error.iter().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
    }
}

fn run() -> err::Result<()> {
    let opts = Options::from_args();

    let mut c = conf::Conf::new();
    c.window_mode.width = 1280;
    c.window_mode.height = 960;
    
    let mut ctx = Context::load_from_conf("super_simple", "ggez", c)?;
    let mut state = MainState::new(opts.image.0, &mut ctx)?;
    event::run(&mut ctx, &mut state)?;

    Ok(())
}

impl FromStr for ArgImage {
    type Err = image::ImageError;

    fn from_str(string: &str) -> std::result::Result<Self, Self::Err> {
        Ok(ArgImage(image::open(string)?.to_rgba()))
    }
}