use crate::favicon_image::FaviconImage;

const FALLBACK_FONT_FAMILY: &str = "arial";

pub fn generate_fallback(name: String, size: u32) -> FaviconImage {
    let fallback_svg = format!(
        r##"
            <svg viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
                <rect width="100%" height="100%" fill="#666666" />
                <text x="50%" y="58%" font-family="{FALLBACK_FONT_FAMILY}" font-size="200" fill="#FFFFFF" dominant-baseline="middle" text-anchor="middle">{}</text>
            </svg>
        "##,
        name.chars().next().unwrap_or('?').to_ascii_uppercase()
    );

    FaviconImage::from_svg_str(fallback_svg, size)
}
