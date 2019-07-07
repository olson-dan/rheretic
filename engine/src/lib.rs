#[macro_use]
extern crate failure;
extern crate byteorder;
extern crate image;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;
use image::{Rgba, RgbaImage};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str;

pub const SCREEN_WIDTH: u32 = 320;
pub const SCREEN_HEIGHT: u32 = 200;

pub struct Lump {
    pub name: String,
    pub data: Vec<u8>,
    pub pos: u64,
    pub len: usize,
}

pub struct Wad {
    pub lumps: Vec<Lump>,
}

impl Wad {
    pub fn from_reader<T>(mut data: T) -> Result<Wad, Error>
    where
        T: Read + Seek,
    {
        let mut tag = [0u8; 4];
        data.read_exact(&mut tag)?;
        let tag = str::from_utf8(&tag)?;
        if tag != "IWAD" && tag != "PWAD" {
            bail!(format!("Invalid WAD magic"));
        }
        let num_lumps = data.read_u32::<LittleEndian>()? as usize;
        let offset = data.read_u32::<LittleEndian>()? as u64;

        data.seek(SeekFrom::Start(offset))?;

        let mut lumps: Vec<Lump> = Vec::with_capacity(num_lumps);
        for _ in 0..num_lumps {
            let pos = data.read_u32::<LittleEndian>()? as u64;
            let len = data.read_u32::<LittleEndian>()? as usize;
            let mut name = [0u8; 8];
            data.read_exact(&mut name)?;
            let name = str::from_utf8(&name)?.trim_end_matches('\0').to_string();
            lumps.push(Lump {
                name,
                pos,
                len,
                data: Vec::new(),
            });
        }
        for lump in lumps.iter_mut() {
            data.seek(SeekFrom::Start(lump.pos))?;
            let mut contents = vec![0u8; lump.len];
            data.read_exact(&mut contents)?;
            lump.data = contents;
        }

        Ok(Wad { lumps })
    }

    pub fn get_num_for_name(&self, name: &str) -> Option<usize> {
        for (i, lump) in self.lumps.iter().enumerate() {
            if lump.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn cache_lump_num(&self, num: usize) -> Option<&[u8]> {
        if num < self.lumps.len() {
            return Some(&self.lumps[num].data);
        }
        None
    }

    pub fn cache_lump_name(&self, name: &str) -> Option<&[u8]> {
        if let Some(lump) = self.lumps.iter().rev().find(|l| l.name == name) {
            Some(&lump.data)
        } else {
            None
        }
    }
}

pub struct Patch {
    pub w: u32,
    pub h: u32,
    pub left: i32,
    pub top: i32,
}

impl Patch {
    pub fn from_lump(mut data: &[u8]) -> Patch {
        let w = data.read_u16::<LittleEndian>().unwrap() as u32;
        let h = data.read_u16::<LittleEndian>().unwrap() as u32;
        let left = data.read_i16::<LittleEndian>().unwrap() as i32;
        let top = data.read_i16::<LittleEndian>().unwrap() as i32;
        Patch { w, h, left, top }
    }
}

pub struct Vid<'a> {
    wad: &'a Wad,
    fb: &'a mut RgbaImage,
    palette: Option<Vec<Rgba<u8>>>,
}

impl<'a> Vid<'a> {
    pub fn new(wad: &'a Wad, fb: &'a mut RgbaImage) -> Vid<'a> {
        Vid {
            wad,
            fb,
            palette: None,
        }
    }

    fn blit_raw(&mut self, data: &[u8], w: u32, h: u32) {
        let palette = self.palette.as_ref().unwrap();
        for (x, y, dest) in self.fb.enumerate_pixels_mut() {
            if y >= h || x >= w {
                continue;
            }
            let coord = (y * w + x) as usize;
            let pixel = palette[data[coord] as usize];
            *dest = pixel;
        }
    }

    fn blit_column(&mut self, data: &[u8], x: u32, y: u32) {
        let palette = self.palette.as_ref().unwrap();
        for (i, p) in data.iter().enumerate() {
            let pixel = palette[*p as usize];
            self.fb[(x, y + i as u32)] = pixel;
        }
    }

    pub fn draw_patch_raw(&mut self, mut data: &[u8], x: u32, y: u32) {
        let img = &data[..];

        let w = data.read_u16::<LittleEndian>().unwrap() as u32;
        let h = data.read_u16::<LittleEndian>().unwrap() as u32;
        let left = data.read_i16::<LittleEndian>().unwrap() as i32;
        let top = data.read_i16::<LittleEndian>().unwrap() as i32;

        let x = (x as i32 - left) as u32;
        let y = (y as i32 - top) as u32;

        if (x + w) > SCREEN_WIDTH || (y + h) > SCREEN_HEIGHT {
            panic!("Bad V_DrawPatch");
        }

        for x_ofs in 0..(w as usize) {
            let mut col_ofs = &data[4 * x_ofs..];
            let mut col_ofs = col_ofs.read_u32::<LittleEndian>().unwrap() as usize;

            let dest_x = x + x_ofs as u32;
            loop {
                let topdelta = img[col_ofs] as u32;
                if topdelta == 255 {
                    break;
                }
                let length = img[col_ofs + 1] as usize;;
                let source0 = col_ofs + 3;
                let source1 = source0 + length;
                let dest_y = topdelta + y;
                self.blit_column(&img[source0..source1], dest_x, dest_y);
                col_ofs += length + 4;
            }
        }
    }

    pub fn draw_raw_screen(&mut self, lump: &str) {
        if let Some(lump) = self.wad.cache_lump_name(lump) {
            self.blit_raw(lump, SCREEN_WIDTH, SCREEN_HEIGHT);
        }
    }

    pub fn draw_patch(&mut self, x: u32, y: u32, lump: &str) {
        if let Some(lump) = self.wad.cache_lump_name(lump) {
            self.draw_patch_raw(lump, x, y);
        }
    }

    pub fn set_palette(&mut self, lump: &str) {
        if let Some(lump) = self.wad.cache_lump_name(lump) {
            self.palette = Some(
                lump.chunks(3)
                    .map(|x| Rgba {
                        data: [x[0], x[1], x[2], 255],
                    })
                    .collect(),
            )
        }
    }
}
