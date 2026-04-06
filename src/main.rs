use std::collections::HashMap;

use clap::Parser;
use terashite::disk;
use terashite::html;
use terashite::scraper;

// TODO: Really this should be a bunch of subcommands, but then -s and -d should be smart enough to get multiple domains/puzzles
/// CLI arguments.
///
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about,
    // same style as Clap, hence the ascii codes
    after_help = "\x1b[1m\x1b[4mNote:\x1b[0m All options are mutually exclusive."

)]
struct Args {
    /// List identifiers and example puzzle urls for all implemented scrapers.
    #[arg(long = "list-domains", short = 'l', conflicts_with_all = ["domains", "singles", "dump_config"])]
    list_domains: bool,

    /// Dump the used config to stdout and exit.
    #[arg(long="dump-config", short='c', conflicts_with_all = ["list_domains", "domains", "singles"])]
    dump_config: bool,

    /// Run scrapers for passed domains.
    #[arg(long = "domain", short = 'd', conflicts_with_all = ["list_domains", "singles", "dump_config"])]
    domains: Vec<String>,

    /// Only scrape a singular puzzle from a particular url.
    #[arg(long = "single", short = 's', conflicts_with_all = ["domains", "list_domains", "dump_config"])]
    singles: Vec<String>,
}

/// Application entry point.
#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.list_domains {
        list_domains();
        return;
    }

    if args.dump_config {
        let cfg = disk::get_config();
        println!("{cfg}");
        return;
    }

    let mut cfg = disk::get_config();
    disk::ensure_outs(&cfg).unwrap_or_else(|e| {
        eprintln!("Could not ensure output directories exist: {e}");
        std::process::exit(1);
    });
    let client = html::create_client();
    let downloader = html::create_downloader(&client);

    // TODO: it should probably be refactored so that we can do scraper::puzzles() and scraper::domains()
    // NOTE: this block will modify cfg.cache
    if !args.singles.is_empty() {
        scrape_puzzles(args.singles, &downloader, &mut cfg).await;
    } else if !args.domains.is_empty() {
        scrape_domains(args.domains, &downloader, &mut cfg).await;
    } else {
        scrape_domains(scraper::names(), &downloader, &mut cfg).await;
    }

    // Always write cache back to disk
    disk::write_cache(&cfg).unwrap_or_else(|e| {
            eprintln!("Error writing cache to disk: {e}");
            eprintln!("Here's the full cache that we tried to write: {:#?}", cfg.cache);
            eprintln!("We suggest either fixing file permissions, or manually writing the cache at the expected path.");
            std::process::exit(1);
        });
}

async fn scrape_puzzles(
    urls: Vec<String>,
    downloader: &html::HTMLDownloader<'_>,
    cfg: &mut disk::Cfg,
) {
    // map url to domain id or "invalid" if no domain matches
    let mut validity_mp: HashMap<String, Vec<String>> = HashMap::new();

    'urls: for url in &urls {
        for name in scraper::names() {
            if scraper::url_ok_for(name, url) == Ok(true) {
                validity_mp
                    .entry(name.to_string())
                    .and_modify(|v| v.push(url.clone()))
                    .or_insert_with(|| vec![url.clone()]);
                continue 'urls; // save minimal time by not checking other domains, since they can't match anyhow
            }
        }

        // if we get here, the url is invalid
        validity_mp
            .entry("invalid".to_string())
            .and_modify(|v| v.push(url.clone()))
            .or_insert_with(|| vec![url.clone()]);
    }

    if validity_mp.contains_key("invalid") {
        let invalids = validity_mp.get("invalid").unwrap();
        eprint!("There was at least one invalid puzzle url. ");
        eprint!("If invalid puzzle urls are included, we don't do any work. ");
        eprintln!("Please fix the invalid puzzle urls and try again.");
        eprintln!("Invalid: {}", invalids.join(", "));
        return;
    }

    for (domain, urls) in validity_mp {
        let Some(scraper) = scraper::for_name_as_single(&domain) else {
            eprintln!("Scraper '{domain}' does not support single puzzle scraping. Skipping.");
            continue;
        };

        for url in urls {
            eprintln!("Scraping puzzle url '{url}' with scraper '{domain}'...");

            match scraper.fetch_single(&url, downloader, cfg).await {
                scraper::ScrapeResult::IsSavedAt(path) => {
                    eprintln!("Puzzle from url '{url}' is already saved at '{path}'.");
                }
                scraper::ScrapeResult::Ok(puzzle) => match disk::save_puzzle(&puzzle, cfg) {
                    Ok(saved_at) => eprintln!("Saved puzzle from '{url}' to '{saved_at}'."),
                    Err(e) => eprintln!("Error saving puzzle from url '{url}': {e}"),
                },
                scraper::ScrapeResult::Err(e) => {
                    eprintln!("Error scraping puzzle url '{url}': {e}");
                }
            }
        }
    }
}

async fn scrape_domains<T: AsRef<str>>(
    domain_ids: Vec<T>,
    downloader: &html::HTMLDownloader<'_>,
    cfg: &mut disk::Cfg,
) {
    let scraper_ids = scraper::names();
    let (invalids, valids) = domain_ids
        .iter()
        .map(std::convert::AsRef::as_ref)
        .partition::<Vec<_>, _>(|d| !scraper_ids.contains(d));

    if !invalids.is_empty() {
        eprint!("There was at least one invalid domain specified. ");
        eprint!("If invalid domains are included, we don't do any work. ");
        eprintln!("Please fix the invalid domains and try again.");
        eprintln!("Invalid domains: {}", invalids.join(", "));
        eprintln!("Valid domains (your input): {}", valids.join(", "));
        eprintln!("Valid domains (implemented): {}", scraper_ids.join(", "));
        return;
    }

    if valids.is_empty() {
        eprint!("No domains specified. ");
        eprint!("Please specify at least one domain to scrape, ");
        eprintln!("or use --list-domains to see valid domains and example puzzle urls.");
        return;
    }

    for domain in valids {
        let scraper = scraper::for_name(domain).unwrap();

        // TODO: This architecture forces us to first scrape ALL puzzles, and then save them.
        // It would be a lot nicer to save each puzzle immediately, so we don't "lose" correct results on failure
        eprintln!("Scraping domain '{domain}'...");
        match scraper.fetch_puzzles(downloader, cfg).await {
            Ok(results) => {
                for result in &results {
                    match result {
                        scraper::ScrapeResult::IsSavedAt(_) => {
                            eprintln!("Puzzle from domain '{domain}' is already saved. Skipping.");
                        }
                        scraper::ScrapeResult::Ok(puzzle) => match disk::save_puzzle(puzzle, cfg) {
                            Ok(path) => {
                                eprintln!("Saved puzzle from domain '{domain}' to '{path}'.")
                            }
                            Err(e) => eprintln!("Error saving puzzle from domain '{domain}': {e}"),
                        },
                        scraper::ScrapeResult::Err(e) => {
                            eprintln!("Error scraping puzzle from domain '{domain}': {e}");
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error scraping domain '{domain}': {e}"),
        }
    }
}

/// List identifiers and urls for all implemented scrapers and exit.
fn list_domains() {
    let output = scraper::SCRAPER_INFO
        .iter()
        .map(|(name, (_domain, puzzle_url))| format!("{name}: {puzzle_url}"))
        .collect::<Vec<_>>()
        .join("\n");
    println!("{output}");
}
