// mod page;
pub mod search;
// use page::Page;

use serde::Deserialize;
// use serde_json::value::Value;
use serde_with::serde_as;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use url::Url;

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

#[derive(Clone, Debug, Deserialize)]
pub struct Tag {
    r#type: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Titles {
    pretty: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Image {
    pub t: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Images {
    pub pages: Vec<Image>,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
struct DoujinInternal {
    #[serde_as(as = "serde_with::PickFirst<(_, serde_with::DisplayFromStr)>")]
    id: u32,
    media_id: String,
    title: Titles,
    images: Images,
    tags: Vec<Tag>,
}

#[derive(Clone, Debug)]
pub struct Doujin {
    pub id: String,
    client: Client,
    pub dir: String,
    semaphore: Arc<Semaphore>,
    pub author: Option<String>,
    internal: DoujinInternal,
}

impl Image {
    fn ext(&self) -> &str {
        match self.t.as_str() {
            "j" => ".jpg",
            "p" => ".png",
            &_ => ".jpg",
        }
    }
}

impl DoujinInternal {
    pub fn find_artist(&self) -> Option<String> {
        for tag in &self.tags {
            if tag.r#type == "artist" {
                return Some(tag.name.clone());
            }
        }
        Some("Unknown".to_string())
    }
    pub fn gen_image_detail(&self, id: usize) -> (String, String) {
        let page = &self.images.pages[id];
        let url = format!(
            "https://i.nhentai.net/galleries/{}/{}{}",
            self.media_id,
            id + 1,
            page.ext()
        );
        let filename = format!("{}/{:0>3}{}", self.title.pretty, id + 1, page.ext());
        return (url, filename);
    }
}

impl Doujin {
    pub async fn new(id: &String) -> Result<Doujin, Box<dyn std::error::Error>> {
        let semaphore = Arc::new(Semaphore::new(25));
        let client = reqwest::Client::builder().build()?;
        let base = Url::parse("https://nhentai.net/api/gallery/")?;

        // Perform the actual execution of the network request
        let resp = client.get(base.join(id)?).send().await?;
        let body = resp.json::<DoujinInternal>().await?;

        Ok(Doujin {
            id: id.clone(),
            client,
            dir: body.title.pretty.clone(),
            semaphore,
            author: body.find_artist(),
            internal: body,
        })
    }

    pub async fn download_to_folder(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading {}...", self.dir);
        create_dir_all(self.dir.as_str())?;
        let mut f = File::create(format!("{}/.id", self.dir))?;
        f.write_all(self.id.as_bytes())?;

        let mut handles = Vec::new();
        for i in 0..self.internal.images.pages.len() {
            handles.push(self.download_image_to_file(i))
        }
        futures::future::join_all(handles).await;
        Ok(())
    }
    async fn download_image_to_file(&self, i: usize) -> Result<(), Box<dyn std::error::Error>> {
        let _permit = self.semaphore.clone().acquire_owned().await?;
        let (url, filename) = self.internal.gen_image_detail(i);
        let mut res = self.client.get(url.as_str()).send().await?.bytes_stream();
        let mut file = tokio::fs::File::create(filename.as_str()).await?;
        while let Some(item) = res.next().await {
            file.write_all_buf(&mut item?).await?;
        }
        Ok(())
    }

    pub async fn download_to_zip(
        &mut self,
        use_author: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut filename = format!("{}.zip", self.dir);
        if use_author {
            let author = self.author.as_ref().unwrap();
            create_dir_all(author)?;
            filename = format!("{}/{}", author, filename);
        }
        if Path::new(&filename).exists() {
            println!("File already exists: {}", filename);
            return Ok(());
        }

        let f = File::create(filename)?;
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} [{elapsed_precise}] {bytes} ({bytes_per_sec})"),
        );
        pb.enable_steady_tick(200);
        pb.set_message(self.dir.clone());

        let mut zip = zip::ZipWriter::new(pb.wrap_write(f));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file(".id", options)?;
        zip.write_all(self.id.as_bytes())?;

        let lock = Arc::new(RwLock::new(zip));

        let mut handles = Vec::new();
        for i in 0..self.internal.images.pages.len() {
            handles.push(self.download_image_to_zip(i, lock.clone(), &options))
        }
        futures::future::join_all(handles).await;
        pb.finish();
        Ok(())
    }

    async fn download_image_to_zip(
        &self,
        i: usize,
        lock: Arc<RwLock<zip::ZipWriter<indicatif::ProgressBarIter<File>>>>,
        options: &zip::write::FileOptions,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // use std::io::Write;
        let _permit = self.semaphore.clone().acquire_owned().await?;
        let (url, filename) = self.internal.gen_image_detail(i);
        let mut res = self.client.get(url.as_str()).send().await?.bytes_stream();
        let mut zip = lock.write().await;

        zip.start_file(filename.as_str(), *options)?;
        while let Some(item) = res.next().await {
            zip.write_all(&mut item?)?;
        }

        Ok(())
    }
}
