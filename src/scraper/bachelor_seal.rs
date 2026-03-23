use async_trait::async_trait;
use scraper::Selector;

use crate::{model::Difficulty, scraper::Scraper};

pub struct BachelorSealScraper;

#[async_trait(?Send)]
impl Scraper for BachelorSealScraper {
    fn domain_id(&self) -> &'static str {
        "bachelor-seal"
    }

    fn first_url(&self) -> &'static str {
        "http://blog.livedoor.jp/bachelor_seal-puzzle/archives/cat_531435.html"
    }

    fn next_page_selector(&self) -> &'static str {
        "a[rel=next]"
    }

    fn entry_selector(&self) -> &'static str {
        "a[title=個別記事ページへ]"
    }

    fn pzpr_selector(&self) -> &'static str {
        "a[title=ぱずぷれで遊ぶ。]"
    }

    // TODO: split up into get_difficulty_elem with generic implementation, and then get_difficulty that impl's have to implement?
    fn get_difficulty(&self, html: &scraper::Html) -> Result<Difficulty, String> {
        let difficulty_selector = Selector::parse("div.article-body-inner")
            .map_err(|e| format!("cannot create difficulty selector: {e}"))?;
        let difficulty_el = html
            .select(&difficulty_selector)
            .next()
            .ok_or("cannot find difficulty element")?;
        let difficulty_text = difficulty_el.text().collect::<String>();
        let star_char = &'☆';
        let difficulty = match difficulty_text.chars().filter(|c| c == star_char).count() {
            1 => Difficulty::VeryEasy,
            2 => Difficulty::Easy,
            3 => Difficulty::Medium,
            4 => Difficulty::Hard,
            5 => Difficulty::VeryHard,
            _ => Difficulty::Unknown,
        };
        Ok(difficulty)
    }
}

// <a href="http://blog.livedoor.jp/bachelor_seal-puzzle/archives/91075385.html" title="個別記事ページへ" rel="bookmark">美術館　３７１</a>
