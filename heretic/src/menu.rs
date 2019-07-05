use ::image::RgbaImage;
use specs::prelude::*;
use specs_derive::Component;

use engine::{Vid, Wad};

#[derive(PartialEq)]
enum Menu {
    None,
    Main,
}

#[derive(Component)]
struct MenuIdent(Menu);

use super::Background;
use super::Sprite;

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

pub fn add_components(world: &mut World) {
    world.register::<MenuIdent>();
}

pub fn add_resources(world: &mut World) {
    world.insert(Menu::Main);
}

pub fn add_entities(world: &mut World) {
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
}

pub fn render(world: &World) {
    let mut render_menus = RenderMenus;
    render_menus.run_now(&world);
}
