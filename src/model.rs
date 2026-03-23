use std::str::FromStr;

use chrono::{DateTime, Utc};

pub enum Difficulty {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
    Unknown,
}

pub struct PzprStr {
    width: usize,
    height: usize,
    cells: String,
}

impl FromStr for PzprStr {
    type Err = String;

    /// Parses a pzpr string into a PzprStr struct.
    ///
    /// Assumes string is in format "w/h/cells", with no leading slash.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Extract width
        if let Some((w, rest)) = s.split_once('/') {
            let width = w
                .parse::<usize>()
                .map_err(|e| format!("cannot parse width: {e}"))?;

            // Extract height and cells
            if let Some((h, cells)) = rest.split_once('/') {
                let height = h
                    .parse::<usize>()
                    .map_err(|e| format!("cannot parse height: {e}"))?;

                return Ok(PzprStr {
                    width,
                    height,
                    cells: cells.to_owned(),
                });
            }
        }

        Err(format!("invalid pzpr string: {s}"))
    }
}

pub struct ScrapeResult {
    pzpr: PzprStr,
    difficulty: Difficulty,
    at: DateTime<Utc>,
    from_url: String,
}

impl ScrapeResult {
    pub fn new(pzpr: PzprStr, difficulty: Difficulty, from_url: String) -> Self {
        let at = Utc::now();
        Self {
            pzpr,
            difficulty,
            at,
            from_url,
        }
    }
}
