//! Wrapper for image data in various formats
//! Implements file and network IO for favicon data

pub mod fetch;
mod svg;

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io;
use thiserror::Error;

const WEBP_QUALITY: f32 = 70.0;

#[derive(Debug)]
pub struct FaviconImage {
    pub data: image::DynamicImage,
    pub format: Option<image::ImageFormat>,
}

#[derive(Error, Debug)]
pub enum WriteImageError {
    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error("Unsupported image format")]
    UnsupportedImageFormat,

    #[error(transparent)]
    IOError(#[from] io::Error),
}

impl FaviconImage {
    pub fn write_to(
        &self,
        writer: &mut (impl io::Write + io::Seek),
        format: image::ImageFormat,
    ) -> Result<(), WriteImageError> {
        // Seperately handle output of webp
        if format == image::ImageFormat::WebP {
            return self.write_to_webp(writer);
        }

        // Convert image format to output format type
        let output_format: image::ImageOutputFormat = format
            .try_into()
            .map_err(|_| WriteImageError::UnsupportedImageFormat)?;

        // Write image
        self.data.write_to(writer, output_format)?;
        Ok(())
    }

    fn write_to_webp(
        &self,
        writer: &mut (impl io::Write + io::Seek),
    ) -> Result<(), WriteImageError> {
        // Ensure image data is in a format supported by `webp`
        let data = match self.data {
            DynamicImage::ImageRgba8(_) => &self.data,
            DynamicImage::ImageRgb8(_) => &self.data,
            _ => {
                let data = self.data.to_rgba8();
                &DynamicImage::ImageRgba8(data)
            }
        };

        // Write to webp
        let encoder =
            webp::Encoder::from_image(data).map_err(|_| WriteImageError::UnsupportedImageFormat)?;
        let webp = encoder.encode(WEBP_QUALITY);
        writer.write_all(webp.as_ref())?;

        Ok(())
    }

    pub fn resize(self, size: u32) -> Self {
        let data = self.data.resize_to_fill(size, size, FilterType::Lanczos3);
        Self { data, ..self }
    }

    pub fn reformat(self, format: ImageFormat) -> Self {
        Self {
            format: Some(format),
            ..self
        }
    }
}

#[cfg(feature = "server")]
mod server {
    use crate::DEFAULT_IMAGE_FORMAT;

    use super::*;
    use axum::response::IntoResponse;

    impl IntoResponse for FaviconImage {
        fn into_response(self) -> axum::response::Response {
            // Determine content type
            let format = self.format.unwrap_or(DEFAULT_IMAGE_FORMAT);
            let content_type = format.to_mime_type();

            // Write image to buffer
            let mut body = io::Cursor::new(Vec::new());
            self.write_to(&mut body, format).unwrap();

            ([("content-type", content_type)], body.into_inner()).into_response()
        }
    }

    trait ImageFormatContentTypeExt {
        fn content_type(&self) -> String;
    }
}
