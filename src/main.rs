use std::{env::args, path::Path};

fn main() {
    let mut args = args().into_iter();
    args.next();

    let mut root_path = String::new();

    for arg in args {
        if !root_path.is_empty() {
            root_path.push(' ');
        }
        root_path += &arg;
    }

    println!("root = {:?}", root_path);

    let root_path = Path::new(&root_path);

    let skip = root_path.iter().count();

    let all_file_path = vocal_sample_pack_fixer::get_and_prepare_all_file_path(root_path, skip)
        .expect(format!("Doesn't find any file in {:?}", root_path).as_str());

    println!("{:?}", all_file_path);

    for path in all_file_path {
        match vocal_sample_pack_fixer::process_file(path, root_path) {
            Ok(_) => {}
            Err(e) => panic!("{e}")
        }
        println!("");
    }
}
