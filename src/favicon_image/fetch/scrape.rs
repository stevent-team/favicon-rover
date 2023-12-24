//! Methods for scraping a website to determine the available favicon urls

use reqwest::{header::USER_AGENT, Client};
use thiserror::Error;
use url::Url;

use super::BOT_USER_AGENT;

#[derive(Debug, Clone)]
struct Link {
    href: String,
    size: usize,
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

/// Scrape the <link /> tags from a given URL to find a favicon url
pub async fn scrape_link_tags(
    client: &Client,
    url: &Url,
    preferred_size: u32,
) -> Result<Url, ScrapeError> {
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
