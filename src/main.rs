// use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io;

use serde_json::{Value};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    let t = parse_doujin(&args[1]).await?;
    let handles = t
        .into_iter()
        .map(|page| tokio::spawn(download_page(page.url, page.filename)));
    futures::future::join_all(handles).await;
    Ok(())
}

struct Page {
    url: String,
    filename: String,
}

async fn parse_doujin(id: &String) -> Result<Vec<Page>, Box<dyn std::error::Error>> {
    // Build the client using the builder pattern
    let client = reqwest::Client::builder()
        .build()?;

    // Perform the actual execution of the network request
    let res = client
        .get(format!("https://nhentai.net/api/gallery/{}", id))
        // .get("https://nhentai.net/api/gallery/218530")
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
    let mut out_pages: Vec<Page> = Vec::new();
    for (i, _e) in pages.iter().enumerate() {
        let page = format!("https://i.nhentai.net/galleries/{}/{}.jpg", media_id, i+1);
        out_pages.push(Page {url: page, filename: format!("{}/{:0>3}.jpg", dir, i+1)})
    }
    fs::create_dir_all(dir)?;
    Ok(out_pages)
}

async fn download_page(url: String, filename: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .build()?;

    // Perform the actual execution of the network request
    let res = client
        .get(url)
        .send()
        .await?
        .bytes()
        .await?;

    // Grab a Ref to data, and write out to file
    let mut data = res.as_ref();
    let mut f = File::create(filename)?;
    io::copy(&mut data, &mut f)?;

    Ok(())
}
