use std::str::FromStr;

use async_trait::async_trait;

use crate::disk;
use crate::html;
use crate::model;
use crate::scraper;
use crate::scraper::SinglePuzzleScraper;

pub(crate) static ID: &str = "tibisukemaru";

pub struct TibisukemaruScraper;

#[async_trait]
impl super::Scraper for TibisukemaruScraper {
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

impl TibisukemaruScraper {
    fn difficulty_from_inner_html(inner_html: &str) -> Result<model::Difficulty, String> {
        let difficulty_str = inner_html
            .split_whitespace()
            .next_back()
            .ok_or_else(|| "could not parse difficulty from title".to_string())?;

        model::Difficulty::from_str(difficulty_str)
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
impl super::SinglePuzzleScraper for TibisukemaruScraper {
    fn difficulty_selector(&self) -> &'static str {
        "head > title"
    }

    fn pzpr_selector(&self) -> &'static str {
        "body > a[title=☆ぱずぷれへ☆]"
    }

    // This happens to be the same as the difficulty selector, so we can just reuse that.
    fn puzzle_no_selector(&self) -> &'static str {
        self.difficulty_selector()
    }

    /// Gets the difficulty of a puzzle from the document.
    /// For tibisukemaru, we can easily find the difficulty in the <title> tag.
    /// It's always the last word.
    ///
    /// Example title: "チビスケ丸のパズル置き場 美術館 258問目 ナカナカ"
    ///
    /// # Errors
    /// Returns an error if the difficulty element cannot be found, or if the difficulty cannot be parsed from the title.
    fn get_difficulty(&self, document: &scraper::Html) -> Result<model::Difficulty, String> {
        let selector = super::parse_selector(self.difficulty_selector(), "difficulty")?;
        let inner_html = document
            .select(&selector)
            .next()
            .ok_or_else(|| "difficulty element not found".to_string())?
            .inner_html();

        Self::difficulty_from_inner_html(&inner_html)
    }

    /// Gets the pzpr string of a puzzle from the document.
    /// Since the pzpr string is in <a>, we can use the `pzpr_from_a` helper function.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found, or if the pzpr string cannot be parsed from the element.
    fn get_pzpr(&self, document: &scraper::Html) -> Result<String, String> {
        super::pzpr_from_selectable(document, self.pzpr_selector())
    }

    /// Gets the puzzle number of a puzzle from the document.
    /// For tibisukemaru, the puzzle number can be found by keeping all digits from the title.
    ///
    /// # Errors
    /// Returns an error if the puzzle number element cannot be found, or if the puzzle number cannot be parsed from the title.
    fn get_puzzle_no(&self, document: &scraper::Html) -> Result<String, String> {
        let selector = super::parse_selector(self.puzzle_no_selector(), "puzzle_no")?;
        let inner_html = document
            .select(&selector)
            .next()
            .ok_or_else(|| "puzzle_no element not found".to_string())?
            .inner_html();

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}

#[async_trait]
impl super::PaginatedScraper for TibisukemaruScraper {
    fn next_page_selector(&self) -> &'static str {
        "div.link >p a:nth-child(2)"
    }

    fn first_url(&self) -> &'static str {
        "http://tibisukemaru.blog.fc2.com/blog-category-14.html"
    }

    fn entry_selector(&self) -> &'static str {
        "div.entry"
    }

    fn entry_difficulty_selector(&self) -> &'static str {
        "div.tit > h2"
    }

    fn entry_pzpr_selector(&self) -> &'static str {
        "a[title='☆ぱずぷれへ☆']"
    }

    fn entry_puzzle_no_selector(&self) -> &'static str {
        self.difficulty_selector()
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
            .ok_or_else(|| "difficulty element not found".to_string())?
            .inner_html();

        Self::difficulty_from_inner_html(&inner_html)
    }

    fn get_entry_puzzle_no(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // same as difficulty, just keep digits
        let selector = super::parse_selector(self.entry_puzzle_no_selector(), "puzzle_no")?;
        let inner_html = entry_el
            .select(&selector)
            .next()
            .ok_or_else(|| "puzzle_no element not found".to_string())?
            .inner_html();

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}
