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

    d.initialize().await?;

    let handles = d
        .pages
        .into_iter()
        .map(|page| tokio::spawn(page.download(semaphore.clone())));
    // tokio::join!(..handles).await;
    futures::future::join_all(handles).await;
    Ok(())
}
