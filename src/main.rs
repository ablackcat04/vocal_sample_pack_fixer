use std::{env::args, path::{Path, PathBuf}, thread};

fn split_path(all_path: Vec<PathBuf>) -> Vec<Vec<PathBuf>> {
    let num_cores = num_cpus::get();
    println!("Number of CPU cores: {}", num_cores);
    let total_tasks = all_path.iter().count();
    let num_thread_spawning = if num_cores * 2 < total_tasks {
        num_cores * 2
    } else {
        total_tasks
    };    // I/O intensive program, spawn more threads
    
    let mut v: Vec<Vec<PathBuf>> = Vec::new();
    for _ in 0..num_thread_spawning {
        v.push(Vec::new());
    }
    
    for (mut thread, path) in all_path.into_iter().enumerate() {
        thread %= num_thread_spawning;
        v[thread].push(path);
    }
    
    v
}

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

    println!("root = {:?}", root_path);

    let root_path = Path::new(&root_path);

    let skip = root_path.iter().count();

    let all_file_path = vocal_sample_pack_fixer::get_and_prepare_all_file_path(root_path, skip)
        .expect(format!("Doesn't find any file in {:?}", root_path).as_str());

    println!("{:?}", all_file_path);

    let splited_path: Vec<Vec<PathBuf>> = split_path(all_file_path);

    println!("{:#?}", splited_path);

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
