use thiserror::Error;
use url::Url;

struct Favicon {
    image: bool,
    did_fallback: bool,
}

struct Link {
    href: String,
    size: Option<usize>,
}

#[derive(Error, Debug)]
enum ScrapeError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    HTMLParse(#[from] tl::ParseError),

    #[error(transparent)]
    URLParse(#[from] url::ParseError),

    #[error("link not found")]
    LinkNotFound,
}

async fn scrape_link_tags(url: Url) -> Result<Url, ScrapeError> {
    let res = reqwest::get(url).await?;
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

async fn get_favicon(url: Url, size: Option<usize>) -> Result<Favicon, ()> {
    let image_url = scrape_link_tags(url)
        .await
        .unwrap_or_else(|_| url.join("/favicon.ico").unwrap());

    // Fetch the image

    ()
}
