mod page;
use page::Page;

use serde::Deserialize;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize)]
pub struct Titles {
    pub pretty: String,
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
pub struct DoujinInternal {
    pub media_id: String,
    pub title: Titles,
    pub images: Images,
}

#[derive(Debug)]
pub struct Doujin {
    id: String,
    pub pages: Vec<Page>,
}

impl Doujin {
    pub fn new(id: &String) -> Doujin {
        Doujin {
            id: id.to_string(),
            pages: Vec::new(),
        }
    }
    pub async fn initialize(
        &mut self,
        client: reqwest::Client,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Perform the actual execution of the network request
        let body = client
            .get(format!("https://nhentai.net/api/gallery/{}", self.id))
            .send()
            .await?
            .json::<DoujinInternal>()
            .await?;

        // Grab what we need
        let dir = body.title.pretty.as_str();
        let media_id = body.media_id;
        let pages = body.images.pages;

        println!("Downloading: {}...", dir);
        let out_pages = pages
            .iter()
            .enumerate()
            .map(|(i, e)| Page::new(&media_id, &dir, i + 1, &e.t))
            .collect();
        create_dir_all(dir)?;
        let mut f = File::create(format!("{}/.id", dir))?;
        f.write_all(self.id.as_bytes())?;
        self.pages = out_pages;
        Ok(())
    }
}
