use terashite::scraper::implemented_scrapers;

/// Dumb entry point: for each implemented scraper, scrape everything.
#[tokio::main]
async fn main() {
    for (scraper, downloader) in implemented_scrapers() {
        let domain = scraper.domain_id();
        println!("Scraping domain: {domain:?}");
        match scraper.fetch_new_puzzles(&downloader).await {
            Ok(results) => println!("Found {} puzzles for {domain:?}", results.len()),
            Err(e) => eprintln!("Error scraping {domain:?}: {e}"),
        }
    }
}
