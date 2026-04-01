//! Module responsible for all operations that have to disk with disk IO.

use crate::model;
use std::{
    fs::{read_to_string, write},
    path::PathBuf,
    str::FromStr,
};

/// We expect the config file to contain (at least) a single line of the form
/// `out = path/to/dir`
pub struct Cfg {
    pub out: PathBuf,
}

impl FromStr for Cfg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out: Option<PathBuf> = None;

        for line in s.lines() {
            // skip # comments and empty lines
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            // not a comment or empty line, so we expect it to be of the form `key = value`
            let (mut k, mut v) = line
                .split_once('=')
                .ok_or_else(|| format!("Invalid line in config file: {line}"))?;
            k = k.trim();
            v = v.trim();
            // This is a match in case we want to add more config options.
            match k {
                "out" => {
                    if out.is_some() {
                        return Err("Multiple entries for 'out' field in config file".to_string());
                    }
                    out = Some(PathBuf::from(v));
                }
                _ => {
                    return Err(format!("Unknown key in config file: {k}"));
                }
            }
        }

        if out.is_none() {
            return Err("Config file must contain an 'out' field".to_string());
        }

        Ok(Self { out: out.unwrap() })
    }
}

impl Default for Cfg {
    /// If you don't specify a config file, we default to putting everything in `./terashite`.
    fn default() -> Self {
        Self {
            out: PathBuf::from("./terashite"),
        }
    }
}

/// Reads a file from disk and returns its contents as a string.
fn read_file(path: &PathBuf) -> Result<String, String> {
    read_to_string(path).map_err(|e| format!("Error reading file: {e}"))
}

/// Gets the config file by searching in XDG config and home, in that order.
#[must_use]
pub fn get_config() -> Cfg {
    let mut cfg: Option<Cfg> = None;

    // .config/terashite/terashite.conf
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("terashite");
        dir.push("terashite.conf");
        if dir.is_file()
            && let Ok(the_cfg) = read_file(&dir).and_then(|s| Cfg::from_str(&s))
        {
            cfg = Some(the_cfg);
        }
    }

    // ~/.terashite.conf
    if cfg.is_none()
        && let Some(mut dir) = dirs::home_dir()
    {
        dir.push(".terashite.conf");
        if dir.is_file()
            && let Ok(the_cfg) = read_file(&dir).and_then(|s| Cfg::from_str(&s))
        {
            cfg = Some(the_cfg);
        }
    }

    cfg.unwrap_or_default()
}

/// Saves a puzzle to disk.
///
/// # Errors
/// Returns an error if we already have a file for this puzzle, or if there was an error writing the file to disk.
pub fn save_puzzle(puzzle: &model::Puzzle, cfg: &Cfg) -> Result<String, String> {
    // construct path to save puzzle to
    let fname = puzzle.filename();
    let path = cfg.out.join(fname);

    // check if we already have the file
    if path.is_file() {
        return Err(format!("Some puzzle already saved at '{}'", path.display()));
    }

    // try to save the puzzle to disk
    let content = puzzle.content();
    write(&path, content).map_err(|e| format!("Error writing puzzle to disk: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// Checks if we have already saved a puzzle to disk.
pub(crate) fn is_already_saved(puzzle: &model::Puzzle, cfg: &Cfg) -> bool {
    let fname = puzzle.filename();
    let path = cfg.out.join(fname);
    path.is_file()
}
