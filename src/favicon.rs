use image::imageops::FilterType;

use crate::get_favicon::GetFaviconError;

#[derive(Debug)]
pub struct Image {
    pub data: image::DynamicImage,
    pub format: Option<image::ImageFormat>,
}

#[derive(Debug)]
pub enum Favicon {
    Image(Image),
    Fallback(Image, GetFaviconError),
}

impl Favicon {
    pub fn image(&self) -> &Image {
        match self {
            Favicon::Image(image) => image,
            Favicon::Fallback(image, _) => image,
        }
    }

    pub fn format(&mut self, format: image::ImageFormat) {
        // "Formatting" is just changing the format we want
        self.set_image_format(format)
    }

    fn set_image_format(&mut self, format: image::ImageFormat) {
        match self {
            Self::Image(ref mut img) => {
                img.format = Some(format);
            }
            Self::Fallback(ref mut img, _) => {
                img.format = Some(format);
            }
        }
    }

    fn set_image_data(&mut self, data: image::DynamicImage) {
        match self {
            Self::Image(ref mut img) => {
                img.data = data;
            }
            Self::Fallback(ref mut img, _) => {
                img.data = data;
            }
        }
    }

    pub fn resize(&mut self, size: u32) {
        let image = self.image();
        let image = image.data.resize_to_fill(size, size, FilterType::Lanczos3);
        self.set_image_data(image);
    }
}
