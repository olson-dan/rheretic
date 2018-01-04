extern crate rheretic;
use std::fs::File;
use std::io::prelude::*;
use std::process::exit;

use rheretic::Wad;

fn main() {
    let mut file = File::open("heretic1.wad").unwrap();
    let mut s: Vec<u8> = Vec::new();
    if let Err(e) = file.read_to_end(&mut s) {
        println!("error: {}", e);
        exit(1);
    }

    let wad = Wad::from_slice(&s);

    for (name, slice) in &wad.lumps {
        println!("{} {}", name, slice.len());
    }

    println!("It Works!");
}