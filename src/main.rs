mod doujin;

use std::env;

use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut d = Doujin::new(&args[1]);

    d.download_to_zip().await?;
    Ok(())
}
