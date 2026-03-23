pub mod bachelor_seal;
pub mod tibisukemaru;

use std::str::FromStr;

use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::html::HTMLDownloader;
use crate::model::{Difficulty, PzprStr, ScrapeResult};
use crate::scraper::bachelor_seal::BachelorSealScraper;
use crate::scraper::tibisukemaru::TibisukemaruScraper;

/// A trait for scraping akari puzzles from a variety of sources.
#[async_trait(?Send)]
pub trait Scraper {
    fn domain_id(&self) -> &'static str;

    /// The URL of the first page to scrape. The scraper will follow "next page" links until there are no more.
    fn first_url(&self) -> &'static str;

    /// CSS selector to obtain the URL of the next page.
    fn next_page_selector(&self) -> &'static str;

    /// CSS selector to obtain the URLs of individual puzzle entries on a page.
    fn entry_selector(&self) -> &'static str;

    /// CSS selector to obtain the URL of the pzpr page for a puzzle entry.
    fn pzpr_selector(&self) -> &'static str;

    /// Default implementation of the scraping logic -- should not need to change.
    async fn fetch_new_puzzles(
        &self,
        client: &HTMLDownloader,
    ) -> Result<Vec<ScrapeResult>, String> {
        let mut results = Vec::new();
        let mut current_url = self.first_url().to_string();

        loop {
            let html = client.fetch_url(&current_url).await?;

            let entry_urls = self.get_entry_urls(&html)?;
            for url in &entry_urls {
                let html = client.fetch_url(url).await?;
                let pzpr = self.get_pzpr(&html)?;
                let difficulty = self.get_difficulty(&html)?;
                results.push(ScrapeResult::new(pzpr, difficulty, url.to_string()));
            }

            match self.get_next_page_url(&html)? {
                Some(next_page_url) => current_url = next_page_url.to_string(),
                None => break,
            }
        }

        Ok(results)
    }

    /// Extract the URL of the next page from the HTML of a page.
    fn get_next_page_url<'a>(&self, html: &'a Html) -> Result<Option<&'a str>, String> {
        first_attr(html, self.next_page_selector(), "href", "next_page")
    }

    /// Extract the URLs of puzzle entries from the HTML of a page.
    fn get_entry_urls(&self, html: &Html) -> Result<Vec<String>, String> {
        collect_attrs(html, self.entry_selector(), "href", "entry")
    }

    /// Extract the pzpr URL from the HTML of a puzzle entry, and convert it to a PzprStr.
    fn get_pzpr(&self, html: &Html) -> Result<PzprStr, String> {
        let pzpr_url = first_attr_required(html, self.pzpr_selector(), "href", "pzpr")?
            .ok_or("cannot find pzpr url")?;
        let pzpr_prefix = "http://pzv.jp/p.html?lightup/";

        if !pzpr_url.starts_with(pzpr_prefix) {
            return Err(format!("unexpected pzpr url: {pzpr_url}"));
        }

        PzprStr::from_str(&pzpr_url[pzpr_prefix.len()..])
    }

    /// Extract the difficulty of a puzzle from the HTML of a puzzle entry.
    fn get_difficulty(&self, html: &Html) -> Result<Difficulty, String>;
}

/// Get a list of all implemented scrapers
pub fn implemented_scrapers() -> [(Box<dyn Scraper>, HTMLDownloader); 2] {
    [
        (Box::new(TibisukemaruScraper), HTMLDownloader::new()),
        (Box::new(BachelorSealScraper), HTMLDownloader::new()),
    ]
}

/// Helper functions for parsing HTML with error handling.
fn parse_selector(selector: &str, label: &str) -> Result<Selector, String> {
    Selector::parse(selector).map_err(|e| format!("cannot create {label} selector: {e}"))
}

/// Gets the first attribute of the first element matching a selector, if it exists.
fn first_attr<'a>(
    html: &'a Html,
    selector: &str,
    attr: &str,
    label: &str,
) -> Result<Option<&'a str>, String> {
    let selector = parse_selector(selector, label)?;
    Ok(html
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr(attr)))
}

/// Gets the first attribute of the first element matching a selector, if it exists. Returns an error if the element doesn't exist.
fn first_attr_required<'a>(
    html: &'a Html,
    selector: &str,
    attr: &str,
    label: &str,
) -> Result<Option<&'a str>, String> {
    let selector = parse_selector(selector, label)?;
    let element = html
        .select(&selector)
        .next()
        .ok_or_else(|| format!("cannot find {label} element"))?;
    Ok(element.value().attr(attr))
}

/// Gets the specified attribute of all elements matching a selector.
fn collect_attrs(
    html: &Html,
    selector: &str,
    attr: &str,
    label: &str,
) -> Result<Vec<String>, String> {
    let selector = parse_selector(selector, label)?;
    Ok(html
        .select(&selector)
        .filter_map(|el| el.value().attr(attr).map(ToOwned::to_owned))
        .collect())
}
