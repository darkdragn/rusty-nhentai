mod doujin;

use clap::clap_app;
use doujin::Doujin;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            (@arg all: --all "Download all search results")
            (@arg author: -a "Write output to an author folder")
            (@arg cbz: -c "Write output to .cbz instead of .zip")
            (@arg english: -e "Appends langauge:english to the query string")
            (@arg long: -l "Appends pages:>100 to the query string")
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
                d.download_to_zip(false, false).await?;
            }
        }
        ("search", Some(sub_m)) => {
            let mut query: Vec<&str> = sub_m.values_of("QUERY").unwrap().collect();
            if sub_m.is_present("english") {
                query.push("language:english");
            }
            if sub_m.is_present("long") {
                query.push("pages:>100");
            }
            if sub_m.is_present("uncensored") {
                query.push("tags:uncensored");
            }

            let results = doujin::search::run_search(query.join(" ")).await?;
            let mut author = false;
            let mut use_cbz = false;
            if sub_m.is_present("author") {
                author = true;
            }
            if sub_m.is_present("cbz"){
                use_cbz = true;
            }
            if sub_m.is_present("numbers") {
                let numbers = sub_m.value_of("numbers").unwrap();
                for n in numbers.split(",") {
                    let index = n.parse::<usize>()?;
                    let mut target = results[index].clone();
                    target.download_to_zip(author, use_cbz).await?;
                }
            } else if sub_m.is_present("all") {
                for mut d in results {
                    d.download_to_zip(author, use_cbz).await?;
                }
            }
            else {
                for (i, d) in results.iter().enumerate() {
                    println!("{}: {:0>6} {} {}", i, d.id, d.dir, d.author.as_ref().unwrap());
                }
                println!("Number of Doujin: {}", results.len());
            }
        }
        _ => {}
    }
    Ok(())
}
