use std::fs::File;
use std::io::BufReader;

use ::image::RgbaImage;
use opengl_graphics::GlGraphics;
use opengl_graphics::Texture;
use piston_window::{
    Context, DrawState, Image, OpenGL, PistonWindow, PressEvent, RenderEvent, TextureSettings,
    Transformed, UpdateEvent, WindowSettings,
};

use engine::Wad;
use failure::Error;
use specs::prelude::*;
use specs_derive::Component;

const SECONDS_PER_TICK: f64 = 1.0 / 35.0;

mod menu;
mod map;

#[derive(Component)]
struct Sprite {
    patch: &'static str,
    x: u32,
    y: u32,
}

fn main() -> Result<(), Error> {
    let mut window: PistonWindow = WindowSettings::new(
        "Heretic",
        [engine::SCREEN_WIDTH * 2, engine::SCREEN_HEIGHT * 2],
    )
    .exit_on_esc(true)
    .build()
    .expect("Failed to create window");
    let mut gl = GlGraphics::new(OpenGL::V3_2);

    let mut world = World::new();
    menu::add_components(&mut world);
    map::add_components(&mut world);
    world.register::<Sprite>();

    let file = BufReader::new(File::open("heretic.wad")?);
    world.insert(Wad::from_reader(file)?);
    world.insert(
        RgbaImage::from_raw(
            engine::SCREEN_WIDTH,
            engine::SCREEN_HEIGHT,
            vec![0u8; (engine::SCREEN_WIDTH * engine::SCREEN_HEIGHT) as usize * 4],
        )
        .unwrap(),
    );

    menu::add_resources(&mut world);
    menu::add_entities(&mut world);

    let mut tick_delta = 0.0;

    while let Some(e) = window.next() {
        if let Some(ref args) = e.render_args() {
            menu::render(&world);

            // Scale and mirror FB to window.
            let fb: (ReadExpect<RgbaImage>) = world.system_data();
            let ref c = Context::new_abs(args.draw_size[0] as f64, args.draw_size[1] as f64);
            let fb_tex = Texture::from_image(&fb, &TextureSettings::new());
            gl.draw(args.viewport(), |_, gl| {
                Image::new().draw(
                    &fb_tex,
                    &DrawState::default(),
                    c.scale(2.0, 2.0).transform,
                    gl,
                );
            });
        }

        if let Some(ref args) = e.update_args() {
            tick_delta += args.dt;
            while tick_delta > SECONDS_PER_TICK {
                menu::tick(&world);
                tick_delta -= SECONDS_PER_TICK;
            }
            world.maintain();
        }

        if let Some(_button) = e.press_args() {
            //
        }
    }

    Ok(())
}
