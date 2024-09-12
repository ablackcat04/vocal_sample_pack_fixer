use std::{env::args, path::Path, thread};

fn main() {
    let start = std::time::Instant::now();

    let mut args = args().into_iter();
    args.next();

    let mut root_path = String::new();

    for arg in args {
        if !root_path.is_empty() {
            root_path.push(' ');
        }
        root_path += &arg;
    }

    let root_path = Path::new(&root_path);

    let skip = root_path.iter().count();

    let all_file_path = vocal_sample_pack_fixer::get_and_prepare_all_file_path(root_path, skip)
        .expect(format!("Doesn't find any file in {:?}", root_path).as_str());

    let splited_path = vocal_sample_pack_fixer::split_task(all_file_path);

    let mut handles = vec![];

    for paths in splited_path {
        let handle = thread::spawn(move || {
            for path in paths {
                match vocal_sample_pack_fixer::process_file(path, skip) {
                    Ok(_) => {}
                    Err(e) => panic!("{e}")
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
