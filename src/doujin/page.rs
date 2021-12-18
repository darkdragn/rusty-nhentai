use std::fs::File;
use std::sync::Arc;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::sync::Semaphore;
// use zip::write::FileOptions;

#[derive(Debug)]
pub struct Page {
    url: String,
    filename: String,
}

impl Page {
    pub fn new(media_id: &str, dir: &str, number: usize, type_: &str) -> Page {
        let ext = match type_ {
            "j" => ".jpg",
            "p" => ".png",
            &_ => ".jpg",
        };
        let page = format!(
            "https://i.nhentai.net/galleries/{}/{}{}",
            media_id, number, ext
        );
        let filename = format!("{}/{:0>3}{}", dir, number, ext);
        Page {
            url: page,
            filename: filename,
        }
    }

    pub async fn download(
        self: Self,
        client: reqwest::Client,
        semaphore: Arc<Semaphore>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let permit = semaphore.acquire_owned().await?;
        let mut res = client.get(self.url.as_str()).send().await?.bytes_stream();
        let mut file = tokio::fs::File::create(self.filename.as_str()).await?;
        while let Some(item) = res.next().await {
            file.write_all_buf(&mut item?).await?;
        }
        drop(permit);
        Ok(())
    }
    pub async fn download_to_zip(
        self: Self,
        client: reqwest::Client,
        // zip: &zip::ZipWriter<&mut File>,
        lock: Arc<RwLock<zip::ZipWriter<File>>>,
        semaphore: Arc<Semaphore>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::io::Write;
        let _permit = semaphore.acquire_owned().await?;
        let mut res = client.get(self.url.as_str()).send().await?.bytes_stream(); //.bytes().await?;
        let mut zip = lock.write().await;
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file(self.filename.as_str(), options)?;
        while let Some(item) = res.next().await {
            (*zip).write_all(&mut item?)?;
        }
        Ok(())
    }
}
