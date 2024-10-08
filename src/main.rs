mod doujin;

use clap::clap_app;
use doujin::Doujin;
use prettytable::{Table, row};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(RustyNHentai =>
        (@setting SubcommandRequiredElseHelp)
        (version: env!("CARGO_PKG_VERSION"))
        (author: "Darkdragn <darkdragn.cjp@gmail.com>")
        (about: "Quick downloader for NHentai, to learn Rust")
        (@subcommand pull =>
            (@setting ArgRequiredElseHelp)
            (about: "Download a Doujin from nhentai.net")
            (version: env!("CARGO_PKG_VERSION"))
            (@arg MAGIC: +required "Magic number for Doujin to download")
            (@arg folder: -f "Download to folder instead of zip (Zip is the default")
        )
        (@subcommand search =>
            (@setting ArgRequiredElseHelp)
            (about: "Search for doujin")
            (version: env!("CARGO_PKG_VERSION"))
            (@arg all: --all "Download all search results")
            (@arg author: -a "Write output to an author folder")
            (@arg english: -e "Appends langauge:english to the query string")
            (@arg long: -l "Appends pages:>100 to the query string")
            (@arg numbers: -n +takes_value "Index within the search to download")
            (@arg uncensored: -u "Appends tags:uncensored to the query string")
            (@arg zip: -z "Write output to .cbz instead of .zip")
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
                print!("Outputting to zipfile (CBZ); ");
                d.download_to_zip(false, true).await?;
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
            let mut use_cbz = true;
            if sub_m.is_present("author") {
                author = true;
            }
            if sub_m.is_present("zip") {
                use_cbz = false;
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
            } else {
                let mut table = Table::new();
                table.add_row(row!["Index", "ID", "Name", "Author"]);
                for (i, d) in results.iter().enumerate() {
                    let default_author = "No Author".to_string();
                    let author_name = d.author.as_ref().unwrap_or(&default_author);
                    table.add_row(row![i, d.id, d.dir, author_name]);
                    // println!("{}: {:0>6} {} {}", i, d.id, d.dir, author_name);
                }
                table.printstd();
                println!("Number of Doujin: {}", results.len());
            }
        }
        _ => {}
    }
    Ok(())
}
