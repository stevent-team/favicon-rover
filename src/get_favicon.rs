use std::io;
use thiserror::Error;
use url::Url;

use crate::favicon::{Favicon, Image};

const DEFAULT_IMAGE_SIZE: u32 = 256;

#[derive(Debug, Clone)]
struct Link {
    href: String,
    size: Option<usize>,
}

#[derive(Error, Debug)]
pub enum GetFaviconError {
    #[error(transparent)]
    Scrape(#[from] ScrapeError),

    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error("Failed to decode image: {0}")]
    ImageError(#[from] image::ImageError),
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

pub async fn get_favicon(target_url: &Url, size: Option<u32>) -> Favicon {
    match fetch_favicon(target_url).await {
        // We have an image from the target, resize if applicable and return
        Ok(mut image) => {
            if let Some(size) = size {
                image.resize(size);
            }
            image
        }

        // We didn't get an image, generate one
        Err(error) => Favicon::Fallback(
            Image {
                data: generate_fallback(target_url, size.unwrap_or(DEFAULT_IMAGE_SIZE)).await,
                format: None,
            },
            error,
        ),
    }
}

async fn generate_fallback(target_url: &Url, size: u32) -> image::DynamicImage {
    todo!()
}

/// Fetch the favicon for a given url
pub async fn fetch_favicon(target_url: &Url) -> Result<Favicon, GetFaviconError> {
    // Determine favicon url
    let image_url = scrape_link_tags(target_url)
        .await
        .unwrap_or_else(|_| target_url.join("/favicon.ico").unwrap());

    // Fetch the image
    let res = reqwest::get(image_url).await?;

    // Get HTTP response body
    let body = res.bytes().await?;
    let cursor = io::Cursor::new(body);

    // Create reader and attempt to guess image format
    let image_reader = image::io::Reader::new(cursor)
        .with_guessed_format()
        .expect("Cursor IO shouldn't fail");

    // Decode the image!
    // TODO: this is blocking, should it be in a tokio blocking_task?
    let image_format = image_reader.format(); // TODO: this being none might need to be an error
    let image_data = image_reader.decode()?;

    Ok(Favicon::Image(Image {
        data: image_data,
        format: image_format,
    }))
}

/// Scrape the <link /> tags from a given URL to find a favicon url
async fn scrape_link_tags(url: &Url) -> Result<Url, ScrapeError> {
    let res = reqwest::get(url.clone()).await?;
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
                        == *"prefers-color-scheme:dark"
                    {
                        return None;
                    }
                }
                Some(Link {
                    href: href.as_utf8_str().into_owned(),
                    size: attr.get("sizes").flatten().and_then(|sizes| {
                        sizes
                            .as_utf8_str()
                            .split_once('x')
                            .and_then(|(size, _)| size.parse().ok())
                    }),
                })
            }
            None => None,
        })
        .collect();

    if links.is_empty() {
        return Err(ScrapeError::LinkNotFound);
    }

    links.sort_unstable_by_key(|link| link.size);

    Ok(Url::parse(&links.get(0).unwrap().href)?)
}
