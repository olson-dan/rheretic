use std::fs::File;
use std::io::BufReader;

use ::image::RgbaImage;
use opengl_graphics::GlGraphics;
use opengl_graphics::Texture;
use piston_window::{
    Context, DrawState, Image, OpenGL, PistonWindow, PressEvent, RenderEvent, TextureSettings,
    Transformed, UpdateEvent, WindowSettings,
};

use failure::Error;

use rheretic::{Vid, Wad};

use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
struct Background {
    patch: &'static str,
}

#[derive(Component)]
struct UISprite {
    patch: &'static str,
    x: u32,
    y: u32,
}

struct RenderBackgrounds;
struct RenderUISprites;

impl<'a> System<'a> for RenderBackgrounds {
    type SystemData = (
        ReadExpect<'a, Wad>,
        WriteExpect<'a, RgbaImage>,
        ReadStorage<'a, Background>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (wad, mut fb, backgrounds) = data;
        let mut vid = Vid::new(&wad, &mut fb);
        vid.set_palette("PLAYPAL");
        for background in backgrounds.join() {
            vid.draw_raw_screen(background.patch);
        }
    }
}

impl<'a> System<'a> for RenderUISprites {
    type SystemData = (
        ReadExpect<'a, Wad>,
        WriteExpect<'a, RgbaImage>,
        ReadStorage<'a, UISprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (wad, mut fb, sprites) = data;
        let mut vid = Vid::new(&wad, &mut fb);
        vid.set_palette("PLAYPAL");
        for s in sprites.join() {
            vid.draw_patch(s.x, s.y, s.patch);
        }
    }
}

fn main() -> Result<(), Error> {
    let mut window: PistonWindow = WindowSettings::new(
        "Heretic",
        [rheretic::SCREEN_WIDTH * 2, rheretic::SCREEN_HEIGHT * 2],
    )
    .exit_on_esc(true)
    .build()
    .expect("Failed to create window");
    let mut gl = GlGraphics::new(OpenGL::V3_2);

    let mut world = World::new();
    world.register::<Background>();
    world.register::<UISprite>();

    let file = BufReader::new(File::open("heretic.wad")?);
    world.insert(Wad::from_reader(file)?);
    world.insert(
        RgbaImage::from_raw(
            rheretic::SCREEN_WIDTH,
            rheretic::SCREEN_HEIGHT,
            vec![0u8; (rheretic::SCREEN_WIDTH * rheretic::SCREEN_HEIGHT) as usize * 4],
        )
        .unwrap(),
    );

    world
        .create_entity()
        .with(Background { patch: "TITLE" })
        .with(UISprite {
            x: 4,
            y: 160,
            patch: "ADVISOR",
        })
        .build();

    let mut render_background = RenderBackgrounds;
    let mut render_ui_sprites = RenderUISprites;

    while let Some(e) = window.next() {
        if let Some(ref args) = e.render_args() {
            render_background.run_now(&world);
            render_ui_sprites.run_now(&world);

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

        if let Some(ref _args) = e.update_args() {
            world.maintain();
        }

        if let Some(ref _args) = e.press_args() {
            //
        }
    }

    Ok(())
}
