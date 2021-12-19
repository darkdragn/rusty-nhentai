mod page;
use page::Page;

use serde::Deserialize;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use url::Url;

use reqwest::Client;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

#[derive(Deserialize)]
pub struct Titles {
    pretty: String,
}

#[derive(Deserialize)]
pub struct Image {
    pub t: String,
}

#[derive(Deserialize)]
pub struct Images {
    pub pages: Vec<Image>,
}

#[derive(Deserialize)]
struct DoujinInternal {
    media_id: String,
    title: Titles,
    images: Images,
}

#[derive(Debug)]
pub struct Doujin {
    id: String,
    client: Client,
    dir: String,
    pages: Vec<Page>,
    semaphore: Arc<Semaphore>,
}

impl Doujin {
    pub async fn new(id: &String) -> Result<Doujin, Box<dyn std::error::Error>> {
        let semaphore = Arc::new(Semaphore::new(25));
        let client = reqwest::Client::builder().build()?;
        let base = Url::parse("https://nhentai.net/api/gallery/")?;

        // Perform the actual execution of the network request
        let resp = client.get(base.join(id)?).send().await?;
        let body = resp.json::<DoujinInternal>().await?;

        // Grab what we need
        let media_id = body.media_id;
        let title = body.title.pretty;
        let pages = body
            .images
            .pages
            .iter()
            .enumerate()
            .map(|(i, e)| Page::new(&media_id, &title, i + 1, &e.t))
            .collect();

        Ok(Doujin {
            id: id.to_string(),
            client: client,
            dir: title,
            pages: pages,
            semaphore: semaphore,
        })
    }

    pub async fn download_to_folder(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading {}...", self.dir);
        create_dir_all(self.dir.as_str())?;
        let mut f = File::create(format!("{}/.id", self.dir))?;
        f.write_all(self.id.as_bytes())?;

        let handles = self
            .pages
            .clone()
            .into_iter()
            .map(|page| page.download_to_folder(self.client.clone(), self.semaphore.clone()));
        futures::future::join_all(handles).await;
        Ok(())
    }

    pub async fn download_to_zip(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading {}...", self.dir);
        let f = File::create(format!("{}.zip", self.dir))?;
        let mut zip = zip::ZipWriter::new(f);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file(".id", options)?;
        zip.write_all(self.id.as_bytes())?;

        let lock = Arc::new(RwLock::new(zip));

        let handles = self.pages.clone().into_iter().map(|page| {
            page.download_to_zip(
                self.client.clone(),
                lock.clone(),
                self.semaphore.clone(),
                &options,
            )
        });
        futures::future::join_all(handles).await;
        Ok(())
    }
}
