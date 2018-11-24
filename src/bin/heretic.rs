extern crate failure;
extern crate image;
extern crate opengl_graphics;
extern crate piston_window;

extern crate rheretic;
use std::fs::File;
use std::io::BufReader;

use image::RgbaImage;
use opengl_graphics::GlGraphics;
use opengl_graphics::Texture;
use piston_window::*;

use failure::Error;

use rheretic::{Vid, Wad};

fn render(vid: &mut Vid) {
    vid.set_palette("PLAYPAL");
    vid.draw_raw_screen("TITLE");
}

fn main() -> Result<(), Error> {
    let mut window: PistonWindow = WindowSettings::new("Heretic", [640, 400])
        .exit_on_esc(true)
        .build()
        .expect("Failed to create window");
    let mut gl = GlGraphics::new(OpenGL::V3_2);

    let file = BufReader::new(File::open("heretic1.wad")?);
    let wad = Wad::from_reader(file)?;

    let mut fb = RgbaImage::from_raw(320, 200, vec![0u8; 320 * 200 * 4]).unwrap();
    let mut fb_tex = Texture::from_image(&fb, &TextureSettings::new());

    let x = wad.cache_lump_name("PLAYPAL").unwrap();
    println!("{:?}", &x[..48]);
    let x = wad.lump("TITLE").unwrap();
    println!("{} {:x} {:x} {:?}", x.name, x.pos, x.len, &x.data[..320]);
    while let Some(e) = window.next() {
        if let Some(ref args) = e.render_args() {
            render(&mut Vid::new(&wad, &mut fb));
            let ref c = Context::new_abs(args.width as f64, args.height as f64);
            fb_tex.update(&fb);
            gl.draw(args.viewport(), |_, gl| {
                Image::new().draw(
                    &fb_tex,
                    &DrawState::default(),
                    c.scale(2.0, 2.0).transform,
                    gl,
                );
            });
        }

        if let Some(ref _args) = e.update_args() {
            //
        }

        if let Some(ref _args) = e.press_args() {
            //
        }
    }

    Ok(())
}
