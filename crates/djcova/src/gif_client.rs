use async_trait::async_trait;
use rand::seq::SliceRandom;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TenorResponse {
    results: Vec<TenorResult>,
}

#[derive(Clone, Debug, Deserialize)]
struct TenorResult {
    media_formats: MediaFormats,
}

#[derive(Clone, Debug, Deserialize)]
struct MediaFormats {
    gif: GifMedia,
}

#[derive(Clone, Debug, Deserialize)]
struct GifMedia {
    url: String,
}

#[async_trait]
pub trait GifService: Send + Sync + std::fmt::Debug {
    async fn fetch_dancing_gif(&self) -> anyhow::Result<String>;
}

#[derive(Debug)]
pub struct TenorGifClient {
    client: Client,
    api_key: Option<String>,
}

impl TenorGifClient {
    pub fn new() -> Self {
        let api_key = std::env::var("TENOR_API_KEY").ok();
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
            api_key,
        }
    }
}

impl Default for TenorGifClient {
    fn default() -> Self {
        Self::new()
    }
}

const SEARCH_TERMS: &[&str] = &[
    "people dancing to music",
    "dance vibes meme",
    "music vibes dancing",
    "party dancing meme",
    "grooving to music",
    "dance party vibe",
    "bobbing head to music",
    "jamming to music meme",
    "dancing in car meme",
    "happy dancing meme",
];

#[async_trait]
impl GifService for TenorGifClient {
    async fn fetch_dancing_gif(&self) -> anyhow::Result<String> {
        let Some(ref key) = self.api_key else {
            anyhow::bail!("TENOR_API_KEY is not set");
        };

        let query = {
            let mut rng = rand::thread_rng();
            *SEARCH_TERMS
                .choose(&mut rng)
                .unwrap_or(&"people dancing to music")
        };

        let url = reqwest::Url::parse_with_params(
            "https://tenor.googleapis.com/v2/search",
            &[
                ("q", query),
                ("key", key.as_str()),
                ("limit", "20"),
                ("random", "true"),
            ],
        )?;

        let response: TenorResponse = self.client.get(url).send().await?.json().await?;

        let result = {
            let mut rng = rand::thread_rng();
            response.results.choose(&mut rng).cloned()
        };

        if let Some(res) = result {
            Ok(res.media_formats.gif.url)
        } else {
            anyhow::bail!("No GIF found in Tenor response")
        }
    }
}
