use std::{time::SystemTime, fs, sync::Arc};
use image::{DynamicImage, ImageBuffer, Rgb};

mod cutter;
mod cluster;
mod sorter;

const DO_LOAD: bool = true;
const DO_CLUSTER: bool = true;
const RAW_PATH: &str = "raw_strips";
const CUT_PATH: &str = "cut_strips";
const PNG_PATH: &str = "result.png";
// const TXT_PATH: &str = "result.txt";
// const SCORES_PATH: &str = "scores.txt";

#[tokio::main]
async fn main() {
    if DO_LOAD {
        let time = SystemTime::now();
        let (total, new) = cutter::download_images(RAW_PATH).await;
        println!("{} images downloaded ({} total) in {:.3?}", new, total, time.elapsed().unwrap());
    
        let time = SystemTime::now();
        let (total, new) = cutter::cut_images(RAW_PATH, CUT_PATH);
        println!("{} images cut ({} total) in {:.3?}", new, total, time.elapsed().unwrap());
    }

    let time = SystemTime::now();
    let filenames = fs::read_dir(CUT_PATH).unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    let images8 = filenames.iter()
        .map(|entry| image::open(entry).unwrap())
        .map(|image| image.into_rgb8())
        .collect::<Vec<_>>();
    let images32f = Arc::new(images8.iter()
        .map(|image| DynamicImage::from(image.clone()))
        .map(|image| image.into_rgb32f())
        .collect::<Vec<_>>());
    println!("{} strips loaded in {:.3?}", images8.len(), time.elapsed().unwrap());

    let clusters = if DO_CLUSTER {
        let time = SystemTime::now();
        let clusters = cluster::cluster(&images8);
        println!("{} clusters found in {:.3?}", clusters.len(), time.elapsed().unwrap());
        clusters
    } else {
        vec![(0..images8.len()).collect::<Vec<_>>()]
    };

    // let mut scores = BufWriter::new(File::create(SCORES_PATH).unwrap());
    // for filename in filenames.iter() {
    //     writeln!(scores, "{}", filename.display()).unwrap();
    // }
    // writeln!(scores, "").unwrap();

    let sorted = clusters.iter()
        .map(|cluster| {
            let time = SystemTime::now();
            let deltas = sorter::find_deltas(&images32f, cluster);
            println!("{} deltas computed in {:?}", deltas.len(), time.elapsed().unwrap());

            let time = SystemTime::now();
            let mut deltas = deltas.iter().collect::<Vec<_>>();
            deltas.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
            println!("{} deltas sorted in {:?}", deltas.len(), time.elapsed().unwrap());

            // for ((i, j), delta) in deltas.iter() {
            //     writeln!(scores, "{} {} {}", i, j, delta).unwrap();
            // }
            // writeln!(scores, "").unwrap();

            let time = SystemTime::now();
            let sorted = sorter::sort(cluster, &deltas);
            println!("{} strips sorted in {:?}", sorted.len(), time.elapsed().unwrap());
            sorted
        })
        .collect::<Vec<_>>();

    let time = SystemTime::now();
    let width = sorted.len() + sorted.iter().map(Vec::len).sum::<usize>();
    let mut png = ImageBuffer::<Rgb<f32>, _>::new(width as u32, images8[0].height());
    // let mut txt = BufWriter::new(File::create(TXT_PATH).unwrap());
    let mut index = 0;

    for cluster in sorted.iter() {
        for &strip in cluster.iter() {
            // writeln!(txt, "{}", filenames[strip].display()).unwrap();
            let image = &images32f[strip];
            for (_, y, pixel) in image.enumerate_pixels() {
                png.put_pixel(index, y, *pixel);
            }
            index += 1;
        }
        // writeln!(txt, "").unwrap();
        index += 1;
    }

    let png = DynamicImage::from(png).into_rgb8();
    png.save(PNG_PATH).unwrap();
    println!("result saved in {:.3?}", time.elapsed().unwrap());
}
