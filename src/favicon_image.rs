use image::{imageops::FilterType, ImageFormat};
use image::{DynamicImage, RgbaImage};
use resvg::{
    tiny_skia,
    usvg::{self, fontdb, Options, Size, TreeParsing, TreeTextToPath},
    Tree,
};
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

    pub fn from_svg_str(svg: String, size: u32) -> Self {
        let rtree = {
            // TODO: include a font file in this project for consistent results
            let mut fontdb = fontdb::Database::new();
            fontdb.load_system_fonts();

            let mut tree = usvg::Tree::from_data(svg.as_bytes(), &Options::default()).unwrap();
            tree.convert_text(&fontdb);
            tree.size = tree
                .size
                .scale_to(Size::from_wh(size as f32, size as f32).unwrap());
            Tree::from_usvg(&tree)
        };

        let pixmap_size = rtree.size.to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        rtree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

        Self {
            data: DynamicImage::ImageRgba8(
                RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                    .unwrap(),
            ),
            format: None,
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
