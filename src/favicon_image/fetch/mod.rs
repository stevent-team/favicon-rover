//! Methods for fetching a favicon image from a url and interpreting its format

mod scrape;

use reqwest::{
    header::{CONTENT_TYPE, USER_AGENT},
    Client,
};
use std::{io, sync::OnceLock};
use thiserror::Error;
use url::Url;

use scrape::{scrape_link_tags, ScrapeError};
pub const BOT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/41.0.2228.0 Safari/537.36";

// TODO: instead store a pool of clients on the axum state and pass a ref into the fetch() method
static REQWEST_CLIENT: OnceLock<Client> = OnceLock::new();
fn reqwest_client() -> &'static Client {
    REQWEST_CLIENT.get_or_init(|| Client::builder().build().unwrap())
}

#[derive(Error, Debug)]
pub enum FetchFaviconError {
    #[error(transparent)]
    Scrape(#[from] ScrapeError),

    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    TokioError(#[from] tokio::task::JoinError),

    #[error("Failed to decode image: {0}")]
    ImageError(#[from] image::ImageError),

    #[cfg(feature = "server")]
    #[error("Provided URL is not a valid url")]
    InvalidUrl,

    #[error("Cannot decode the image type")]
    CannotDecode,
}

/// Fetch the favicon for a given url
impl super::FaviconImage {
    pub async fn fetch_for_url(target_url: &Url, size: u32) -> Result<Self, FetchFaviconError> {
        // Determine favicon url
        let image_url = scrape_link_tags(reqwest_client(), target_url, size)
            .await
            .unwrap_or_else(|_| target_url.join("/favicon.ico").unwrap());

        // Fetch the image
        let client = reqwest_client();
        let res = client
            .get(image_url)
            .header(USER_AGENT, BOT_USER_AGENT)
            .send()
            .await?;

        // Render SVGs
        if res
            .headers()
            .get(CONTENT_TYPE)
            .is_some_and(|content_type| content_type == "image/svg+xml")
        {
            let svg = res.text().await?;
            return Ok(Self::from_svg_str(svg, size));
        }

        // Get HTTP response body
        let body = res.bytes().await?;
        let cursor = io::Cursor::new(body);

        // Create reader and attempt to guess image format
        let image_reader = image::io::Reader::new(cursor)
            .with_guessed_format()
            .expect("Cursor IO shouldn't fail");

        // Decode the image!
        let image_format = image_reader.format();
        let image_data = tokio::task::spawn_blocking(move || {
            match image_format {
                // Use `webp` crate to decode WebPs
                Some(image::ImageFormat::WebP) => {
                    let data = image_reader.into_inner().into_inner();
                    let decoder = webp::Decoder::new(&data);
                    decoder
                        .decode()
                        .ok_or(FetchFaviconError::CannotDecode)
                        .map(|webp| webp.to_image())
                }

                // Use image to decode other
                Some(_) => image_reader.decode().map_err(|e| e.into()),

                // We don't know the format
                None => Err(FetchFaviconError::CannotDecode),
            }
        })
        .await??;

        Ok(Self {
            data: image_data,
            format: image_format,
        })
    }
}
