mod page;

use std::env;
use std::fs::create_dir_all;
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(20));
    let args: Vec<String> = env::args().collect();
    let t = parse_doujin(&args[1]).await?;
    let handles = t
        .into_iter()
        .map(|page| tokio::spawn(page.download(semaphore.clone())));
    futures::future::join_all(handles).await;
    Ok(())
}

#[derive(Deserialize)]
struct Titles {
    pretty: String
}

#[derive(Deserialize)]
struct Image {
    t: String
}

#[derive(Deserialize)]
struct Images {
    pages: Vec<Image>
}

#[derive(Deserialize)]
struct Doujin {
    media_id: String,
    title: Titles,
    images: Images
}

async fn parse_doujin(id: &String) -> Result<Vec<page::Page>, Box<dyn std::error::Error>> {
    // Build the client using the builder pattern
    let client = reqwest::Client::builder()
        .build()?;

    // Perform the actual execution of the network request
    let res = client
        .get(format!("https://nhentai.net/api/gallery/{}", id))
        .send()
        .await?;

    // Parse the response body as Json in this case
    let body = res
        .json::<Doujin>()
        .await?;

    let dir = body.title.pretty;
    let media_id = body.media_id;
    let pages = body.images.pages;

    println!("Downloading: {}...", dir);
    let out_pages = pages
        .iter()
        .enumerate()
        .map(|(i, e)| page::Page::new(&media_id, &dir, i+1, &e.t )) //["t"].as_str().unwrap()))
        .collect();
    create_dir_all(dir)?;
    Ok(out_pages)
}

