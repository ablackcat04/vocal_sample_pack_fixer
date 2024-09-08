use std::env::{self, args};

fn main() {
    println!("{:#?}", env::args());

    args().next();

    match args().next() {
        Some(root_folder) => {}
        None => {}
    }

    let p = "speedcore.wav";
    match vocal_sample_pack_fixer::process_file(p) {
        Ok(_) => {}
        Err(e) => panic!("{e}")
    }
}
