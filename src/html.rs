use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Mutex;

/// A wrapper around a reqwest client with `DDoS` protection.
///
/// Protection is simple: only allow 1 request per domain every 10 seconds.
pub struct HTMLDownloader<'a> {
    client: &'a reqwest::Client,
    request_times: Arc<Mutex<HashMap<String, Instant>>>,
}

impl<'a> From<&'a reqwest::Client> for HTMLDownloader<'a> {
    fn from(client: &'a reqwest::Client) -> Self {
        Self {
            client,
            request_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl HTMLDownloader<'_> {
    /// Extract the domain from a URL string.
    fn extract_domain(url: &str) -> Result<String, String> {
        // Remove the scheme
        let url_without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        // Get the part before the first '/' or ':'
        let domain = url_without_scheme
            .split('/')
            .next()
            .and_then(|part| part.split(':').next())
            .ok_or_else(|| format!("could not parse domain from '{url}'"))?;

        if domain.is_empty() {
            Err(format!("could not parse domain from '{url}'"))
        } else {
            Ok(domain.to_string())
        }
    }

    /// Gets the HTML for some URL by downloading it.
    /// Does not involve the cache in any way, and does not do any checks or parsing.
    ///
    /// Enforces rate limiting: maximum 1 request per domain every 10 seconds.
    /// If a request is attempted before the 10-second interval has passed, this function
    /// will sleep until the interval is satisfied.
    ///
    /// # Errors
    /// Returns an error if the URL cannot be fetched, if the status code is not success,
    /// or if the domain cannot be parsed.
    pub(crate) async fn download(&self, url: &str) -> Result<String, String> {
        eprintln!("[{}] Downloading '{url}'...", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        let domain = Self::extract_domain(url)?;

        // Rate limiting: ensure 10 seconds between requests per domain
        {
            let mut times = self.request_times.lock().await;
            if let Some(last_request) = times.get(&domain) {
                let elapsed = last_request.elapsed();
                if elapsed < Duration::from_secs(10) {
                    let wait_time = Duration::from_secs(10) - elapsed;
                    drop(times); // Release the lock before sleeping
                    eprintln!(
                        "Waiting for {} seconds before making request to domain '{domain}' to respect rate limits...",
                        wait_time.as_secs()
                    );
                    tokio::time::sleep(wait_time).await;
                    times = self.request_times.lock().await;
                }
            }
            times.insert(domain.clone(), Instant::now());
        }

        match self.client.get(url).send().await {
            Ok(res) => {
                let status = res.status();
                if status.is_success() {
                    let html = res
                        .text()
                        .await
                        .map_err(|e| format!("could not get text for '{url}' because {e}"))?;
                    Ok(html)
                } else {
                    Err(format!(
                        "fetching '{url}' not successful: {}",
                        status.as_str()
                    ))
                }
            }
            Err(e) => Err(format!("could not fetch '{url}' because {e}")),
        }
    }
}

#[must_use]
pub fn create_downloader(client: &reqwest::Client) -> HTMLDownloader<'_> {
    HTMLDownloader::from(client)
}

/// Creates a reqwest client to use in the project.
///
/// # Panics
/// Panics if the client cannot be built, which should never happen.
#[must_use]
pub fn create_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0")
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(2))
        .build()
        .expect("creating client")
}
