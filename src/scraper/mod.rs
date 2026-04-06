pub mod bachelor_seal;
pub mod tibisukemaru;

use std::borrow::ToOwned;

use async_trait::async_trait;
use scraper::{ElementRef, Html, Selector};

use crate::disk;
use crate::html::HTMLDownloader;
use crate::model;

/// A "wrapper" around a Result so we can also encode the case where a puzzle is already saved.
pub enum ScrapeResult {
    /// The puzzle is already saved at the given path.
    IsSavedAt(String),
    // Fetching puzzle is Ok
    Ok(model::Puzzle),
    /// Fetching puzzle is Err
    Err(String),
}

/// A trait for scraping akari puzzles from a variety of sources.
/// Certain sources will be paginated, others use javascript.
///
/// This trait should be implemented for _all_ scrapers.
#[async_trait]
pub trait Scraper: Send + Sync {
    /// A human readable name for the scaper.
    fn name(&self) -> &'static str;

    /// Download a URL and return the HTML.
    ///
    /// # Errors
    /// Returns an error if the URL cannot be fetched, or if the status code is not success.
    async fn download(&self, downloader: &HTMLDownloader<'_>, url: &str) -> Result<Html, String> {
        downloader
            .download(url)
            .await
            .map(|html_str| Html::parse_document(&html_str))
    }

    /// Gets all new puzzles for this scraper.
    ///
    /// # Errors
    /// Returns an error if we cannot fetch the necessary URLs, if the HTML cannot be parsed, or if the puzzles cannot be extracted from the HTML.
    async fn fetch_puzzles(
        &self,
        downloader: &HTMLDownloader,
        cfg: &disk::Cfg,
    ) -> Result<Vec<ScrapeResult>, String>;
}

/// A trait for scraping single akari puzzles.
/// This is only used when we can't otherwise easily extract puzzles from a paginated archive, or when explicitly requested from CLI arguments.
#[async_trait]
pub trait SinglePuzzleScraper: Scraper {
    /// CSS selectors for difficulty element to be used in `get_difficulty`.
    fn difficulty_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for pzpr element to be used in `get_pzpr`.
    fn pzpr_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for puzzle number element to be used in `get_puzzle_no`.
    fn puzzle_no_selector(&self) -> Vec<&'static str>;

    /// Gets the difficulty of a puzzle from the document.
    ///
    /// # Errors
    /// Returns an error if the difficulty element cannot be found, or if the difficulty cannot be parsed from the element.
    fn get_difficulty(&self, document: &Html) -> Result<model::Difficulty, String>;

    /// Gets the pzpr string of a puzzle from the document.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found, or if the pzpr string cannot be parsed from the element.
    fn get_pzpr(&self, document: &Html) -> Result<String, String>;

    /// Gets the puzzle number of a puzzle from the document.
    ///
    /// # Errors
    /// Returns an error if the puzzle number element cannot be found, or if the puzzle number cannot be parsed from the element.
    fn get_puzzle_no(&self, document: &Html) -> Result<String, String>;

    /// Fetches a single puzzle from some URL.
    ///
    /// # Errors
    /// Returns an error if we cannot fetch the URL, if the HTML cannot be parsed, or if any of the difficulty, pzpr, or puzzle number cannot be extracted from the HTML.
    async fn fetch_single(
        &self,
        url: &str,
        downloader: &HTMLDownloader<'_>,
        cfg: &disk::Cfg,
    ) -> ScrapeResult {
        if let Some(path) = cfg.cache.get(url) {
            return ScrapeResult::IsSavedAt(path.to_string());
        }

        let get = async |url| {
            let document = self.download(downloader, url).await?;
            let difficulty = self.get_difficulty(&document)?;
            let pzpr = self.get_pzpr(&document)?;
            let number = self.get_puzzle_no(&document)?;
            let domain_name = self.name().to_string();

            Ok(model::Puzzle {
                domain_name,
                difficulty,
                number,
                at: chrono::Utc::now(),
                from_url: url.to_string(),
                pzpr,
            })
        };

        match get(url).await {
            Ok(puzzle) => ScrapeResult::Ok(puzzle),
            Err(e) => ScrapeResult::Err(e),
        }
    }
}

