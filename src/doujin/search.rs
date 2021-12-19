use super::Doujin;
use super::DoujinInternal;
use super::Page;

use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::Semaphore;

#[derive(Clone, Debug, Deserialize)]
struct Search {
    num_pages: u8,
    result: Vec<DoujinInternal>,
}

pub async fn run_search(query: String) -> Result<Vec<Doujin>, Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(25));
    let client = reqwest::Client::builder().build()?;
    let query_set = ("query", query.as_str());
    let mut url = url::Url::parse("https://nhentai.net/api/galleries/search")?;

    let mut page = 1u8;
    let mut results: Vec<DoujinInternal> = Vec::new();
    loop {
        url.query_pairs_mut()
            .clear()
            .extend_pairs(&[query_set, ("page", &page.to_string())]);

        let resp = client.get(url.as_str()).send().await?;
        let body = resp.json::<Search>().await?;
        results.extend(body.result);
        page += 1;
        if page > body.num_pages {
            break;
        }
    }
    let mut output: Vec<Doujin> = Vec::new();
    for d in results.iter() {
        let media_id = &d.media_id;
        let title = &d.title.pretty;
        let pages: Vec<Page> = d
            .images
            .pages
            .iter()
            .enumerate()
            .map(|(i, e)| Page::new(&media_id, title, i + 1, &e.t))
            .collect();
        output.push(Doujin {
            id: d.id.to_string(),
            client: client.clone(),
            dir: title.clone(),
            pages: pages,
            semaphore: semaphore.clone(),
            author: None,
        })
    }
    Ok(output)
}
