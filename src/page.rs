use std::sync::Arc;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct Page {
    url: String,
    filename: String,
}

impl Page {
    pub fn new(url: String, filename: String) -> Page{
        Page { url: url, filename: filename}
    }

    pub async fn download(self: Self, semaphore: Arc<Semaphore>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .build()?;
    
        let permit = semaphore.acquire_owned().await?;
    
        // Perform the actual execution of the network request
        let mut res = client
            .get(self.url.as_str())
            .send()
            .await?
            .bytes_stream();
        let mut file = tokio::fs::File::create(self.filename.as_str()).await?;
        while let Some(item) = res.next().await {
            file.write_all_buf(&mut item?).await?;
        }
        drop(permit);
    
        Ok(())
    }
}