mod doujin;

use std::env;
use std::sync::Arc;

use tokio::sync::Semaphore;

use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(20));
    let args: Vec<String> = env::args().collect();
    let mut d = Doujin::new(&args[1]);
    let client = reqwest::Client::builder().build()?;

    d.initialize(client.clone()).await?;

    let handles = d
        .pages
        .into_iter()
        .map(|page| page.download(client.clone(), semaphore.clone()));
    futures::future::join_all(handles).await;
    Ok(())
}
