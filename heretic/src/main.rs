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

use engine::{Vid, Wad};

use specs::prelude::*;
use specs_derive::Component;

#[derive(PartialEq)]
enum Menu {
    None,
    Main,
}

#[derive(Component)]
struct MenuIdent(Menu);

#[derive(Component)]
struct Background {
    patch: &'static str,
}

#[derive(Component)]
struct Sprite {
    patch: &'static str,
    x: u32,
    y: u32,
}

struct RenderMenus;

impl<'a> System<'a> for RenderMenus {
    type SystemData = (
        ReadExpect<'a, Wad>,
        WriteExpect<'a, RgbaImage>,
        ReadExpect<'a, Menu>,
        ReadStorage<'a, MenuIdent>,
        ReadStorage<'a, Background>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (wad, mut fb, menu, idents, backgrounds, sprites) = data;
        let mut vid = Vid::new(&wad, &mut fb);
        let menu: &Menu = &menu;
        vid.set_palette("PLAYPAL");
        for (id, background) in (&idents, &backgrounds).join() {
            if id.0 == *menu {
                vid.draw_raw_screen(background.patch);
            }
        }
        for (id, s) in (&idents, &sprites).join() {
            if id.0 == *menu {
                vid.draw_patch(s.x, s.y, s.patch);
            }
        }
    }
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
    world.register::<MenuIdent>();
    world.register::<Background>();
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
    world.insert(Menu::Main);

    world
        .create_entity()
        .with(MenuIdent(Menu::Main))
        .with(Background { patch: "TITLE" })
        .build();
    world
        .create_entity()
        .with(MenuIdent(Menu::Main))
        .with(Sprite {
            x: 4,
            y: 160,
            patch: "ADVISOR",
        })
        .build();

    let mut render_menus = RenderMenus;

    while let Some(e) = window.next() {
        if let Some(ref args) = e.render_args() {
            render_menus.run_now(&world);

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
