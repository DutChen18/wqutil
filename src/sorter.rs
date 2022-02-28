use std::{sync::Mutex, collections::HashMap, mem};
use image::{ImageBuffer, Rgb, Pixel};
use crate::config;

fn delta(a: &ImageBuffer<Rgb<f32>, Vec<f32>>, b: &ImageBuffer<Rgb<f32>, Vec<f32>>) -> f32 {
    let mut count = 0;
    let sum = a.pixels().zip(b.pixels())
        .map(|(a, b)| {
            a.channels().iter().zip(b.channels())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>()
        })
        .filter(|&d| d < config::MAX_GRADIENT * 3f32)
        .inspect(|_| count += 1)
        .map(|mut d| {
            for _ in 0..config::SQRT_COUNT {
                d = d.sqrt();
            }
            d
        })
        .sum::<f32>();

    let confidence = count as f32 / a.height() as f32;
    if count == 0 || confidence < config::MIN_CONFIDENCE {
        f32::MAX / 2.0
    } else {
        sum / (count as f32).powf(config::CONFIDENCE_BONUS)
    }
}

pub fn find_deltas(images: &[ImageBuffer<Rgb<f32>, Vec<f32>>], indices: &[usize]) -> Vec<(usize, usize, f32)> {
    let pairs = Mutex::new(Vec::new());
    let deltas = Mutex::new(Vec::new());

    {
        let mut pairs = pairs.lock().unwrap();
        for i in 0..indices.len() {
            for j in i+1..indices.len() {
                pairs.push((indices[i], indices[j]));
            }
        }
    }

    crossbeam::scope(|s| {
        for _ in 0..config::SORT_THREADS {
            s.spawn(|_| {
                while let Some(pair) = {
                    let mut pairs = pairs.lock().unwrap();
                    pairs.pop()
                } {
                    let delta = delta(&images[pair.0], &images[pair.1]);
                    deltas.lock().unwrap().push((pair.0, pair.1, delta));
                }
            });
        }
    }).unwrap();

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