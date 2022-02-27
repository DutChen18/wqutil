use std::{collections::HashSet};
use image::{ImageBuffer, Rgb};

pub fn cluster(images: &[ImageBuffer<Rgb<u8>, Vec<u8>>]) -> Vec<Vec<usize>> {
    let mut palettes = images.iter().enumerate()
        .map(|(i, image)| (i, image.pixels().collect::<HashSet<_>>()))
        .collect::<Vec<_>>();

    palettes.sort_by_cached_key(|&(_, ref palette)| palette.len());
    palettes.reverse();

    let mut clusters: Vec<(Vec<usize>, HashSet<&Rgb<u8>>)> = Vec::new();

    for (index, palette) in palettes {
        let mut done = false;

        for (cluster, target) in clusters.iter_mut() {
            if palette.is_subset(target) {
                cluster.push(index);
                done = true;
            }
        }
        
        if !done {
            clusters.push((vec![index], palette));
        }
    }

    clusters.into_iter().map(|(cluster, _)| cluster).collect()
}