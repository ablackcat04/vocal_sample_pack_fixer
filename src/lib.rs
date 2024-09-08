use core::f32;

use hound;
extern crate queues;
use queues::*;

const ACTIVE_RATE: f32 = 0.015;
const ACTIVE_RATIO: f32 = 0.15;
const SAMPLES_PER_BLOCK: usize = 400;
const TEMP_ROOT_FOLDER_FOR_SAMPLES :&str = "samples/";

fn find_first_valid(peak: f32, v: &Vec<f32>) -> usize {
    v.iter().enumerate().filter(|(_, x)| **x > peak*ACTIVE_RATE).take(1)
        .next().expect("This should work since it's the first and only call of next()").0
}

pub fn process_file(path: &str) -> Result<(), String> {
    let in_path = String::from(TEMP_ROOT_FOLDER_FOR_SAMPLES) + path;
    println!("Processing {}", in_path);
    let mut reader = hound::WavReader::open(&in_path)
        .map_err(|_| format!("Failed to open {in_path}, maybe the file name is incorrect"))?;

    let samples = reader.samples::<f32>();

    let samples: Vec<f32> = samples
        .map(|x| x.map_err(|_| format!("The wave file is broken at {in_path}"))) // Propagate the error with a custom message
        .collect::<Result<_, _>>()?;

    let peak = *samples.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap_or(&1.0);

    println!("peak = {}", peak);

    let first = find_first_valid(peak, &samples);

    let mut q: Queue<usize> = queue![];
    q.add(first).expect("Adding an element to a quene should be fine");

    let mut last = first;
    let mut start = 0;

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

    let mut writer = hound::WavWriter::create(String::from("outputs/") + path, reader.spec().clone())
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
