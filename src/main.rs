mod page;

// use serde::Deserialize;
use std::env;
use std::fs::create_dir_all;
use std::sync::Arc;
//use std::fs::File;
//use std::io;

use serde_json::{Value};
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
        .json::<Value>()
        .await?;

    // println!("{:#?}", body["title"]["pretty"].as_str().unwrap());
    let dir = body["title"]["pretty"].as_str().unwrap();
    let media_id = body["media_id"].as_str().unwrap();
    let pages = body["images"]["pages"].as_array().unwrap();

    println!("Downloading: {}...", dir);
    let out_pages = pages
        .iter()
        .enumerate()
        .map(|(i, e)| page::Page::new(media_id, dir, i+1, e["t"].as_str().unwrap()))
        .collect();
    create_dir_all(dir)?;
    Ok(out_pages)
}

