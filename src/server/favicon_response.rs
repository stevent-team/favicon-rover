use crate::favicon_image::fetch::FetchFaviconError;
use crate::favicon_image::FaviconImage;
use axum::http::{header, HeaderMap, HeaderName};
use axum::response::IntoResponse;
use image::ImageFormat;

use super::fallback::generate_fallback;

#[derive(Debug)]
pub struct FaviconResponse {
    image: FaviconImage,
    headers: HeaderMap,
}

impl FaviconResponse {
    pub fn from_fetch_result(
        res_value: Result<FaviconImage, FetchFaviconError>,
        host: String,
        size: u32,
        format: ImageFormat,
    ) -> Self {
        // Construct response headers
        let mut headers = HeaderMap::new();
        headers.insert(header::CACHE_CONTROL, "max-age=604800".parse().unwrap());

        if let Err(error) = &res_value {
            headers.insert(
                HeaderName::from_static("x-fallback"),
                "true".parse().unwrap(),
            );
            headers.insert(
                HeaderName::from_static("x-fallback-reason"),
                error.to_string().parse().unwrap(),
            );
        }

        // Get image or fallback w/ correct size
        let mut image = match res_value {
            Ok(image) => image.resize(size),
            Err(_) => generate_fallback(host, size),
        };

        // Set desired format
        image = image.reformat(format);

        Self { image, headers }
    }
}

impl IntoResponse for FaviconResponse {
    fn into_response(self) -> axum::response::Response {
        (self.headers, self.image).into_response()
    }
}