// A trait for scraping akari puzzles from paginated archives
#[async_trait]
pub trait PaginatedScraper: Scraper {
    /// CSS selectors for puzzle entry element to be used in `get_entries`.
    fn entry_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for difficulty element within an entry to be used in `get_entry_difficulty`.
    fn entry_difficulty_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for pzpr element within an entry to be used in `get_entry_pzpr`.
    fn entry_pzpr_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for puzzle number element within an entry to be used in `get_entry_puzzle_no`.
    fn entry_puzzle_no_selector(&self) -> Vec<&'static str>;

    /// CSS selectors for URL of puzzle, so we can save it to cache and check if we already have it.
    /// To be used in `get_entry_as_url`.
    fn entry_as_url_selector(&self) -> Vec<&'static str>;

    /// Gets the difficulty of a puzzle from an entry element.
    ///
    /// # Errors
    /// Returns an error if the difficulty element cannot be found within the entry, or if the difficulty cannot be parsed from the element.
    fn get_entry_difficulty(&self, entry_el: ElementRef) -> Result<model::Difficulty, String>;

    /// Gets the pzpr string of a puzzle from an entry element.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found within the entry, or if the pzpr string cannot be parsed from the element.
    fn get_entry_pzpr(&self, entry_el: ElementRef) -> Result<String, String>;

    /// Gets the puzzle number of a puzzle from an entry element.
    ///
    /// # Errors
    /// Returns an error if the puzzle number element cannot be found within the entry, or if the puzzle number cannot be parsed from the element.  
    fn get_entry_puzzle_no(&self, entry_el: ElementRef) -> Result<String, String>;

    /// Gets the url of a puzzle from an entry element.
    ///
    /// # Errors
    /// Returns an error if the url element cannot be found within the entry, or if the url cannot be extracted from the element.
    fn get_entry_as_url(&self, entry_el: ElementRef) -> Result<String, String> {
        first_attr_required(
            &entry_el,
            self.entry_as_url_selector().as_slice(),
            "href",
            "entry_as_url",
        )
        .map(ToOwned::to_owned)
    }

    /// CSS selectors for the "next page" element to be used in `get_next_page_url`.
    fn next_page_selector(&self) -> Vec<&'static str>;

    /// The URL of the first page of the archive.
    fn first_url(&self) -> &'static str;

