use std::str::FromStr;

use async_trait::async_trait;

use crate::disk;
use crate::html;
use crate::model;
use crate::scraper;
use crate::scraper::parse_selector;
use crate::scraper::try_selectors;

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
    ) -> Result<Vec<super::ScrapeResult>, String> {
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
        // since bachelor_seal uses a Japanese font, and thus full-width digits, we convert them so filenames are as expected
        let fix_digits = |c| match c {
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            _ => c,
        };

        let puzzle_no = inner_html
            .chars()
            .map(|c| c.to_owned())
            .map(fix_digits)
            .filter(|c| c.is_ascii_digit())
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
    fn difficulty_selector(&self) -> Vec<&'static str> {
        vec!["div.article-body-inner"]
    }

    fn pzpr_selector(&self) -> Vec<&'static str> {
        vec!["a[title=ぱずぷれで遊ぶ。]"]
    }

    fn puzzle_no_selector(&self) -> Vec<&'static str> {
        vec!["h2.entry-title > a"]
    }

    /// Gets the difficulty of a puzzle from the document.
    /// For bachelor seal, the difficulty is stuck in a big blob of text.
    /// Luckily, if we just count the number of stars, we can get the difficulty quite easily.
    ///
    /// # Errors
    /// Returns an error if the difficulty element cannot be found, or if the difficulty cannot be parsed from the title.
    fn get_difficulty(&self, document: &scraper::Html) -> Result<model::Difficulty, String> {
        let untrimmed_text = super::inner_html_required(
            document,
            self.difficulty_selector().as_slice(),
            "difficulty",
        )?;
        Self::difficulty_from_inner_html(&untrimmed_text)
    }

    /// Gets the pzpr string of a puzzle from the document.
    /// Since the pzpr string is in <a>, we can use the `pzpr_from_a` helper function.
    ///
    /// # Errors
    /// Returns an error if the pzpr element cannot be found, or if the pzpr string cannot be parsed from the element.
    fn get_pzpr(&self, html: &scraper::Html) -> Result<String, String> {
        super::pzpr_from_el(html, self.pzpr_selector().as_slice())
    }

    fn get_puzzle_no(&self, document: &scraper::Html) -> Result<String, String> {
        let inner_html = super::inner_html_required(
            document,
            self.puzzle_no_selector().as_slice(),
            "puzzle_no",
        )?;

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}

#[async_trait]
impl super::PaginatedScraper for BachelorSealScraper {
    fn next_page_selector(&self) -> Vec<&'static str> {
        vec!["a[rel=next]"]
    }

    fn first_url(&self) -> &'static str {
        "http://blog.livedoor.jp/bachelor_seal-puzzle/archives/cat_531435.html"
    }

    fn entry_selector(&self) -> Vec<&'static str> {
        vec!["div.article-outer-3"]
    }

    fn entry_difficulty_selector(&self) -> Vec<&'static str> {
        vec!["div.article-body-inner", ".article-category-second > a"]
    }

    fn entry_pzpr_selector(&self) -> Vec<&'static str> {
        // bachelor seal makes us sad.
        // - earlier puzzles have <a title=...>. Later ones don't add it to HTML.
        // - the <a href=...> starts on http://pzv.jp, but later switches to https://puzz.link.
        // - when selecting on href, we also get an "overview" post with many many links -- these need to be filtered out!
        vec![
            "a[title=ぱずぷれで遊ぶ。]", // earliest puzzles, up to 32
            "a[href*='pzv.jp']",         // most puzzles, up to 327
            "a[href*='puzz.link']",      // latest puzzles (so far)
        ]
    }

    // get inner html, strip non-digits, done
    fn entry_puzzle_no_selector(&self) -> Vec<&'static str> {
        vec!["h2.entry-title > a"]
    }

    // same element, get href
    fn entry_as_url_selector(&self) -> Vec<&'static str> {
        self.entry_puzzle_no_selector()
    }

    fn get_entry_pzpr(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // the link text can be one of these, since we don't want to get the links from overview posts
        let valid_link_text = vec!["puzz.linkで遊ぶ。", "ぱずぷれで遊ぶ。"];

        // try each selector in order to get pzpr
        let url = try_selectors(
            self.entry_pzpr_selector().as_slice(),
            "pzpr",
            |selector_str| {
                // get the actual selector (scraper::Selector))
                let selector = parse_selector(selector_str, " pzpr selector")?;
                // select from the entry
                let el = entry_el
                    .select(&selector)
                    // keep only the elements with the right link text, since we get a lot of extraneous links otherwise
                    .find(|e| {
                        let text = e.text().collect::<String>();
                        valid_link_text.contains(&text.as_str())
                    })
                    // if we can't find it, then not a proper link
                    .ok_or_else(|| format!("'{selector_str}': incorrect pzpr link"))?;
                // we assume correct link now; get the href
                el.value()
                    .attr("href") // get href (should not really error because this gets called when selecting with href selector)
                    .ok_or_else(|| {
                        format!(
                            "could not find href attribute in element for selector '{selector_str}'"
                        )
                    })
                    .map(|s| s.to_string())
            },
        )?;

        super::pzpr_from_url(&url)
    }

    fn get_entry_difficulty(
        &self,
        entry_el: scraper::ElementRef,
    ) -> Result<model::Difficulty, String> {
        super::try_selectors(
            self.entry_difficulty_selector().as_slice(),
            "difficulty",
            |selector| {
                let inner_html = super::inner_html_required(&entry_el, &[selector], "difficulty")?;
                Self::difficulty_from_inner_html(&inner_html)
            },
        )
    }

    fn get_entry_puzzle_no(&self, entry_el: scraper::ElementRef) -> Result<String, String> {
        // select the element, get inner html, strip non-digits, done
        let inner_html = super::inner_html_required(
            &entry_el,
            self.entry_puzzle_no_selector().as_slice(),
            "puzzle_no",
        )?;

        Self::puzzle_no_from_inner_html(&inner_html)
    }
}
