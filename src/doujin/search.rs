use super::Config;
use super::Doujin;
use super::DoujinInternal;

use std::sync::Arc;

use reqwest::header::COOKIE;
use reqwest::header::USER_AGENT;
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
    let sort = ("sort", "popular");
    let mut url = url::Url::parse("https://nhentai.net/api/galleries/search")?;

    let mut page = 1u8;
    let mut results: Vec<Doujin> = Vec::new();
    loop {
        url.query_pairs_mut()
            .clear()
            .extend_pairs(&[query_set, sort, ("page", &page.to_string())]);

        let scrape_config: Config = Doujin::fetch_headers();
        let resp = client.get(url.as_str())
            .header(COOKIE, scrape_config.cookie)
            .header(USER_AGENT, scrape_config.user_agent)
            .send().await?;
        let body = resp.json::<Search>().await?;
        results.extend(body.result.iter().map(|d| -> Doujin {
            Doujin {
                id: d.id.to_string(),
                client: client.clone(),
                dir: d.title.pretty.clone(),
                semaphore: semaphore.clone(),
                author: d.find_artist(),
                internal: d.clone(),
            }
        }));
        page += 1;
        if page > body.num_pages {
            break;
        }
    }
    Ok(results)
}