    /// Gets the URL of the next page from the document of a page.
    ///
    /// # Errors
    /// Returns an error if the next page element cannot be found, if the next page URL cannot be extracted from the element, or if the selector is invalid.
    fn get_next_page_url<'a>(&self, html: &'a Html) -> Result<Option<&'a str>, String> {
        first_attr(
            html,
            self.next_page_selector().as_slice(),
            "href",
            "next_page",
        )
    }

    /// Get all entries from a page given the document of the page.
    ///
    /// # Errors
    /// Returns an error if the entry elements cannot be found, or if the selector is invalid.
    fn get_entries<'a>(&self, document: &'a Html) -> Result<Vec<ElementRef<'a>>, String> {
        try_selectors(self.entry_selector().as_slice(), "entry", |selector| {
            let selector = parse_selector(selector, "entry")?;
            let entries = document.select(&selector).collect::<Vec<_>>();
            if entries.is_empty() {
                Err("no entry elements matched selector".to_string())
            } else {
                Ok(entries)
            }
        })
    }

    /// Fetches all puzzles from the paginated archive.
    ///
    /// # Errors
    /// Returns an error if we cannot fetch any of the necessary URLs, if any of the HTML documents cannot be parsed, if the puzzles cannot be extracted from any of the pages, or if the next page URL cannot be extracted from any of the pages.
    async fn fetch_puzzles(
        &self,
        downloader: &HTMLDownloader,
        cfg: &disk::Cfg,
    ) -> Result<Vec<ScrapeResult>, String> {
        let mut results = Vec::new();
        let mut current = self.first_url().to_owned();
        'outer: loop {
            let document = self.download(downloader, &current).await?;
            let next_page_url = self.get_next_page_url(&document)?.map(ToOwned::to_owned);
            let this_page_puzzles = self.extract_puzzles_from_page(&document, cfg)?;

            for scrape_result in this_page_puzzles {
                if let ScrapeResult::IsSavedAt(path) = &scrape_result {
                    eprintln!(
                        "Some puzzle from '{current}' is already saved: '{path}'. Stopping pagination."
                    );
                    break 'outer;
                }

                results.push(scrape_result);
            }

            match next_page_url {
                Some(next_page_url) => current = next_page_url,
                None => break,
            }
        }

        Ok(results)
    }

    /// Extracts puzzles from a page given the document and URL of the page.
    ///
    /// # Errors
    /// Returns an error if any of the puzzles cannot be extracted from the page.
    fn extract_puzzles_from_page(
        &self,
        document: &Html,
        cfg: &disk::Cfg,
    ) -> Result<Vec<ScrapeResult>, String> {
        let entries = self.get_entries(document)?;
        Ok(entries
            .into_iter()
            .map(|entry_el| {
                let entry_url = match self.get_entry_as_url(entry_el) {
                    Ok(url) => url,
                    Err(e) => return ScrapeResult::Err(format!("cannot get entry url: {e}")),
                };

                if let Some(path) = cfg.cache.get(&entry_url) {
                    return ScrapeResult::IsSavedAt(path.to_string());
                }

                let get = |entry_el| {
                    let difficulty = self.get_entry_difficulty(entry_el)?;
                    let pzpr = self.get_entry_pzpr(entry_el)?;
                    let number = self.get_entry_puzzle_no(entry_el)?;

                    let domain_name = self.name().to_string();

                    Ok(model::Puzzle {
                        domain_name,
                        difficulty,
                        number,
                        at: chrono::Utc::now(),
                        from_url: entry_url.to_string(),
                        pzpr,
                    })
                };

                match get(entry_el) {
                    Ok(puzzle) => ScrapeResult::Ok(puzzle),
                    Err(e) => ScrapeResult::Err(e),
                }
            })
            .collect())
    }
}

// NOTE: This must always be sorted alphabetically by name.
/// A static map of scraper names to their base URLs and puzzle URL formats.
///
/// The map is sorted by name, so that we can use binary search to access it.
/// If new scrapers are added, they should modify this map.
///
/// The "we have a static map at home" stores: `name` -> (`base_url`, `puzzle_url`).
///
/// The `base_url` is used to match user-provided puzzle URLs to scrapers.
/// The `puzzle_url` is mostly used to communicate valid puzzle URLs to the user. We hijack it in implementation for easy extraction to the puzzle number.
pub static SCRAPER_INFO: &[(&str, (&str, &str))] = &[
    (
        bachelor_seal::ID,
        (
            "http://blog.livedoor.jp/bachelor_seal-puzzle/",
            "http://blog.livedoor.jp/bachelor_seal-puzzle/archives/{{puzzle_no}}.html",
        ),
    ),
    (
        tibisukemaru::ID,
        (
            "http://tibisukemaru.blog.fc2.com/",
            "http://tibisukemaru.blog.fc2.com/blog-entry-{{puzzle_no}}.html",
        ),
    ),
];

/// Gets the base URL and puzzle URL format for a scraper by its name.
#[must_use]
pub fn info_for_name(name: &str) -> Option<(&'static str, &'static str)> {
    // eprintln!("Getting scraper info for domain '{name}'...");
    SCRAPER_INFO
        .binary_search_by(|(k, _)| k.cmp(&name))
        .ok()
        .map(|x| SCRAPER_INFO[x].1)
}

/// Gets the names of all scrapers.
#[must_use]
pub fn names() -> Vec<&'static str> {
    SCRAPER_INFO
        .iter()
        .map(|(name, (_base_url, _puzzle_url))| *name)
        .collect()
}

