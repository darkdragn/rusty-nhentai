mod doujin;

use std::fs::create_dir_all;

use clap::clap_app;
use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let args: Vec<String> = env::args().collect();

    let matches = clap_app!(RustyNHentai =>
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
            (@arg author: -a +takes_value "Write output to an author folder")
            (@arg english: -e "Appends langauge:english to the query string")
            (@arg numbers: -n +takes_value "Index within the search to download")
            (@arg uncensored: -u "Appends tags:uncensored to the query string")
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
            let mut query: Vec<&str> = sub_m.values_of("QUERY").unwrap().collect();
            if sub_m.is_present("english") {
                query.push("language:english");
            }
            if sub_m.is_present("uncensored") {
                query.push("tags:uncensored");
            }

            let results = doujin::search::run_search(query.join(" ")).await?;
            let mut author: Option<String> = None;
            if sub_m.is_present("author") {
                let author_dir = sub_m.value_of("author").unwrap();
                author = Some(author_dir.to_string());
                create_dir_all(author_dir)?;
            }
            if sub_m.is_present("numbers") {
                let numbers = sub_m.value_of("numbers").unwrap();
                for n in numbers.split(",") {
                    let index = n.parse::<usize>()?;
                    let mut target = results[index].clone();
                    target.author = author.clone();
                    target.download_to_zip().await?;
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
