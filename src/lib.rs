#[macro_use]
extern crate failure;
extern crate byteorder;
extern crate image;

use byteorder::{LittleEndian, ReadBytesExt};
use failure::Error;
use image::{Rgb, Rgba, RgbaImage};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str;

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
            let name = str::from_utf8(&name)?.trim_right_matches('\0').to_string();
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

pub struct Vid<'a> {
    wad: &'a Wad,
    fb: &'a mut RgbaImage,
    palette: Option<Vec<Rgb<u8>>>,
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
            dest[0] = pixel[0];
            dest[1] = pixel[1];
            dest[2] = pixel[2];
            dest[3] = 255;
        }
    }

    fn blit_column(&mut self, data: &[u8], x: u32, y: u32) {
        let palette = self.palette.as_ref().unwrap();
        for (i, p) in data.iter().enumerate() {
            let pixel = palette[*p as usize];
            self.fb[(x, y + i as u32)] = Rgba {
                data: [pixel[0], pixel[1], pixel[2], 255],
            };
        }
    }

    fn blit_patch(&mut self, mut data: &[u8], x: u32, y: u32) {
        let img = &data[..];

        let w = data.read_u16::<LittleEndian>().unwrap() as usize;
        let _h = data.read_u16::<LittleEndian>().unwrap();
        let left = data.read_u16::<LittleEndian>().unwrap() as u32;
        let top = data.read_u16::<LittleEndian>().unwrap() as u32;

        let x = x - left;
        let y = y - top;

        for x_ofs in 0..w {
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
        let lump = self
            .wad
            .cache_lump_name(lump)
            .expect(&format!("Couldn't find lump {}", lump));
        self.blit_raw(lump, 320, 200);
    }

    pub fn draw_patch(&mut self, x: u32, y: u32, lump: &str) {
        let lump = self
            .wad
            .cache_lump_name(lump)
            .expect(&format!("Couldn't find lump {}", lump));
        self.blit_patch(lump, x, y);
    }

    pub fn set_palette(&mut self, lump: &str) {
        let lump = self
            .wad
            .cache_lump_name(lump)
            .expect(&format!("Couldn't find lump {}", lump));
        self.palette = Some(
            lump.chunks(3)
                .map(|x| Rgb {
                    data: [x[0], x[1], x[2]],
                }).collect(),
        )
    }
}
