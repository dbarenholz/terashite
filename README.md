# terashite

A web scraper for Japanese akari archives/sites, implemented in Rust.
It could relatively trivially be extended to scrape any puzzle type from the implemented domains.

## Domains

We have scrapers for the [bachelor_seal](http://blog.livedoor.jp/bachelor_seal-puzzle/archives/cat_531435.html) akari archive and the [tibisukemaru](http://tibisukemaru.blog.fc2.com/blog-category-14.html) akari archive. More can be added in the future.

## Usage

The scraper is a single binary with no options. It always scrapes all new puzzles from all implemented domains, and saves them to disk. We save the visited html files under `~/.terashite/html`, and resulting puzzles in `~/.terashite/puzzles`. When rerunning the scraper, if html files exist locally, they will be used instead of hammering the servers needlessly. If html files need to be fetched, we only send 1 request every 10 seconds (per domain) to avoid DoS-ing the servers.

## Puzzle format

Puzzles are saved using following naming scheme, where words in curlies are `{placeholders}`.

```raw
~/.terashite/puzzles/{domain-id}-{difficulty}-{puzzle-id}.pzprc
```

They are plaintext files -- the extension is chosen as `pzpr` for the [pzpr format](#), with `c` for comments, similar to `jsonc` to `json`. Comments start with `#`.

```txt
# scraped by terashite at {date}
# source: {url}
{width}
{heigth}
{pzpr-string}
```

| Placeholder | Explanation |
| - | - |
| `domain-id` | a short string identifying the domain the puzzle was scraped from |
| `difficulty` | the difficulty of the puzzle, mapped to a human-readable word, e.g. `easy`, `medium`, `hard` |
| `puzzle-id` | a unique identifier for the puzzle for the domain |
| `date` | the date at which the puzzle was scraped |
| `url` | the URL of the original puzzle page |
| `width` | the width of the puzzle |
| `height` | the height of the puzzle |
| `pzpr-string` | the puzzle in pzpr format (with width and height removed) |
