use ::image::RgbaImage;
use specs::prelude::*;
use specs_derive::Component;

use engine::{Vid, Wad};

#[derive(PartialEq)]
enum Menu {
    None,
    Main,
    Episode,
    Skill,
    Options,
    Options2,
    Files,
    Load,
    Save,
}

#[derive(Component)]
struct MenuIdent(Menu);

#[derive(Component)]
struct MenuOffset(u32, u32);

enum MenuAction {
    None,
    SetMenu(Menu),
    NetCheck(u32, Menu),
    Info,
    QuitGame,
    Episode(u32),
    LoadGame(u32),
    SaveGame(u32),
    Skill(u32),
    EndGame,
    Messages,
    MouseSensitivity,
    ScreenSize,
    SfxVolume,
    MusicVolume,
}

struct MenuBackground {
    patch: &'static str,
}

#[derive(Component)]
struct MenuItem {
    text: &'static str,
    action: MenuAction,
}

struct MenuSelection(Option<u32>);

use super::Sprite;

struct RenderMenus;

impl<'a> System<'a> for RenderMenus {
    type SystemData = (
        ReadExpect<'a, Wad>,
        WriteExpect<'a, RgbaImage>,
        ReadExpect<'a, Menu>,
        ReadExpect<'a, MenuBackground>,
        ReadExpect<'a, MenuSelection>,
        ReadStorage<'a, MenuIdent>,
        ReadStorage<'a, MenuOffset>,
        ReadStorage<'a, MenuItem>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (wad, mut fb, menu, bg, sel, ids, offsets, items, sprites) = data;
        let mut vid = Vid::new(&wad, &mut fb);
        let menu: &Menu = &menu;
        vid.set_palette("PLAYPAL");
        if !bg.patch.is_empty() {
            vid.draw_raw_screen(bg.patch);
        }

        let (x, y) = {
            if let Some((_, offset)) = (&ids, &offsets).join().next() {
                (offset.0, offset.1)
            } else {
                (0, 0)
            }
        };

        match *menu {
            Menu::Main => {
                vid.draw_patch(88, 0, "M_HTIC");
            }
            _ => {}
        }

        for (id, s) in (&ids, &sprites).join() {
            if id.0 == *menu {
                vid.draw_patch(s.x, s.y, s.patch);
            }
        }
    }
}

pub fn add_components(world: &mut World) {
    world.register::<MenuIdent>();
    world.register::<MenuOffset>();
    world.register::<MenuItem>();
}

pub fn add_resources(world: &mut World) {
    world.insert(Menu::Main);
    world.insert(MenuBackground { patch: "TITLE" });
    world.insert(MenuSelection(None));
}

pub fn add_entities(world: &mut World) {
    let main_items = [
        ("NEW GAME", MenuAction::NetCheck(1, Menu::Episode)),
        ("OPTIONS", MenuAction::SetMenu(Menu::Options)),
        ("GAME FILES", MenuAction::SetMenu(Menu::Files)),
        ("INFO", MenuAction::Info),
        ("QUIT GAME", MenuAction::QuitGame),
    ];

    let episode_items = [
        ("CITY OF THE DAMNED", MenuAction::Episode(1)),
        ("HELL's MAW", MenuAction::Episode(2)),
        ("THE DOME OF D'SPARIL", MenuAction::Episode(3)),
        ("THE OSSUARY", MenuAction::Episode(4)),
        ("THE STAGNANT DEMESNE", MenuAction::Episode(5)),
    ];

    let files_items = [
        ("LOAD GAME", MenuAction::NetCheck(2, Menu::Load)),
        ("SAVE GAME", MenuAction::SetMenu(Menu::Save)),
    ];

    let skill_items = [
        ("THOU NEEDETH A WET-NURSE", MenuAction::Skill(0)),
        ("YELLOWBELLIES-R-US", MenuAction::Skill(1)),
        ("BRINGEST THEM ONETH", MenuAction::Skill(2)),
        ("THOU ART A SMITE-MEISTER", MenuAction::Skill(3)),
        ("BLACK PLAGUE POSSESSES THEE", MenuAction::Skill(4)),
    ];

    let options_items = [
        ("END GAME", MenuAction::EndGame),
        ("MESSAGES : ", MenuAction::Messages),
        ("MOUSE SENSITIVITY", MenuAction::MouseSensitivity),
        ("", MenuAction::None),
        ("MORE...", MenuAction::SetMenu(Menu::Options2)),
    ];

    let options2_items = [
        ("SCREEN SIZE", MenuAction::ScreenSize),
        ("", MenuAction::None),
        ("SFX VOLUME", MenuAction::SfxVolume),
        ("", MenuAction::None),
        ("MUSIC VOLUME", MenuAction::MusicVolume),
    ];

    /*
    world
        .create_entity()
        .with(MenuIdent(Menu::Main))
        .with(Sprite {
            x: 4,
            y: 160,
            patch: "ADVISOR",
        })
        .build();
    */
}

pub fn render(world: &World) {
    let mut render_menus = RenderMenus;
    render_menus.run_now(&world);
}
