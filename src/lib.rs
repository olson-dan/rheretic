
use std::collections::HashMap;

use std::str::from_utf8;

macro_rules! le32 {
    ($slice:expr, $offset:expr) => (
        {
            let a = $slice[$offset + 0] as u32;
            let b = $slice[$offset + 1] as u32;
            let c = $slice[$offset + 2] as u32;
            let d = $slice[$offset + 3] as u32;
            ((d << 24) | (c << 16) | (b << 8) | a) as usize
        }
    );
}

pub struct Wad<'a> {
    pub lumps: HashMap<String, &'a [u8]>,
}

impl<'a> Wad<'a> {
    pub fn from_slice(data: &'a [u8]) -> Wad {
        let mut lumps : HashMap<String, &'a [u8]> = HashMap::new();

        let tag = from_utf8(&data[..4]).expect("couldn't decode IWAD tag");
        if tag == "IWAD" || tag == "PWAD" {
            let num_lumps = le32!(data, 4);
            let mut offset = le32!(data, 8);
            for _ in 0..num_lumps {
                let pos = le32!(data, offset + 0);
                let length = le32!(data, offset + 4);
                let name = String::from(from_utf8(&data[offset + 8..offset+16]).unwrap().trim_right_matches('\0'));
                lumps.insert(name, &data[pos..pos+length]);
                offset += 16;
            }
        }
        Wad { lumps: lumps }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}