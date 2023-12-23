//! Svg operations for favicon images

use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use resvg::{
    tiny_skia,
    usvg::{self, fontdb, Options, Size, TreeParsing, TreeTextToPath},
    Tree,
};

// Load fonts once
lazy_static! {
    static ref FONT_DB: fontdb::Database = {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
        {
            dbg!("Loading from /usr/share/fonts");
            db.load_fonts_dir("/usr/share/fonts/");
        }

        db
    };
}

impl super::FaviconImage {
    /// Rasterise an svg string to a formatless favicon image
    pub fn from_svg_str(svg: String, size: u32) -> Self {
        dbg!("Showing fonts");
        for font in FONT_DB.faces() {
            dbg!(font);
        }

        let rtree = {
            let mut tree = usvg::Tree::from_data(svg.as_bytes(), &Options::default()).unwrap();
            tree.convert_text(&FONT_DB);
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
