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
        (@subcommand search =>
            (@setting ArgRequiredElseHelp)
            (about: "Search for doujin")
            (@arg numbers: -n +takes_value "Index within the search to download")
            (@arg QUERY: +required ... "Query string")
        )
    )
    .get_matches();

    match matches.subcommand() {
        ("pull", Some(sub_m)) => {
            let magic = sub_m.value_of("MAGIC").unwrap().to_string();
            let mut d = Doujin::new(&magic).await?;
            if sub_m.is_present("folder") {
                print!("Outputting to folder; ");
                d.download_to_folder().await?;
            } else {
                print!("Outputting to zipfile; ");
                d.download_to_zip().await?;
            }
        }
        ("search", Some(sub_m)) => {
            let query: Vec<&str> = sub_m.values_of("QUERY").unwrap().collect();
            let mut results = doujin::search::run_search(query.join(" ")).await?;
            if sub_m.is_present("numbers") {
                let numbers = sub_m.value_of("numbers").unwrap();
                for n in numbers.split(",") {
                    let index = n.parse::<usize>()?;
                    results[index].download_to_zip().await?;
                }
            } else {
                for (i, d) in results.iter().enumerate() {
                    println!("{}: {:0>6} {}", i, d.id, d.dir);
                }
                println!("Number of Doujin: {}", results.len());
            }
        }
        _ => {}
    }
    Ok(())
}
