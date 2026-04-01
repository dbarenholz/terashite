use std::{fmt::Display, path::PathBuf, str::FromStr};

use chrono::{DateTime, Utc};

pub enum Difficulty {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
    Unknown,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VeryEasy => write!(f, "very-easy"),
            Self::Easy => write!(f, "easy"),
            Self::Medium => write!(f, "medium"),
            Self::Hard => write!(f, "hard"),
            Self::VeryHard => write!(f, "very-hard"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl FromStr for Difficulty {
    type Err = String;

    // NOTE: we want to match same arms as we split this up by domain.
    #[allow(clippy::match_same_arms)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // tibisukemaru
            "ナカナカ" => Ok(Self::Hard),
            "フツウ" => Ok(Self::Medium),
            "カンタン" => Ok(Self::Easy),
            // bachelor-seal
            "☆" => Ok(Self::VeryEasy),
            "☆☆" => Ok(Self::Easy),
            "☆☆☆" => Ok(Self::Medium),
            "☆☆☆☆" => Ok(Self::Hard),
            "☆☆☆☆☆" => Ok(Self::VeryHard),

            // "very-easy" => Ok(Self::VeryEasy),
            // "easy" => Ok(Self::Easy),
            // "medium" => Ok(Self::Medium),
            // "hard" => Ok(Self::Hard),
            // "very-hard" => Ok(Self::VeryHard),
            // "unknown" => Ok(Self::Unknown),
            _ => Err(format!("invalid difficulty string: {s}")),
        }
    }
}

// fn parse_pzpr(s: &str) -> Result<String, String> {
//     // Extract width
//     if let Some((w, rest)) = s.split_once('/') {
//         let width = w
//             .parse::<usize>()
//             .map_err(|e| format!("cannot parse width: {e}"))?;

//         // Extract height and cells
//         if let Some((h, cells)) = rest.split_once('/') {
//             let height = h
//                 .parse::<usize>()
//                 .map_err(|e| format!("cannot parse height: {e}"))?;

//             return Ok(Self {
//                 width,
//                 height,
//                 cells: cells.to_owned(),
//             });
//         }
//     }

//     Err(format!("invalid pzpr string: {s}"))
// }

pub struct Puzzle {
    pub(crate) pzpr: String,
    pub(crate) difficulty: Difficulty,
    pub(crate) at: DateTime<Utc>,
    pub(crate) from_url: String,
    pub(crate) domain_name: String,
    pub(crate) number: String,
}

impl Puzzle {
    /// Gets the filename that this puzzle should be saved to.
    pub(crate) fn filename(&self) -> PathBuf {
        let subdir = &self.domain_name;
        let name = format!("{}-{}.pzprc", self.number, self.difficulty);

        let mut path = PathBuf::from(subdir);
        path.push(name);
        path
    }

    /// Gets the content that should be saved for this puzzle.
    pub(crate) fn content(&self) -> String {
        format!(
            "domain_name: {}\ndifficulty: {}\npuzzle_no: {}\nat: {}\nfrom_url: {}\n{}",
            self.domain_name,
            self.difficulty,
            self.number,
            self.at.to_rfc3339(),
            self.from_url,
            self.pzpr,
        )
    }
}
