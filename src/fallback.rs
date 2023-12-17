use crate::favicon_image::FaviconImage;
use image::{DynamicImage, RgbaImage};
use resvg::{
    tiny_skia,
    usvg::{self, fontdb, Options, Size, TreeParsing, TreeTextToPath},
    Tree,
};

pub fn generate_fallback(name: String, size: u32) -> FaviconImage {
    let fallback_svg = format!(
        r##"
            <svg viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
                <rect width="100%" height="100%" fill="#666666" />
                <text x="50%" y="58%" font-family="sans-serif" font-size="200" fill="#FFFFFF" dominant-baseline="middle" text-anchor="middle">{}</text>
            </svg>
        "##,
        name.chars().next().unwrap_or('?').to_ascii_uppercase()
    );

    let rtree = {
        // TODO: include a font file in this project for consistent results
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        let mut tree = usvg::Tree::from_data(fallback_svg.as_bytes(), &Options::default()).unwrap();
        tree.convert_text(&fontdb);
        tree.size = tree
            .size
            .scale_to(Size::from_wh(size as f32, size as f32).unwrap());
        Tree::from_usvg(&tree)
    };

    let pixmap_size = rtree.size.to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    rtree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    FaviconImage {
        data: DynamicImage::ImageRgba8(
            RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec()).unwrap(),
        ),
        format: None,
    }
}