/// Gets the base URLs of all scrapers.
#[must_use]
pub fn base_urls() -> Vec<&'static str> {
    SCRAPER_INFO
        .iter()
        .map(|(_name, (base_url, _puzzle_url))| *base_url)
        .collect()
}

/// Gets the puzzle URL format for a scraper by its name.
///
/// Returns `None` if no scraper with the given name exists.
#[must_use]
pub fn puzzle_url_for(domain: &str) -> Option<&'static str> {
    // eprintln!("Getting puzzle url format for domain '{domain}'...");
    let (_, puzzle_url) = info_for_name(domain)?;
    Some(puzzle_url)
}

#[must_use]
/// Gets the base URL for a scraper by its name.
///
/// Returns `None` if no scraper with the given name exists.
pub fn base_url_for(domain: &str) -> Option<&'static str> {
    let (base_url, _) = info_for_name(domain)?;
    Some(base_url)
}

/// Checks if a URL is valid for a scraper by its name.
///
/// # Errors
/// Returns an error if no scraper with the given name exists, or if the URL does not match the expected format for the scraper.
pub fn url_ok_for(name: &str, url: &str) -> Result<bool, String> {
    const SENTINEL: &str = "{{puzzle_no}}";

    let url_fmt = puzzle_url_for(name)
        .ok_or_else(|| format!("no puzzle url format found for domain '{name}'"))?;
    let url_start = url_fmt.split(SENTINEL).next().unwrap_or("");
    let url_end = url_fmt.split(SENTINEL).nth(1).unwrap_or("");

    Ok(url.starts_with(url_start) && url.ends_with(url_end))
}

#[must_use]
pub fn for_name<'a>(domain: &str) -> Option<Box<dyn Scraper + 'a>> {
    match domain {
        "tibisukemaru" => Some(Box::new(tibisukemaru::TibisukemaruScraper)),
        "bachelor_seal" => Some(Box::new(bachelor_seal::BachelorSealScraper)),
        _ => None,
    }
}

#[must_use]
pub fn for_name_as_single<'a>(domain: &str) -> Option<Box<dyn SinglePuzzleScraper + 'a>> {
    match domain {
        "tibisukemaru" => Some(Box::new(tibisukemaru::TibisukemaruScraper)),
        "bachelor_seal" => Some(Box::new(bachelor_seal::BachelorSealScraper)),
        _ => None,
    }
}

/// The `scraper` crate defines `.select()` methods on `Html` and `ElementRef` separately.
/// I want to use it as a unified "select stuff from some HTML fragment".
pub(crate) trait Selectable {
    type Iter<'a, 'b>: Iterator<Item = ElementRef<'a>>
    where
        Self: 'a;
    fn get<'a, 'b>(&'a self, selector: &'b Selector) -> Result<Self::Iter<'a, 'b>, String>;
}

impl Selectable for Html {
    type Iter<'a, 'b> = scraper::html::Select<'a, 'b>;
    fn get<'a, 'b>(&'a self, selector: &'b Selector) -> Result<Self::Iter<'a, 'b>, String> {
        Ok(self.select(selector))
    }
}

impl Selectable for ElementRef<'_> {
    type Iter<'a, 'b>
        = scraper::element_ref::Select<'a, 'b>
    where
        Self: 'a;
    fn get<'a, 'b>(&'a self, selector: &'b Selector) -> Result<Self::Iter<'a, 'b>, String> {
        Ok(self.select(selector))
    }
}

/// Helper functions for parsing HTML with error handling.
fn parse_selector(selector: &str, label: &str) -> Result<Selector, String> {
    Selector::parse(selector).map_err(|e| format!("cannot create selector for {label}: {e}"))
}

fn try_selectors<T, F>(selectors: &[&str], label: &str, mut attempt: F) -> Result<T, String>
where
    F: FnMut(&str) -> Result<T, String>,
{
    if selectors.is_empty() {
        return Err(format!("no selectors configured for {label}"));
    }

    let mut errors = Vec::new();
    for selector in selectors {
        match attempt(selector) {
            Ok(value) => return Ok(value),
            Err(error) => errors.push(format!("'{selector}': {error}")),
        }
    }

    Err(format!(
        "cannot get {label} with configured selectors: {}",
        errors.join("; ")
    ))
}

