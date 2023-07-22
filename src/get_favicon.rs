use reqwest::header::{HeaderValue, CONTENT_TYPE};
use std::io;
use thiserror::Error;
use url::Url;

#[derive(Debug)]
pub enum Favicon {
    Image(image::DynamicImage),
    Fallback(image::DynamicImage, GetFaviconError),
}

#[derive(Debug)]
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
enum ScrapeError {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    HTMLParse(#[from] tl::ParseError),

    #[error(transparent)]
    URLParse(#[from] url::ParseError),

    #[error("link not found")]
    LinkNotFound,
}

pub async fn get_favicon(target_url: &Url) -> Favicon {
    match fetch_favicon(target_url).await {
        Ok(image) => image,
        Err(error) => Favicon::Fallback(generate_fallback(target_url).await, error),
    }
}

async fn generate_fallback(target_url: &Url) -> image::DynamicImage {
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
    let image = image_reader.decode()?;

    Ok(Favicon::Image(image))
}

enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Bmp,
    Ico,
    Gif,
    Tiff,
    Svg,
}

impl ImageFormat {
    // pub fn from_content_type(content_type: &HeaderValue) -> Option<Self> {
    //     match content_type.to_str().ok()? {
    //         "image/png" => Some(Self::Png),
    //         "image/jpeg" => Some(Self::Jpeg),
    //         "image/webp" => Some(Self::Webp),
    //         "image/bmp" => Some(Self::Bmp),
    //         "image/gif" => Some(Self::Gif),
    //         "image/tiff" => Some(Self::Tiff),
    //         "image/svg+xml" => Some(Self::Svg),
    //         "image/vnd.microsoft.icon" | "image/x-icon" => Some(Self::Ico),
    //         _ => None,
    //     }
    // }
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
