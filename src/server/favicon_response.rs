use crate::fallback::generate_fallback;
use crate::favicon_image::FaviconImage;
use crate::get_favicon::GetFaviconError;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::IntoResponse;
use image::ImageFormat;

#[derive(Debug)]
pub struct FaviconResponse {
    image: FaviconImage,
    headers: HeaderMap,
}

impl FaviconResponse {
    pub fn from_fetch_result(
        res_value: Result<FaviconImage, GetFaviconError>,
        host: String,
        size: u32,
        format: ImageFormat,
    ) -> Self {
        // Construct response headers
        let headers = match &res_value {
            Ok(_) => Default::default(),
            Err(error) => [
                ("x-fallback", "true"),
                ("x-fallback-reason", &error.to_string()),
            ]
            .into_iter()
            .map(|(k, v)| {
                (
                    HeaderName::from_static(k),
                    HeaderValue::from_str(v).unwrap(),
                )
            })
            .collect(),
        };

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