/// Gets the first attribute of the first element matching a selector, if it exists.
fn first_attr<'a, S: Selectable>(
    selectable: &'a S,
    selectors: &[&str],
    attr: &str,
    label: &str,
) -> Result<Option<&'a str>, String> {
    for selector in selectors {
        let selector = parse_selector(selector, label)?;
        if let Some(value) = selectable
            .get(&selector)?
            .next()
            .and_then(|el| el.value().attr(attr))
        {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// Gets the first attribute of the first element matching a selector, if it exists. Returns an error if the element doesn't exist.
fn first_attr_required<'a, S: Selectable>(
    selectable: &'a S,
    selectors: &[&str],
    attr: &str,
    label: &str,
) -> Result<&'a str, String> {
    try_selectors(selectors, label, |selector| {
        let selector = parse_selector(selector, label)?;
        let element = selectable
            .get(&selector)?
            .next()
            .ok_or_else(|| format!("cannot find {label} element"))?;

        element
            .value()
            .attr(attr)
            .ok_or_else(|| format!("{label} element found but does not have {attr} attribute"))
    })
}

fn inner_html_required<S: Selectable>(
    selectable: &S,
    selectors: &[&str],
    label: &str,
) -> Result<String, String> {
    try_selectors(selectors, label, |selector| {
        let selector = parse_selector(selector, label)?;
        let element = selectable
            .get(&selector)?
            .next()
            .ok_or_else(|| format!("cannot find {label} element"))?;

        Ok(element.inner_html())
    })
}

fn text_required<S: Selectable>(
    selectable: &S,
    selectors: &[&str],
    label: &str,
) -> Result<String, String> {
    try_selectors(selectors, label, |selector| {
        let selector = parse_selector(selector, label)?;
        let element = selectable
            .get(&selector)?
            .next()
            .ok_or_else(|| format!("cannot find {label} element"))?;

        Ok(element.text().collect::<String>())
    })
}

pub(crate) fn pzpr_from_url(url: &str) -> Result<String, String> {
    let possible_prefixes = [
        "http://pzv.jp/p.html?lightup/",
        "https://pzv.jp/p.html?lightup/",
        "http://pzv.jp/p.html?akari/",
        "https://pzv.jp/p.html?akari/",
        "http://puzz.link/p?lightup/",
        "https://puzz.link/p?lightup/",
        "http://puzz.link/p?akari/",
        "https://puzz.link/p?akari/",
        "http://puzz.link/p.html?lightup/",
        "https://puzz.link/p.html?lightup/",
        "http://puzz.link/p.html?akari/",
        "https://puzz.link/p.html?akari/",
    ];

    let pzpr_prefix = possible_prefixes
        .iter()
        .find(|prefix| url.starts_with(*prefix))
        .ok_or_else(|| format!("pzpr url '{url}' does not start with any expected prefix"))?;

    let stripped = url.strip_prefix(pzpr_prefix).ok_or_else(|| {
        format!("cannot strip expected prefix '{pzpr_prefix}' from pzpr url '{url}'")
    })?;

    Ok(stripped.to_string())
}

/// Extract the pzpr URL from the HTML of a puzzle entry, and convert it to a `PzprStr`.
/// Optionally, just provide the URL directly.
///
/// # Errors
/// Returns an error if the pzpr URL cannot be extracted, if the selector is invalid, if the pzpr URL does not have the expected format, or if the pzpr string cannot be parsed.
pub(crate) fn pzpr_from_el<S: Selectable>(
    selectable: &S,
    selectors: &[&str],
) -> Result<String, String> {
    try_selectors(selectors, "pzpr", |selector| {
        let pzpr_url = first_attr_required(selectable, &[selector], "href", "pzpr")?;
        pzpr_from_url(pzpr_url)
    })
}
