use core::f32;
use std::{fs, io, path::{Path, PathBuf}};

use hound;

const ACTIVE_RATE: f32 = 0.015;
const ACTIVE_RATIO: f32 = 0.15;
const SAMPLES_PER_BLOCK: usize = 400;

fn find_first_valid(peak: f32, v: &Vec<f32>) -> usize {
    v.iter().enumerate().filter(|(_, x)| **x > peak*ACTIVE_RATE).take(1)
        .next().expect("This should work since it's the first and only call of next()").0
}

pub fn split_task<T>(all_path: Vec<T>) -> Vec<Vec<T>> {
    let num_cores = num_cpus::get();
    println!("Number of CPU cores: {}", num_cores);
    let total_tasks = all_path.iter().count();
    let num_thread_spawning = if num_cores * 2 < total_tasks {
        num_cores * 2
    } else {
        total_tasks
    };    // I/O intensive program, spawn more threads
    
    let mut v: Vec<Vec<T>> = Vec::new();
    for _ in 0..num_thread_spawning {
        v.push(Vec::new());
    }
    
    for (mut thread, path) in all_path.into_iter().enumerate() {
        thread %= num_thread_spawning;
        v[thread].push(path);
    }
    
    v
}


pub fn get_and_prepare_all_file_path(root: &Path, skip: usize) -> io::Result<Vec<PathBuf>> {
    let output_dir = Path::new("./outputs");

    if !output_dir.exists() {
        match fs::create_dir(&output_dir) {
            Ok(_) => {}
            Err(_) => {println!("Failed to create output directory {:?}", output_dir)}
        }
    }

    let mut entries = fs::read_dir(root)?
        .map(|res| res.map(|e| e.path()))
        .filter(|e| e.as_ref().unwrap().is_file())
        .collect::<Result<Vec<_>, io::Error>>()?;

    let sub_dirs = fs::read_dir(root)?
        .map(|res| res.map(|e| e.path()))
        .filter(|e| e.as_ref().unwrap().is_dir())
        .collect::<Result<Vec<_>, io::Error>>()?;

    for sub_dir in sub_dirs {
        let mut output_sub_dir = PathBuf::from("./outputs");
        output_sub_dir.extend(sub_dir.iter().skip(skip));

        if !output_sub_dir.exists() {
            match fs::create_dir(&output_sub_dir) {
                Ok(_) => {}
                Err(_) => {println!("Failed to create sub directory {:?}", output_sub_dir)}
            }
        }

        entries.append(&mut get_and_prepare_all_file_path(&sub_dir, skip).unwrap());
    }

    Ok(entries)
}

fn find_start(samples: &Vec<f32>, peak: f32, channels: u16) -> usize {
    let mut start = find_first_valid(peak, &samples);

    loop {
        let end = if start + SAMPLES_PER_BLOCK < samples.len() {
            start + SAMPLES_PER_BLOCK
        } else {
            samples.len()
        };
        let sample_slice = &samples[start .. end];

        let mut active_counter = 0;

        for x in sample_slice.iter() {
            if *x > peak*ACTIVE_RATE {
                active_counter += 1;
            }
        }

        if active_counter as f32 > SAMPLES_PER_BLOCK as f32 * ACTIVE_RATIO {
            break;
        }

        let a = samples[end .. samples.len()].iter().enumerate().
            map(|(count, x)| (count, *x)).
            filter(|(_, x)| *x > peak*ACTIVE_RATE).take(1).next();
        if let Some((a, _)) = a {
            start = a + end;
        }
    }

    // make sure that the channels are not flipped
    let start_sample_channel = start % channels as usize;
    start - start_sample_channel

}


pub fn process_file(path: PathBuf, skip: usize) -> Result<(), String> {
    // use this to log messeges at once, since this is a multi-thread program
    let mut infos: Vec<String> = Vec::new();
    infos.push(format!("Processing {:?}", path));

    let mut reader = hound::WavReader::open(&path)
        .map_err(|_| format!("Failed to open {:?}, maybe the file name is incorrect", path))?;

    let samples: Vec<f32> = reader.samples::<f32>()
        .map(|x| x.map_err(|_| format!("The wave file is broken at {:?}", path)))
        .collect::<Result<_, _>>()?;

    let peak = *samples.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap_or(&1.0);
    infos.push(format!("peak = {}", peak));

    let start = find_start(&samples, peak, reader.spec().channels);
    infos.push(format!("start at {}s", start as f32 / 48000.0 / reader.spec().channels as f32));

    let mut output_path = PathBuf::from("./outputs");
    output_path.extend(path.iter().skip(skip)); // add relative path after outputs
    infos.push(format!("output path = {:?}", output_path));

    let mut writer = hound::WavWriter::create(output_path, reader.spec().clone())
        .map_err(|e| format!("{e}"))?;

    // little exponential fade in
    let (fade_start, fade_len) = if start < 400 {
        (0, start)
    } else {
        (start - 400, 400)
    };
    let num = f32::consts::E;
    for (count, &sample) in samples[fade_start .. start].iter().enumerate() {
        let norm_count = (count + 1) as f32 / (fade_len as f32);
        let log_scale = (norm_count * num + 1e-5).ln() / (num + 1e-5).ln();
        writer.write_sample(sample / peak * 0.9 * log_scale).map_err(|e| format!("{e}"))?;
    }

    for &sample in samples[start .. samples.len()].iter() {
        writer.write_sample(sample / peak * 0.9).map_err(|e| format!("{e}"))?;
    }

    writer.finalize().map_err(|e| format!("{e}"))?;

    infos.push(String::from(""));
    for info in infos {
        println!("{info}");
    }
    Ok(())
}
