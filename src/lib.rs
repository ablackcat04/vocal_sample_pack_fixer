use core::f32;
use std::{fs, io, path::{Path, PathBuf}};

use hound;
extern crate queues;
use queues::*;

const ACTIVE_RATE: f32 = 0.015;
const ACTIVE_RATIO: f32 = 0.15;
const SAMPLES_PER_BLOCK: usize = 400;

fn find_first_valid(peak: f32, v: &Vec<f32>) -> usize {
    v.iter().enumerate().filter(|(_, x)| **x > peak*ACTIVE_RATE).take(1)
        .next().expect("This should work since it's the first and only call of next()").0
}

pub fn get_and_prepare_all_file_path(root: &Path, skip: usize) -> io::Result<Vec<PathBuf>> {
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


pub fn process_file(path: PathBuf, root: &Path) -> Result<(), String> {
    println!("Processing {:?}", path);
    let mut reader = hound::WavReader::open(&path)
        .map_err(|_| format!("Failed to open {:?}, maybe the file name is incorrect", path))?;

    let samples = reader.samples::<f32>();

    let samples: Vec<f32> = samples
        .map(|x| x.map_err(|_| format!("The wave file is broken at {:?}", path))) // Propagate the error with a custom message
        .collect::<Result<_, _>>()?;

    let peak = *samples.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap_or(&1.0);

    println!("peak = {}", peak);

    let first = find_first_valid(peak, &samples);

    let mut q: Queue<usize> = queue![];
    q.add(first).expect("Adding an element to a quene should be fine");

    let mut last = first;
    let mut start;

    loop {
        start = q.remove().expect("This is fine");
        let end = if start + SAMPLES_PER_BLOCK < samples.len() {
            start + SAMPLES_PER_BLOCK
        } else {
            samples.len()
        };
        let sample_slice = &samples[start .. end];

        let mut active_counter = 0;

        for x in sample_slice.iter().rev() {
            if *x > peak*ACTIVE_RATE {
                let f = find_first_valid(peak, &samples);
                if f > last {
                    q.add(f).expect("Adding an element to a quene should be fine");
                    last = f;
                }
                active_counter += 1;
            }
        }

        if active_counter as f32 > SAMPLES_PER_BLOCK as f32 * ACTIVE_RATIO {
            break;
        }

        if q.peek().is_err() {
            let a = samples[end .. samples.len()].iter().enumerate().map(|(count, x)| (count, *x)).filter(|(_, x)| *x > peak*ACTIVE_RATE).take(1).next();
            if let Some((a, _)) = a {
                q.add(a + end).expect("Adding an element to a quene should be fine");
            }
        }
    }

    // make sure that the channels are not flipped
    let start_sample_channel = start % reader.spec().channels as usize;
    start -= start_sample_channel;
    
    println!("start at {}s", start as f32 / 48000.0 / reader.spec().channels as f32);

    let skip = root.iter().count();

    let mut output_path = PathBuf::from("./outputs");
    output_path.extend(path.iter().skip(skip));

    println!("output path = {:?}", output_path);

    let mut writer = hound::WavWriter::create(output_path, reader.spec().clone())
        .map_err(|e| format!("{e}"))?;

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

    Ok(())
}
