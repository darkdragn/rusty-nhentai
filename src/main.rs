mod doujin;

use clap::clap_app;
use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args: Vec<String> = env::args().collect();

    let matches = clap_app!(myapp =>
        (@setting SubcommandRequiredElseHelp)
        (version: "1.0")
        (author: "Darkdragn <darkdragn.cjp@gmail.com>")
        (about: "Quick downloader for NHentai, to learn Rust")
        (@subcommand pull =>
            (@setting ArgRequiredElseHelp)
            (about: "Download a Doujin from nhentai.net")
            (version: "1.0")
            (@arg MAGIC: +required "Magic number for Doujin to download")
            (@arg folder: -f "Download to folder instead of zip (Zip is the default")
        )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("pull"){
        let magic = matches.value_of("MAGIC").unwrap().to_string();
        let mut d = Doujin::new(&magic);
        if matches.is_present("folder") {
            print!("Outputting to folder; ");
            d.download_to_folder().await?;
        } else {
            print!("Outputting to zipfile; ");
            d.download_to_zip().await?;
        }
    }
    Ok(())
}
