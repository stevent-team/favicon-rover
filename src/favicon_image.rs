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
        let encoder = webp::Encoder::from_image(&self.data).expect("Image format is supported");
        let webp = encoder.encode(WEBP_QUALITY);
        writer.write_all(webp.as_ref())?;
        Ok(())
    }
}

#[cfg(feature = "server")]
mod server {
    use super::*;
    use axum::response::IntoResponse;

    impl IntoResponse for FaviconImage {
        fn into_response(self) -> axum::response::Response {
            // Determine content type
            // TODO: use accept-content header to determine output format
            let format = self.format.unwrap_or(image::ImageFormat::Jpeg);
            let content_type = format.content_type();

            // Write image to buffer
            let mut body = io::Cursor::new(Vec::new());
            self.write_to(&mut body, format).unwrap();

            ([("content-type", content_type)], body.into_inner()).into_response()
        }
    }

    trait ImageFormatContentTypeExt {
        fn content_type(&self) -> String;
    }

    impl ImageFormatContentTypeExt for image::ImageFormat {
        fn content_type(&self) -> String {
            match self {
                image::ImageFormat::Png => "image/png",
                image::ImageFormat::Jpeg => "image/jpeg",
                image::ImageFormat::Gif => "image/gif",
                image::ImageFormat::WebP => "image/webp",
                image::ImageFormat::Tiff => "image/tiff",

                image::ImageFormat::Bmp => "image/bmp",
                image::ImageFormat::Ico => "image/x-icon",
                image::ImageFormat::Avif => "image/avif",

                image::ImageFormat::OpenExr => todo!(),
                image::ImageFormat::Farbfeld => todo!(),
                image::ImageFormat::Qoi => todo!(),
                image::ImageFormat::Pnm => todo!(),
                image::ImageFormat::Tga => todo!(),
                image::ImageFormat::Dds => todo!(),
                image::ImageFormat::Hdr => todo!(),
                _ => todo!(),
            }
            .into()
        }
    }
}
