use std::{collections::HashMap, io::Read, time::Duration};

use reqwest::{Client, redirect::Policy};
use scraper::Html;
use tokio::{sync::Mutex, time::sleep};

use crate::cache;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) enum Domain {
    Tibisukemaru,
    BachelorSeal,
}

pub struct HTMLDownloader {
    client: Client,
    download_lock: Mutex<()>,
}

impl HTMLDownloader {
    pub(crate) fn new() -> Self {
        let user_agent_str =
            "Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0";
        let client = reqwest::Client::builder()
            .user_agent(user_agent_str)
            .redirect(Policy::none())
            .build()
            // .map_err(|e| format!("could not create client: {e}"))
            .expect("creating client");

        Self {
            client,
            download_lock: Mutex::new(()),
        }
    }

    pub(crate) async fn fetch_url(&self, url: &str) -> Result<Html, String> {
        if let Ok(mut f) = cache::try_get_file(url) {
            let mut txt = String::new();
            f.read_to_string(&mut txt)
                .map_err(|e| format!("could not read file '{f:?}': {e}"))?;
            return Ok(scraper::Html::parse_document(&txt));
        }

        // Not cached, go download; sleep afterwards for 10s within mutex to keep server happy
        let _guard = self.download_lock.lock().await;
        let txt = self.download_url(url).await?;
        cache::save_html(url, &txt)?;
        sleep(Duration::from_secs(10)).await;
        Ok(scraper::Html::parse_document(&txt))
    }

    async fn download_url(&self, url: &str) -> Result<String, String> {
        let result = match self.client.get(url).send().await {
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
        };
        result
    }
}
