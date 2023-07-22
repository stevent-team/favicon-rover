use std::fs;
use std::io::{self, BufWriter};
use std::path::PathBuf;

use crate::favicon_image::{FaviconImage, WriteImageError};

pub enum ImageWriter {
    ToFile(BufWriter<fs::File>),
    ToStdout(io::Cursor<Vec<u8>>),
}

impl ImageWriter {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        match file_path {
            Some(path) => Self::ToFile(BufWriter::new(fs::File::create(path).unwrap())),
            None => Self::ToStdout(io::Cursor::new(Vec::new())),
        }
    }

    pub fn write_image(&mut self, image: &FaviconImage) -> Result<(), WriteImageError> {
        let format = image.format.unwrap_or(image::ImageFormat::Png);
        image.write_to(self, format)
    }
}

impl io::Write for ImageWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            ImageWriter::ToFile(writer) => writer.write(buf),
            ImageWriter::ToStdout(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            ImageWriter::ToFile(writer) => writer.flush(),
            ImageWriter::ToStdout(writer) => writer
                .flush()
                .and_then(|_| io::stdout().write_all(writer.get_ref())),
        }
    }
}

impl io::Seek for ImageWriter {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self {
            ImageWriter::ToFile(writer) => writer.seek(pos),
            ImageWriter::ToStdout(writer) => writer.seek(pos),
        }
    }
}
