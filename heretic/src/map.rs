use ::image::RgbaImage;
use specs::prelude::*;
use specs_derive::Component;

use engine::{Patch, Vid, Wad};

#[derive(Component)]
struct BlockMap {
    orig_x: f32,
    orig_y: f32,
    width: u32,
    height: u32,
    map: Vec<u8>,
}

impl BlockMap {
    fn from_lump(mut data: &[u8]) {

    }
}

pub fn add_components(world: &mut World) {
    world.register::<BlockMap>();
}

pub fn load(world: &mut World, name: &str) {
    let wad: ReadExpect<Wad> = world.system_data();

    let base = wad.get_num_for_name(name).expect("Couldn't load map");
}
