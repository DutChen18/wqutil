use std::{sync::{Mutex, Arc, atomic::{AtomicUsize, Ordering}}, fs::{self, File}, path::{Path, PathBuf}, io::Write};
use futures::StreamExt;
use image::imageops::FilterType;

const THREADS: usize = 6;

pub async fn get_links() -> Vec<String> {
    const PATH: &str = "https://docs.google.com/spreadsheets/d/1Y4qoXTpd0ZO2CRZzgYV3Lvd1Aihui_Ya6p0V89nKdNU/gviz/tq?tqx=out:csv&sheet=Unique%20Codes";
    let body = reqwest::get(PATH).await.unwrap().text().await.unwrap();
    let rdr = csv::Reader::from_reader(body.as_bytes());
    let mut links = Vec::new();
    
    for result in rdr.into_records() {
        let record = result.unwrap();
        links.push(record[0].to_string());
    }

    links
}

pub async fn download_images(dst: &str) -> (usize, usize) {
    let links = get_links().await;
    let client = reqwest::Client::new();
    let new = AtomicUsize::new(0);

    fs::create_dir_all(dst).unwrap();

    let results = futures::stream::iter(links)
        .map(|link| async {
            let url = url::Url::parse(&link).unwrap();
            let name = url.path_segments().unwrap().last().unwrap();
            let path = Path::new(dst).join(name);

            if !path.exists() {
                let resp = client.get(link).send().await.unwrap();
                let mut file = File::create(path).unwrap();
                file.write_all(&resp.bytes().await.unwrap()).unwrap();
                new.fetch_add(1, Ordering::SeqCst);
            }
        })
        .buffer_unordered(40)
        .count()
        .await;

    (results, new.load(Ordering::SeqCst))
}

pub fn cut_images(src: &str, dst: &str) -> (usize, usize) {
    let entries = Arc::new(Mutex::new(fs::read_dir(src).unwrap()));
    let mut handles = Vec::new();
    let total = Arc::new(AtomicUsize::new(0));
    let new = Arc::new(AtomicUsize::new(0));

    fs::create_dir_all(&dst).unwrap();

    for _ in 0..THREADS {
        let entries = entries.clone();
        let dst = PathBuf::from(dst);
        let total = total.clone();
        let new = new.clone();

        handles.push(std::thread::spawn(move || {
            while let Some(entry) = {
                let mut entries = entries.lock().unwrap();
                entries.next()
            } {
                let path = entry.unwrap().path();
                let mut image = None;

                for i in 0..5 {
                    let name = format!("{}-{}", path.file_stem().unwrap().to_str().unwrap(), i);
                    let dst = dst.join(name).with_extension(path.extension().unwrap());

                    if !dst.exists() {
                        let image = if let Some(ref image) = image {
                            image
                        } else {
                            image.insert(image::open(&path).unwrap())
                        };

                        let image = image.crop_imm(200 + i * 210, 103, 10, 10330);
                        let image = image.resize(10, 1033, FilterType::Nearest);

                        image.save(dst).unwrap();
                        new.fetch_add(1, Ordering::SeqCst);
                    }

                    total.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    (total.load(Ordering::SeqCst), new.load(Ordering::SeqCst))
}