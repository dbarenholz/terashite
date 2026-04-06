use std::str::FromStr;

use async_trait::async_trait;

use crate::disk;
use crate::html;
use crate::model;
use crate::scraper;

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
    ) -> Result<Vec<super::ScrapeResult>, String> {
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

    fn puzzle_no_from_text(text: &str) -> Result<String, String> {
        let puzzle_no = text
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
    fn difficulty_selector(&self) -> Vec<&'static str> {
        vec!["head > title"]
    }

    fn pzpr_selector(&self) -> Vec<&'static str> {
        vec!["a[title='☆ぱずぷれへ☆']"]
    }

    // This happens to be the same as the difficulty selector, so we can just reuse that.
    fn puzzle_no_selector(&self) -> Vec<&'static str> {
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
        let inner_html = super::inner_html_required(
            document,
            self.difficulty_selector().as_slice(),
            "difficulty",
        )?;
        Self::difficulty_from_inner_html(&inner_html)
    }

    /// Gets the pzpr string of a puzzle from the document.
    /// Since the pzpr string is in <a>, we can use the `pzpr_from_a` helper function.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found, or if the pzpr string cannot be parsed from the element.
    fn get_pzpr(&self, document: &scraper::Html) -> Result<String, String> {
        // calls try_selectors() internally
        super::pzpr_from_el(document, self.pzpr_selector().as_slice())
    }

    /// Gets the puzzle number of a puzzle from the document.
    /// For tibisukemaru, the puzzle number can be found by keeping all digits from the title.
    ///
    /// # Errors
    /// Returns an error if the puzzle number element cannot be found, or if the puzzle number cannot be parsed from the title.
    fn get_puzzle_no(&self, document: &scraper::Html) -> Result<String, String> {
        // calls try_selectors() internally
        let text =
            super::text_required(document, self.puzzle_no_selector().as_slice(), "puzzle_no")?;

        Self::puzzle_no_from_text(&text)
    }
}

#[async_trait]
impl super::PaginatedScraper for TibisukemaruScraper {
    fn next_page_selector(&self) -> Vec<&'static str> {
        vec!["div.link > p a:last-child"]
    }

    fn first_url(&self) -> &'static str {
        "http://tibisukemaru.blog.fc2.com/blog-category-14.html"
    }

    /// Tibisukemaru links back to Home on the last page; we don't want to go home.
    fn get_next_page_url<'a>(&self, html: &'a scraper::Html) -> Result<Option<&'a str>, String> {
        let url = super::first_attr(
            html,
            self.next_page_selector().as_slice(),
            "href",
            "next_page",
        )?;
        Ok(url.filter(|u| *u != "http://tibisukemaru.blog.fc2.com/"))
    }

    fn entry_selector(&self) -> Vec<&'static str> {
        vec!["div.entry"]
    }

    fn entry_difficulty_selector(&self) -> Vec<&'static str> {
        vec!["div.tit > h2"]
    }

    fn entry_pzpr_selector(&self) -> Vec<&'static str> {
        // note: the `*=` is very important due to html changes over time -- with `~=` things break horribly
        vec!["a[title*='ぱずぷれへ']"]
    }

    fn entry_puzzle_no_selector(&self) -> Vec<&'static str> {
        self.entry_difficulty_selector()
    }

    fn entry_as_url_selector(&self) -> Vec<&'static str> {
        vec!["div.date > p > a"]
    }

    fn get_entry_pzpr(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // calls try_selectors() internally
        super::pzpr_from_el(&entry_el, self.entry_pzpr_selector().as_slice())
    }

    fn get_entry_difficulty(
        &self,
        entry_el: scraper::ElementRef,
    ) -> Result<model::Difficulty, String> {
        let res = super::inner_html_required(
            &entry_el,
            self.entry_difficulty_selector().as_slice(),
            "difficulty",
        )?;
        Self::difficulty_from_inner_html(&res)
    }

    fn get_entry_puzzle_no(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // calls try_selectors() internally
        let text = super::text_required(
            &entry_el,
            self.entry_puzzle_no_selector().as_slice(),
            "puzzle_no",
        )?;

        Self::puzzle_no_from_text(&text)
    }
}
