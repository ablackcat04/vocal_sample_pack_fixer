use hound;
extern crate queues;
use queues::*;

const ACTIVE_RATE: f32 = 0.015;
const SAMPLES_PER_BLOCK: usize = 400;

fn main() {
    let mut reader = hound::WavReader::open("samples/speedcore.wav").unwrap();
    
    let sqr_sum = reader.samples::<f32>();

    let v: Vec<f32> = sqr_sum.map(|x| *x.as_ref().expect("The wave file is broken.")).collect();
    
    let peak = *v.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    println!("peak = {}", peak);

    let mut q: Queue<usize> = queue![];

    let first = v.iter().enumerate().map(|(count, x)| (count, *x)).filter(|(_, x)| *x > peak*ACTIVE_RATE).take(1).next().unwrap().0;

    q.add(first).expect("");

    let mut last = first;
    let mut start = 0;

    while q.peek().is_ok() {
        start = q.remove().unwrap();
        let end = if start + SAMPLES_PER_BLOCK < v.len() {
            start + SAMPLES_PER_BLOCK
        } else {
            v.len()
        };
        let slice = &v[start .. end];

        let mut c = 0;

        for x in slice.iter().rev() {
            // println!("{}: {x}", count + start);
            if *x > peak*ACTIVE_RATE {
                let f = v.iter().enumerate().map(|(count, x)| (count, *x)).filter(|(_, x)| *x > peak*ACTIVE_RATE).take(1).next().unwrap().0;
                if f > last {
                    q.add(f).unwrap();
                    last = f;
                }
                c += 1;
            }
        }

        if c as f32 > SAMPLES_PER_BLOCK as f32 * 0.15 {
            break;
        }

        if q.peek().is_err() {
            let a = v[end .. v.len()].iter().enumerate().map(|(count, x)| (count, *x)).filter(|(_, x)| *x > peak*ACTIVE_RATE).take(1).next();
            if let Some((a, _)) = a {
                q.add(a + end).unwrap();
            }
        }
    }

    if start % 2 == 1 {
        start -= 1;
    }

    println!("start at {}s", start as f32 / 48000.0 / reader.spec().channels as f32);


    let mut writer = hound::WavWriter::create("outputs/test_out.wav", reader.spec().clone()).unwrap();

    let fade_start = if start < 400 {
        0
    } else {
        start - 400
    };
    for (count, &fade_in) in v[fade_start .. start].iter().enumerate() {
        writer.write_sample(fade_in / peak * 0.9 * count as f32 / 400.0).unwrap();
    }

    for &sample in v[start .. v.len()].iter() {
        writer.write_sample(sample / peak * 0.9).unwrap();
    }

    writer.finalize().unwrap();
}
