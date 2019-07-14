use std::convert::TryFrom;

use ::image::RgbaImage;
use specs::prelude::*;
use specs_derive::Component;

use engine::{Patch, Vid, Wad};

#[derive(PartialEq, Clone)]
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

#[derive(Clone)]
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

struct MenuBackground(Option<&'static str>);

#[derive(Component)]
struct MenuItem {
    text: &'static str,
    action: MenuAction,
}

struct MenuSelection(Option<u32>);
struct MenuTime(u32);

use super::Sprite;

struct TickMenus;
struct RenderMenus;

fn render_specific_menus(wad: &Wad, vid: &mut Vid, menu: &Menu, time: u32) {
    match *menu {
        Menu::Main => {
            let base = wad.get_num_for_name("M_SKL00").expect("missing M_SKL00");
            let frame = ((time / 3) % 18) as usize;
            vid.draw_patch(88, 0, "M_HTIC");
            vid.draw_patch_raw(wad.cache_lump_num(base + (17 - frame)).unwrap(), 40, 10);
            vid.draw_patch_raw(wad.cache_lump_num(base + frame).unwrap(), 232, 10);
        }
        _ => {}
    }
}

fn render_text(wad: &Wad, vid: &mut Vid, font: &str, text: &str, x: u32, y: u32) {
    let mut x = x;
    let base = wad.get_num_for_name(font).expect("Missing font") + 1;
    for c in text.chars() {
        if let Ok(c) = u32::try_from(c) {
            if c < 33 {
                x += 5;
            } else {
                let lump = wad.cache_lump_num(base + (c as usize) - 33).unwrap();
                vid.draw_patch_raw(lump, x, y);
                let patch = Patch::from_lump(lump);
                x += patch.w - 1;
            }
        }
    }
}

fn text_width(wad: &Wad, font: &str, text: &str) -> u32 {
    let mut w = 0;
    let base = wad.get_num_for_name(font).expect("Missing font") + 1;
    for c in text.chars() {
        if let Ok(c) = u32::try_from(c) {
            if c < 33 {
                w += 5;
            } else {
                let lump = wad.cache_lump_num(base + (c as usize) - 33).unwrap();
                let patch = Patch::from_lump(lump);
                w += patch.w - 1;
            }
        }
    }
    w
}

impl<'a> System<'a> for TickMenus {
    type SystemData = (ReadExpect<'a, Menu>, WriteExpect<'a, MenuTime>);

    fn run(&mut self, data: Self::SystemData) {
        let (menu, mut time) = data;
        let menu: &Menu = &menu;
        match *menu {
            Menu::None => {}
            _ => {
                time.0 += 1;
            }
        }
    }
}

impl<'a> System<'a> for RenderMenus {
    type SystemData = (
        ReadExpect<'a, Wad>,
        WriteExpect<'a, RgbaImage>,
        ReadExpect<'a, Menu>,
        ReadExpect<'a, MenuBackground>,
        ReadExpect<'a, MenuSelection>,
        ReadExpect<'a, MenuTime>,
        ReadStorage<'a, MenuIdent>,
        ReadStorage<'a, MenuOffset>,
        ReadStorage<'a, MenuItem>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (wad, mut fb, menu, bg, sel, time, ids, offsets, items, sprites) = data;
        let mut vid = Vid::new(&wad, &mut fb);
        let menu: &Menu = &menu;
        vid.set_palette("PLAYPAL");
        if let Some(patch) = bg.0 {
            vid.draw_raw_screen(patch);
        }

        if *menu == Menu::None {
            return;
        }

        let mut x = 0;
        let mut orig_y = 0;

        for (id, offset) in (&ids, &offsets).join() {
            if id.0 == *menu {
                x = offset.0;
                orig_y = offset.1;
                break;
            }
        }

        render_specific_menus(&wad, &mut vid, menu, time.0);

        let mut y = orig_y;
        for (id, item) in (&ids, &items).join() {
            if id.0 == *menu {
                if !item.text.is_empty() {
                    render_text(&wad, &mut vid, "FONTB_S", item.text, x, y);
                }
                y += 20;
            }
        }

        if let Some(item_num) = sel.0 {
            let y = orig_y + item_num * 20 - 1;
            if (time.0 & 16) != 0 {
                vid.draw_patch(x - 28, y, "M_SLCTR1");
            } else {
                vid.draw_patch(x - 28, y, "M_SLCTR2");
            }
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
    world.insert(MenuBackground(Some("TITLE")));
    world.insert(MenuSelection(Some(0)));
    world.insert(MenuTime(0));
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

    // Main
    world
        .create_entity()
        .with(MenuIdent(Menu::Main))
        .with(MenuOffset(110, 56))
        .build();
    for (text, action) in &main_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Main))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

    // Episode
    world
        .create_entity()
        .with(MenuIdent(Menu::Episode))
        .with(MenuOffset(80, 50))
        .build();
    for (text, action) in &episode_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Episode))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

    // Files
    world
        .create_entity()
        .with(MenuIdent(Menu::Files))
        .with(MenuOffset(110, 60))
        .build();
    for (text, action) in &files_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Files))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

    // Skill
    world
        .create_entity()
        .with(MenuIdent(Menu::Skill))
        .with(MenuOffset(38, 30))
        .build();
    for (text, action) in &skill_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Skill))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

    // Options
    world
        .create_entity()
        .with(MenuIdent(Menu::Options))
        .with(MenuOffset(88, 30))
        .build();
    for (text, action) in &options_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Options))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

    // Options2
    world
        .create_entity()
        .with(MenuIdent(Menu::Options2))
        .with(MenuOffset(90, 20))
        .build();
    for (text, action) in &options2_items {
        world
            .create_entity()
            .with(MenuIdent(Menu::Options2))
            .with(MenuItem {
                text: text,
                action: action.clone(),
            })
            .build();
    }

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

pub fn tick(world: &World) {
    let mut tick_menus = TickMenus;
    tick_menus.run_now(&world);
}

pub fn render(world: &World) {
    let mut render_menus = RenderMenus;
    render_menus.run_now(&world);
}
