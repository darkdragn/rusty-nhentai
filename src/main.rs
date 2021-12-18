mod doujin;

use std::env;
use std::sync::Arc;

use std::fs::File;
// use std::io::Write;
// use zip::write::FileOptions;

use tokio::sync::RwLock;
use tokio::sync::Semaphore;

use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(20));
    let args: Vec<String> = env::args().collect();
    let mut d = Doujin::new(&args[1]);
    let client = reqwest::Client::builder().build()?;

    d.initialize(client.clone()).await?;

    let f = File::create(format!("{}.zip", d.dir))?;
    let mut zip = zip::ZipWriter::new(f);
    zip = d.start_zip(zip)?;

    let lock = Arc::new(RwLock::new(zip));

    let handles = d
        .pages
        .into_iter()
        .map(|page| page.download_to_zip(client.clone(), lock.clone(), semaphore.clone()));
    futures::future::join_all(handles).await;
    // zip.finish()?;
    Ok(())
}
