## Introduction

I'm just slapping this together last minute. I've used this solo for years and
some others might want it. 

### TL;DR

1. Download a binary from the releases tab or build it yourself, see below
1. Create a yaml called rusty-nhentai.yaml in your current directory
  1. it should have the keys cookies and user_agent, get those from your
     browser inspect screen
1. Enjoy! You can pull with the magic number or search.

### Building

As simple as: `cargo build --release`

### Using the ripper

It's currently undocumented, I need to update that, but it leverages a yaml
config file from either your current directory ./rusty-nhentay.yaml or 
${HOME}/.config/rusty-nhentai.yaml. It should have the following format:

```yaml

# You can literally open chrome, press CTRL+SHIFT+I, load nhentai and copy the
# cookie field from the first page load. This is for cloudflare bypass
cookie: "THE_COOKIE_DATA_COPIED_FROM_A_BROWSER"
user_agent: "MUST_MATCH_BROWSER_THE_COOKIE_CAME_FROM"

```

Here's a working example from my system:

```yaml
cookie: "cf_clearance=zBv03f_J0Lu1hkGQCxxC_BOPdNS3z8n3FN1iUH.SWms-1713673089-1.0.1.1-7PD3OLktHVSy0Uhst7cNrXvxkY_gL1xaHSmr09wtonDN2caD3AuSXtdbueL_fkoaRy2xYOLQLwja1qDf7oxVyQ; csrftoken=996K9kkecYKFy662mlCXtwbZvXorLQZSIGikantbe5G8N5CRYPodVylSsXED8aQ9"
user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"
```

They rotate their keys every 12 or so hours, so I don't expect that file to
work for you.

### Examples

#### Search

Some convience search options include:
- -e (Add language:english to the search)
- -l (Only list items with >100 pages)
- -a (Download results in folders named after the artists)
- --all (Download all results in the search)


Here's a download all from an artist, that have the language english tag:

```
-> % ./target/release/rusty-nhentai search artist:jairou -e -a --all
File already exists: jairou | xil/Shounen ga Otona ni Natta Natsu.cbz
File already exists: jairou/Rankou de Wakarou! | Let's Learn With Orgy!.cbz
File already exists: jairou/Gakuen Rankou.cbz
File already exists: jairou/Live!.cbz
File already exists: jairou/Shinjin Kyoushi Fujiwara-san no Ayashii Kyouin Nikki.cbz
File already exists: jairou/Yami no Serva FesWelcome to the Forbidden Paradise!.cbz
File already exists: jairou/Inran Shounen Nazo no Bitch Shota to Ossan no Monogatari Vol. 0.cbz
  Rankou de Wakarou! Ch. 1-3 [00:00:11] 93.55MiB (8.29MiB/s)
  Inran Shounen "Nazo no Bitch Shota to Ossan no Monogatari"  VOL.2 [00:00:06] 27.25MiB (3.99MiB/s)
File already exists: jairou/Rankou de Wakarou! Ch. 1-3.cbz
  Soto de Shiyou! Pakopako Densha Namabangumi [00:00:05] 6.18MiB (1.18MiB/s)
  Rankou de Wakarou!san no Ayashii Kyouin Nikki- Ch.4 [00:00:06] 27.89MiB (4.08MiB/s)
⠐ Inran Shounen "Nazo no Bitch Shota to Ossan no Monogatari" | Slut boy in the tale of a man and a mysterious sissy boy [00:00:04] 14.47MiB (3.36MiB/s)

```

Here's a search example:

```
-> % ./target/release/rusty-nhentai search artist:jairou -e         
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| Index | ID     | Name                                                                                                                  | Author       |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 0     | 516730 | Shounen ga Otona ni Natta Natsu                                                                                       | jairou | xil |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 1     | 479323 | Rankou de Wakarou! | Let's Learn With Orgy!                                                                           | jairou       |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 2     | 356541 | Gakuen Rankou                                                                                                         | jairou       |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 3     | 337563 | Live!                                                                                                                 | jairou       |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 4     | 381114 | Shinjin Kyoushi Fujiwara-san no Ayashii Kyouin Nikki                                                                  | jairou       |
+-------+--------+-----------------------------------------------------------------------------------------------------------------------+--------------+
| 5     | 283057 | Yami no Serva FesWelcome to the Forbidden Paradise!                                                                   | jairou       |
```

There were 75 results, but I don't need to post all of that. With search you
can refine down until a search is just what you want then run --all to download
all of then to your current directory. You can use `-a --all` to download all
of them to sub directories based on artist name!

### Upon request!!!! DOCKER

I can build a quick docker image for people on windows to make life easier,
upon request. Just open an issue, and I'm down to help!!!!
