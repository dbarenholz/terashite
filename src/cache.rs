use std::{
    fs::{File, OpenOptions, create_dir_all},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::{Path, PathBuf},
};

/// Create the terashite directory and subdirectories if they don't exist.
/// Returns an error if the directories cannot be created.
fn create_dirs_if_not_exists() -> Result<(), String> {
    let dir = get_terashite_dir()?;
    create_dir_all(&dir).map_err(|e| format!("could not init terashite dir: {e}"))?;
    create_dir_all(dir.join("html")).map_err(|e| format!("could not init html cache dir: {e}"))?;
    create_dir_all(dir.join("puzzles")).map_err(|e| format!("could not init puzzle dir: {e}"))?;
    Ok(())
}

/// Gets the terashite directory.
/// Returns an error if the home directory cannot be obtained.
fn get_terashite_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .ok_or_else(|| format!("could not obtain home directory"))
        .map(|mut path| {
            path.push(".terashite");
            path
        })
}

/// Gets the cache directory for HTML files.
/// Returns an error if the home directory cannot be obtained.
fn get_html_cache_dir() -> Result<PathBuf, String> {
    let mut dir = get_terashite_dir()?;
    dir.push("html");
    Ok(dir)
}

/// Gets the cache directory for puzzle files.
/// Returns an error if the home directory cannot be obtained.
fn get_puzzle_dir() -> Result<PathBuf, String> {
    let mut dir = get_terashite_dir()?;
    dir.push("puzzles");
    Ok(dir)
}

/// Get file path for a html file in the cache.
fn get_path_for(filename: String) -> Result<PathBuf, String> {
    create_dirs_if_not_exists()?;
    let html_cache = get_html_cache_dir()?;
    Ok(html_cache.join(filename))
}

/// Open a file in read only mode.
///
/// Returns an error when `OpenOptions::open()` does.
fn open_ro<P>(path: P) -> Result<File, String>
where
    P: AsRef<Path>,
{
    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| format!("could not open file as read-only: {e}"))
}

/// Open a file in rw mode. If the file doesn't exist, it will be created.
///
/// Returns an error when `OpenOptions::open()` does.
fn open_rw<P>(path: P) -> Result<File, String>
where
    P: AsRef<Path>,
{
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .map_err(|e| format!("could not open file as rw: {e}"))
}

/// Checks if a site is cached, and if so returns the file for it.
///
/// returns Ok(file) if it is, and the file is readable
/// returns Err(msg) if there's an error reading the cached file
pub(crate) fn try_get_file(url: &str) -> Result<File, String> {
    let filename = url_to_filename(url);
    let file = get_path_for(filename)?;
    open_ro(file)
}

/// Checks if a site is cached by checking if the corresponding file exists and is readable.
fn is_cached(url: &str) -> bool {
    try_get_file(url).is_ok()
}

/// Converts a URL to a filename for storing to disk.
///
/// The conversion is done by computing the default hash of the url, and returning it appended with `.html`.
fn url_to_filename(url: &str) -> String {
    let mut s = DefaultHasher::new();
    url.hash(&mut s);
    let hash_no = s.finish();
    hash_no.to_string() + ".html"
}

/// Saves the given text to a file corresponding to the given URL in the cache directory.
pub(crate) fn save_html(url: &str, txt: &str) -> Result<(), String> {
    let filename = url_to_filename(url);
    let filepath = get_path_for(filename)?;
    let mut file = open_rw(&filepath)?;
    file.write_all(txt.as_bytes())
        .map_err(|e| format!("could not file '{}' for '{url}': {e}", filepath.display()))
}

// TODO: implement saving puzzles
fn save_puzzle() {
    todo!()
}
