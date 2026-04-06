//! Module responsible for all operations that have to disk with disk IO.

use crate::model;
use crate::scraper;

use std::collections::HashMap;
use std::{
    fmt::Display,
    fs::{read_to_string, write},
    path::PathBuf,
    str::FromStr,
};

/// We expect the config file to contain lines of the form `key = value`.
pub struct Cfg {
    /// The output directory used for scraped puzzles.
    pub out_dir: PathBuf,
    /// The path to the cache file on disk.
    pub cache_file: PathBuf,
    /// A map of URLs to paths on disk, representing which puzzles we've already scraped and where they're saved on disk.
    pub cache: HashMap<String, String>,
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // get cache contents
        let content = self
            .cache
            .iter()
            .enumerate()
            .map(|(i, (url, path))| format!("{}: \"{url}\" -> \"{path}\"", i + 1))
            .collect::<Vec<String>>()
            .join("\n");
        write!(
            f,
            "puzzle output directory: {}\ncache file: {}\n{content}",
            self.out_dir.display(),
            self.cache_file.display()
        )
    }
}

impl FromStr for Cfg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out: Option<PathBuf> = None;
        let mut cache_file: Option<PathBuf> = None;

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
            match k {
                // out_dir stores the puzzles
                "out" => {
                    if out.is_some() {
                        return Err("Multiple entries for 'out' field in config file".to_string());
                    }
                    out = Some(PathBuf::from(v));
                }
                // cache_file stores which URLs have been scraped before, and where they're saved on disk
                // it's basically a Map<String, String> on disk, with URL as key and Path as value
                "cache" => {
                    if cache_file.is_some() {
                        return Err("Multiple entries for 'cache' field in config file".to_string());
                    }
                    cache_file = Some(PathBuf::from(v));
                }
                _ => {
                    return Err(format!("Unknown key in config file: {k}"));
                }
            }
        }

        // If none provided, use default
        if out.is_none() {
            out = Some(default_out_dir());
        }

        if cache_file.is_none() {
            cache_file = Some(default_cache_file());
        }

        let out_dir = out.unwrap();
        let cache_file = cache_file.unwrap();
        let cache = read_cache(&cache_file);

        Ok(Self {
            out_dir,
            cache,
            cache_file,
        })
    }
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("./terashite")
}

/// The default cache file is at XDG cache dir or ./cache if XDG cache dir is not available, with filename "terashite.cache".
fn default_cache_file() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("./cache"))
        .join("terashite.cache")
}

/// Reads the cache file, and returns a map of URLs to paths on disk.
///
/// The cache file is expected to be a plaintext file with lines of the form
///
/// ```text
/// "url1" -> "path1"
/// url2 -> path2
/// ```
///
/// Quotes are technically optional, but terashite writes them by default.
///
/// If at any point we encounter an error reading a particular line, we ignore that line.
/// If we can't read the file, we return an empty cache.
fn read_cache(cache_file: &PathBuf) -> HashMap<String, String> {
    let mut cache = HashMap::new();

    // if the cache file doesn't exist, we create it and return an empty cache
    if !cache_file.is_file() {
        if let Err(e) = write(&cache_file, "") {
            eprintln!(
                "Error creating cache file at '{}': {e}. Continuing with empty cache.",
                cache_file.display()
            );
        }
        return cache;
    }

    // parses a line of the form `"url" -> "path"` or `url -> path` into a (url, path) tuple.
    // returns None if the line is invalid.
    let parse_line = |line: &str| {
        // trim it
        let line = line.trim();
        // split on `->`; if line empty, this will be None.
        let (url, path) = line.split_once("->")?;
        // remove quotes
        let url = url.trim().trim_matches('"');
        let path = path.trim().trim_matches('"');
        // trim whitespace
        let url = url.trim();
        let path = path.trim();
        Some((url.to_string(), path.to_string()))
    };

    // cache file exists; parse it and return cache.
    // cache format has quoted strings split by `->`:
    // "url" -> "path"
    // still works if strings aren't quoted too.
    if let Ok(content) = read_file(&cache_file) {
        for line in content.lines() {
            if let Some((url, path)) = parse_line(line) {
                cache.insert(url, path);
            }
        }
    }

    cache
}

/// Writes the cache to disk.
/// See `read_cache` for the expected format of the cache file.
///
/// # Errors
/// Returns an error if we fail to write the cache to disk for any reason.
pub fn write_cache(cfg: &Cfg) -> Result<(), String> {
    let cache_file = &cfg.cache_file;
    let cache = &cfg.cache;

    let mut content = String::new();
    for (url, path) in cache {
        // each line is of the form "url" -> "path"
        content.push_str(&format!("\"{url}\" -> \"{path}\"\n"));
    }
    // write the content to disk
    write(cache_file, content).map_err(|e| {
        format!(
            "Error writing cache to {} on disk: {e}",
            cache_file.display()
        )
    })
}

impl Default for Cfg {
    /// The default configuration is as follows:
    /// - out_dir: ./terashite
    /// - cache: empty, with default cache file at XDG cache dir or ./cache if XDG cache dir is not available
    fn default() -> Self {
        // create the default cache file if it doesn't exist, and warn if we fail to create it
        let cache_file = default_cache_file();
        if !cache_file.is_file() {
            if let Err(e) = write(&cache_file, "") {
                eprintln!(
                    "Error creating default cache file at '{}': {e}. Continuing with empty cache.",
                    cache_file.display()
                );
            }
        }

        // then return the default config
        Self {
            out_dir: default_out_dir(),
            cache_file: default_cache_file(),
            cache: HashMap::new(),
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

/// Ensures that the output directories for puzzles exist.
///
/// # Errors
///
/// Returns an error if we fail to create any of the output directories for any reason. Note that if this function returns an error, we won't do any work at all, since we won't be able to save any puzzles to disk.
pub fn ensure_outs(cfg: &Cfg) -> Result<(), String> {
    scraper::names().iter().try_for_each(|name| {
        let dir = cfg.out_dir.join(name);
        std::fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Error creating output directory for domain '{name}' at '{}': {e}",
                dir.display()
            )
        })?;
        Ok(())
    })
}

/// Saves a puzzle to disk.
///
/// # Errors
/// Returns an error if we already have a file for this puzzle, or if there was an error writing the file to disk.
pub fn save_puzzle(puzzle: &model::Puzzle, cfg: &mut Cfg) -> Result<String, String> {
    // construct path to save puzzle to
    let fname = puzzle.filename();
    let path = cfg.out_dir.join(fname);

    // check if we already have the file
    if path.is_file() {
        return Err(format!("Some puzzle already saved at '{}'", path.display()));
    }

    // try to save the puzzle to disk
    let content = puzzle.content();
    write(&path, content)
        .map_err(|e| format!("Error writing puzzle to {} on disk: {e}", path.display()))?;

    // update cache
    let path = path.to_string_lossy().to_string();
    cfg.cache.insert(puzzle.from_url.clone(), path.clone());

    // then return path
    Ok(path)
}

// /// Checks if we have already saved a puzzle to disk.
// pub(crate) fn is_already_saved(puzzle: &model::Puzzle, cfg: &Cfg) -> bool {
//     let fname = puzzle.filename();
//     let path = cfg.out_dir.join(fname);
//     path.is_file()
// }
