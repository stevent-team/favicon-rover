use reqwest::{
    header::{CONTENT_TYPE, USER_AGENT},
    Client,
};
use std::{io, sync::OnceLock};
use thiserror::Error;
use url::Url;

use crate::favicon_image::FaviconImage;

const BOT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/41.0.2228.0 Safari/537.36";

static REQWEST_CLIENT: OnceLock<Client> = OnceLock::new();

fn reqwest_client() -> &'static Client {
    REQWEST_CLIENT.get_or_init(|| Client::builder().build().unwrap())
}

#[derive(Debug, Clone)]
struct Link {
    href: String,
    size: usize,
}

#[derive(Error, Debug)]
pub enum GetFaviconError {
    #[error(transparent)]
    Scrape(#[from] ScrapeError),

    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    TokioError(#[from] tokio::task::JoinError),

    #[error("Failed to decode image: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Provided URL is not a valid url")]
    InvalidUrl,

    #[error("Cannot decode the image type")]
    CannotDecode,
}

#[derive(Error, Debug)]
pub enum ScrapeError {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    HTMLParse(#[from] tl::ParseError),

    #[error(transparent)]
    URLParse(#[from] url::ParseError),

    #[error("link not found")]
    LinkNotFound,
}

/// Fetch the favicon for a given url
pub async fn fetch_favicon(target_url: &Url, size: u32) -> Result<FaviconImage, GetFaviconError> {
    // Determine favicon url
    let image_url = scrape_link_tags(target_url, size)
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
        return Ok(FaviconImage::from_svg_str(svg, size));
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
                    .ok_or(GetFaviconError::CannotDecode)
                    .map(|webp| webp.to_image())
            }

            // Use image to decode other
            Some(_) => image_reader.decode().map_err(|e| e.into()),

            // We don't know the format
            None => Err(GetFaviconError::CannotDecode),
        }
    })
    .await??;

    Ok(FaviconImage {
        data: image_data,
        format: image_format,
    })
}

/// Scrape the <link /> tags from a given URL to find a favicon url
async fn scrape_link_tags(url: &Url, preferred_size: u32) -> Result<Url, ScrapeError> {
    let client = reqwest_client();
    let res = client
        .get(url.clone())
        .header(USER_AGENT, BOT_USER_AGENT)
        .send()
        .await?;
    let html = res.text().await?;

    let dom = tl::parse(&html, tl::ParserOptions::default())?;
    let parser = dom.parser();
    let mut links: Vec<_> = dom
        .query_selector("link[rel*=\"icon\"]")
        .unwrap()
        .map(|link| link.get(parser).unwrap().as_tag().unwrap().attributes())
        .filter_map(|attr| match attr.get("href").flatten() {
            Some(href) => {
                if let Some(media) = attr.get("media").flatten() {
                    if String::from(media.as_utf8_str())
                        .replace(' ', "")
                        .to_ascii_lowercase()
                        .contains("prefers-color-scheme:dark")
                    {
                        return None;
                    }
                }
                Some(Link {
                    href: href.as_utf8_str().into_owned(),
                    size: attr
                        .get("sizes")
                        .flatten()
                        .and_then(|sizes| {
                            sizes
                                .as_utf8_str()
                                .split_once('x')
                                .and_then(|(size, _)| size.parse().ok())
                        })
                        .unwrap_or(0),
                })
            }
            None => None,
        })
        .collect();

    if links.is_empty() {
        return Err(ScrapeError::LinkNotFound);
    }

    links.sort_unstable_by_key(|link| link.size);

    // If an icon larger than the preferred size exists, use the closest
    // to what we want instead of always using the largest image available
    let filtered_links: Vec<_> = links
        .iter()
        .filter(|link| link.size < preferred_size as usize)
        .collect();
    if !filtered_links.is_empty() {
        return Ok(url.join(&filtered_links.first().unwrap().href)?);
    }

    Ok(url.join(&links.last().unwrap().href)?)
}
