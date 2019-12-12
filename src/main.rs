use std::io::{Cursor, Read};

use pbo;

fn main() {
    let mut p = pbo::PBO::read(Cursor::new(
        std::fs::read(std::path::Path::new("synixe_training.pbo")).unwrap(),
    ))
    .unwrap();
    println!("{:?}", p.files(true));
    let mut c = p.retrieve("XEH_PREP.hpp").unwrap();
    let mut con = String::new();
    println!("c: {:?}", c.read_to_string(&mut con));
    println!("con: {:?}", con);
}
