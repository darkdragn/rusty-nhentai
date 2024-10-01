pub mod search;

use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use serde_with::serde_as;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use url::Url;

use futures::stream::FuturesUnordered;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use reqwest::header::COOKIE;
use reqwest::header::USER_AGENT;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    cookie: String,
    user_agent: String
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
        None
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

    async fn download_image_to_zip(
        &self,
        i: usize,
        lock: Arc<RwLock<zip::ZipWriter<indicatif::ProgressBarIter<File>>>>,
        options: &zip::write::SimpleFileOptions,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _permit = self.semaphore.clone().acquire_owned().await?;
        let (url, filename) = self.internal.gen_image_detail(i);
        let mut res = self.client.get(url.as_str()).send().await?.bytes().await?;
        let mut zip = lock.write().await;

        zip.start_file(filename.as_str(), *options)?;
        zip.write_all(&mut res)?;
        // while let Some(item) = res.next().await {
        //     zip.write_all(&mut item?)?;
        // }

        Ok(())
    }
    pub async fn download_to_folder(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading {}...", self.dir);
        create_dir_all(self.dir.as_str())?;
        let mut f = File::create(format!("{}/.id", self.dir))?;
        f.write_all(self.id.as_bytes())?;

        let mut handles = FuturesUnordered::new();
        for i in 0..self.internal.images.pages.len() {
            handles.push(self.download_image_to_file(i))
        }
        while let Some(_result) = handles.next().await {}
        Ok(())
    }

    pub async fn download_to_zip(
        &mut self,
        use_author: bool,
        use_cbz: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ext = ".zip";
        if use_cbz {
            ext = ".cbz"
        }
        let mut filename = format!("{}{}", self.dir, ext);
        filename = filename.replace("/", "|");
        let windows_no_like = ['|', '\"', '<', '>', ':', '?', '\\', '/'];

        if cfg!(windows){
            filename.retain(|x| x.is_ascii());
            filename.retain(|x| {!windows_no_like.contains(&x)});
        }
        if use_author {
            let author = self.author.as_mut().unwrap();
            if cfg!(windows){
                author.retain(|x| {!windows_no_like.contains(&x)});
            }
            create_dir_all(&author)?;
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
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file(".id", options)?;
        zip.write_all(self.id.as_bytes())?;
        zip.start_file("tags.json", options)?;
        let tags = serde_json::to_string(&self.internal.tags)?;
        zip.write_all(tags.as_bytes())?;

        let lock = Arc::new(RwLock::new(zip));

        let mut handles = FuturesUnordered::new();
        for i in 0..self.internal.images.pages.len() {
            handles.push(self.download_image_to_zip(i, lock.clone(), &options))
        }
        while let Some(_result) = handles.next().await {}
        pb.finish();
        Ok(())
    }

    pub fn fetch_headers() -> Config
    {
        let mut config_path = format!("./rusty-nhentai.yaml");
        if Path::new("./rusty-nhentai.yaml").exists() {
        } else {
            config_path = format!("{}/.config/rusty-nhentai.yaml", std::env::var("HOME").unwrap());
        }
        let f = std::fs::File::open(config_path).expect("Could not open file.");
        let scrape_config: Config = serde_yaml::from_reader(f).expect("Could not read values.");
        return scrape_config;
    }
    pub async fn new(id: &String) -> Result<Doujin, Box<dyn std::error::Error>> {
        let semaphore = Arc::new(Semaphore::new(25));
        let client = reqwest::Client::builder().build()?;
        let url = Url::parse("https://nhentai.net/api/gallery/")?.join(id)?;

        // Perform the actual execution of the network request
        let scrape_config: Config = Doujin::fetch_headers();
        let resp = client.get(url)
            .header(COOKIE, scrape_config.cookie)
            .header(USER_AGENT, scrape_config.user_agent)
            .send().await?;
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
}
