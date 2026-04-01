use std::str::FromStr;

use async_trait::async_trait;

use crate::disk;
use crate::html;
use crate::model;
use crate::scraper;

pub(crate) static ID: &str = "bachelor_seal";

pub struct BachelorSealScraper;

#[async_trait]
impl super::Scraper for BachelorSealScraper {
    fn name(&self) -> &'static str {
        ID
    }

    async fn fetch_puzzles(
        &self,
        downloader: &html::HTMLDownloader,
        cfg: &disk::Cfg,
    ) -> Result<Vec<model::Puzzle>, String> {
        super::PaginatedScraper::fetch_puzzles(self, downloader, cfg).await
    }
}

impl BachelorSealScraper {
    fn difficulty_from_inner_html(inner_html: &str) -> Result<model::Difficulty, String> {
        let difficulty_str = inner_html
            .trim()
            .chars()
            .filter(|c| c == &'☆')
            .collect::<String>();

        model::Difficulty::from_str(&difficulty_str)
    }

    fn puzzle_no_from_inner_html(inner_html: &str) -> Result<String, String> {
        let puzzle_no = inner_html
            .chars()
            .filter(char::is_ascii_digit)
            .collect::<String>();

        if puzzle_no.is_empty() {
            Err("could not parse puzzle number from title".to_string())
        } else {
            Ok(puzzle_no)
        }
    }
}

#[async_trait]
impl super::SinglePuzzleScraper for BachelorSealScraper {
    fn difficulty_selector(&self) -> &'static str {
        "div.article-body-inner"
    }

    fn pzpr_selector(&self) -> &'static str {
        "a[title=ぱずぷれで遊ぶ。]"
    }

    fn puzzle_no_selector(&self) -> &'static str {
        "h2.entry-title > a"
    }

    /// Gets the difficulty of a puzzle from the document.
    /// For bachelor seal, the difficulty is stuck in a big blob of text.
    /// Luckily, if we just count the number of stars, we can get the difficulty quite easily.
    ///
    /// # Errors
    /// Returns an error if the difficulty element cannot be found, or if the difficulty cannot be parsed from the title.
    fn get_difficulty(&self, document: &scraper::Html) -> Result<model::Difficulty, String> {
        let selector = super::parse_selector(self.difficulty_selector(), "difficulty")?;
        let untrimmed_text = document
            .select(&selector)
            .next()
            .ok_or_else(|| "could not find difficulty element".to_string())?
            .inner_html();
        Self::difficulty_from_inner_html(&untrimmed_text)
    }

    /// Gets the pzpr string of a puzzle from the document.
    /// Since the pzpr string is in <a>, we can use the `pzpr_from_a` helper function.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found, or if the pzpr string cannot be parsed from the element.
    fn get_pzpr(&self, html: &scraper::Html) -> Result<String, String> {
        super::pzpr_from_selectable(html, self.pzpr_selector())
    }

    fn get_puzzle_no(&self, document: &scraper::Html) -> Result<String, String> {
        let selector = super::parse_selector(self.puzzle_no_selector(), "puzzle_no")?;
        let inner_html = document
            .select(&selector)
            .next()
            .ok_or_else(|| "could not find puzzle no element".to_string())?
            .inner_html();

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}

#[async_trait]
impl super::PaginatedScraper for BachelorSealScraper {
    fn next_page_selector(&self) -> &'static str {
        "a[rel=next]"
    }

    fn first_url(&self) -> &'static str {
        "http://blog.livedoor.jp/bachelor_seal-puzzle/archives/cat_531435.html"
    }

    fn entry_selector(&self) -> &'static str {
        "div.article-outer-3"
    }

    fn entry_difficulty_selector(&self) -> &'static str {
        "div.article-body-inner"
    }

    fn entry_pzpr_selector(&self) -> &'static str {
        "a[title='ぱずぷれで遊ぶ。']"
    }

    // get inner html, strip non-digits, done
    fn entry_puzzle_no_selector(&self) -> &'static str {
        "h2.entry-title > a"
    }

    fn get_entry_pzpr(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        super::pzpr_from_selectable(&entry_el, self.entry_pzpr_selector())
    }

    fn get_entry_difficulty(
        &self,
        entry_el: scraper::ElementRef,
    ) -> Result<model::Difficulty, String> {
        let selector = super::parse_selector(self.entry_difficulty_selector(), "difficulty")?;
        let inner_html = entry_el
            .select(&selector)
            .next()
            .ok_or_else(|| "could not find difficulty element".to_string())?
            .inner_html();

        Self::difficulty_from_inner_html(&inner_html)
    }

    fn get_entry_puzzle_no(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // select the element, get inner html, strip non-digits, done
        let selector = super::parse_selector(self.entry_puzzle_no_selector(), "puzzle_no")?;
        let inner_html = entry_el
            .select(&selector)
            .next()
            .ok_or_else(|| "could not find puzzle no element".to_string())?
            .inner_html();

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}
