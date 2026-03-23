use async_trait::async_trait;
use scraper::Selector;

use crate::{model::Difficulty, scraper::Scraper};

pub struct TibisukemaruScraper;

#[async_trait(?Send)]
impl Scraper for TibisukemaruScraper {
    fn domain_id(&self) -> &'static str {
        "tibisukemaru"
    }

    fn first_url(&self) -> &'static str {
        "http://tibisukemaru.blog.fc2.com/blog-category-14.html"
    }

    fn next_page_selector(&self) -> &'static str {
        "div.link >p a:nth-child(2)"
    }

    fn entry_selector(&self) -> &'static str {
        "div.date >p a"
    }

    fn pzpr_selector(&self) -> &'static str {
        "a[title=☆ぱずぷれへ☆]"
    }

    // TODO: split up into get_difficulty_elem with generic implementation, and then get_difficulty that impl's have to implement?
    fn get_difficulty(&self, html: &scraper::Html) -> Result<Difficulty, String> {
        let s = "div.tit > h2";
        let difficulty_selector =
            Selector::parse(s).map_err(|e| format!("cannot create difficulty selector: {e}"))?;
        let difficulty_el = html
            .select(&difficulty_selector)
            .next()
            .ok_or("cannot find difficulty element")?;
        // <h2><a id="4134" name="4134"></a>美術館 258問目 ナカナカ</h2>
        let difficulty_text = difficulty_el.text().collect::<String>();
        let sentinel = '目';
        // skip while not sentinel
        if let Some((_, diff)) = difficulty_text.split_once(sentinel) {
            // NOTE: we don't yet know which difficulties this site uses, so we print all of them and then fix the code to produce the correct enum value
            eprintln!("tibisukemaru difficulty '{diff}'");
        }
        Ok(Difficulty::Unknown)
    }
}
