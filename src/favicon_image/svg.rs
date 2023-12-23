//! Svg operations for favicon images

use image::{DynamicImage, RgbaImage};
use resvg::{
    tiny_skia,
    usvg::{self, fontdb, Options, Size, TreeParsing, TreeTextToPath},
    Tree,
};

impl super::FaviconImage {
    /// Rasterise an svg string to a formatless favicon image
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
