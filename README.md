# terashite

A web scraper for Japanese akari archives/sites, implemented in Rust.
It could relatively trivially be extended to scrape any puzzle type from the implemented domains.

## Domains

We have scrapers for the [bachelor_seal](http://blog.livedoor.jp/bachelor_seal-puzzle/archives/cat_531435.html) akari archive and the [tibisukemaru](http://tibisukemaru.blog.fc2.com/blog-category-14.html) akari archive. More can be added in the future.

## Usage

For specifics, see the `--help` output. Generally speaking, you can either specify directly:

- puzzle urls (only works if we know how to handle it) using `-s/--single`
- domains (only works if we have a scraper for it) using `-d/--domain`
- or just scrape everything by running without any arguments

It is probably useful to list the supported domains first, to know what you can scrape:

```bash
cargo run -- --list-domains
```

It is probably also useful to create a config file to specify where you want puzzles to be saved.
When trying to read a config file, we look for `terashite.conf` in the XDG config directory (e.g. `~/.config/terashite/terashite.conf`), and if it doesn't exist, we look for `terashite.conf` in the home directory. If neither exists, we use the default config, which saves puzzles to `./terashite` (i.e. a `terashite` directory in the current working directory).

A config file is exceedingly simple, and looks like this:

```toml
# This is a comment that will be ignored.
out = "/path/to/save/puzzles"
```

## Puzzle format

Puzzles are saved using following naming scheme, where words in curlies are `{placeholders}`.

```raw
{out}/{domain-id}/{puzzle-no}-{difficulty}.pzprc
```

They are plaintext files -- the extension is chosen as `pzpr` for the [pzpr format](<https://github.com/robx/pzprjs/>), with `c` for comments, similar to `jsonc` to `json`. Comments start with `#`.

```raw
# domain_name: {domain-name}
# difficulty: {difficulty}
# puzzle_no: {puzzle-no}
# at: {date}
# from_url: {url}
{pzpr}
```

| Placeholder | Explanation |
| - | - |
| `out` | the output directory specified in the config file, or `./terashite` if no config file is found |
| `domain-name` | a short string identifying the domain the puzzle was scraped from |
| `puzzle-no` | the puzzle number from the domain |
| `difficulty` | the difficulty of the puzzle, mapped to a human-readable word, e.g. `easy`, `medium`, `hard` |
| `date` | the date at which the puzzle was scraped |
| `url` | the URL of the original puzzle page |
| `pzpr` | the puzzle in pzpr format, e.g. `11/13/rcjascjazlezlbjdsbjdp` |
