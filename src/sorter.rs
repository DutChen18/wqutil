use std::{sync::{Arc, Mutex}, collections::HashMap, mem};
use image::{ImageBuffer, Rgb};

const THREADS: usize = 6;

const MAX_GRADIENT: f32 = 1.0;
const CONFIDENCE_BONUS: f32 = 1.0;
const MIN_CONFIDENCE: f32 = 0.0;
const SQRT_COUNT: u32 = 4;

fn delta(a: &ImageBuffer<Rgb<f32>, Vec<f32>>, b: &ImageBuffer<Rgb<f32>, Vec<f32>>) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;

    for (pixel, other) in a.pixels().zip(b.pixels()) {
        let mut tmp = 0.0;
        for c in 0..3 {
            tmp += (pixel[c] - other[c]).powi(2);
        }
        let mut tmp = tmp / 3.0;
        if tmp < MAX_GRADIENT.powi(2) {
            for _ in 0..SQRT_COUNT {
                tmp = tmp.sqrt();
            }
            sum += tmp;
            count += 1;
        }
    }

    if (count as f32 / a.height() as f32) < MIN_CONFIDENCE {
        return std::f32::MAX / 2.0;
    }
    if count == 0 {
        return std::f32::MAX / 2.0;
    }

    sum / (count as f32).powf(CONFIDENCE_BONUS)
}

pub fn find_deltas(images: &Arc<Vec<ImageBuffer<Rgb<f32>, Vec<f32>>>>, indices: &[usize]) -> Vec<(usize, usize, f32)> {
    let pairs = Arc::new(Mutex::new(Vec::new()));
    let deltas = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    {
        let mut pairs = pairs.lock().unwrap();
        for i in 0..indices.len() {
            for j in i+1..indices.len() {
                pairs.push((indices[i], indices[j]));
            }
        }
    }

    for _ in 0..THREADS {
        let images = images.clone();
        let deltas = deltas.clone();
        let pairs = pairs.clone();

        handles.push(std::thread::spawn(move || {
            while let Some(pair) = {
                let mut pairs = pairs.lock().unwrap();
                pairs.pop()
            } {
                let delta = delta(&images[pair.0], &images[pair.1]);
                deltas.lock().unwrap().push((pair.0, pair.1, delta));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut deltas = deltas.lock().unwrap();
    mem::take(&mut deltas)
}

pub fn sort(indices: &[usize], deltas: &[(usize, usize, f32)]) -> Vec<usize> {
    let mut chunks = indices.iter().map(|&i| vec![i]).collect::<Vec<_>>();
    let mut table = HashMap::new();
    let mut index = 0;

    for (i, v) in indices.iter().enumerate() {
        table.insert(*v, (i, 0));
    }

    for (i, j, _) in deltas {
        let a = *table.get(i).unwrap();
        let b = *table.get(j).unwrap();
        if a.0 == b.0 {
            continue;
        }
        if a.1 != 0 && a.1 != chunks[a.0].len() - 1 {
            continue;
        }
        if b.1 != 0 && b.1 != chunks[b.0].len() - 1 {
            continue;
        }

        let mut l = mem::take(&mut chunks[a.0]);
        let mut r = mem::take(&mut chunks[b.0]);
        if a.1 == 0 {
            l.reverse();
        }
        if b.1 != 0 {
            r.reverse();
        }
        l.append(&mut r);
        for (k, v) in l.iter().enumerate() {
            table.insert(*v, (a.0, k));
        }
        chunks[a.0] = l;
        index = a.0;
    }

    mem::take(&mut chunks[index])
}