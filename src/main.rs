mod cli_args;
mod fallback;
mod favicon_image;
mod get_favicon;
mod image_writer;

#[cfg(feature = "server")]
mod server;

use std::io::Write;

use clap::Parser;
use cli_args::{Cli, Command};
use get_favicon::fetch_favicon;
use image::ImageFormat;
use image_writer::ImageWriter;

pub const DEFAULT_IMAGE_SIZE: u32 = 256;
pub const DEFAULT_IMAGE_FORMAT: ImageFormat = ImageFormat::Jpeg;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Get {
            url,
            out,
            size,
            format,
        }) => {
            // Get favicon (may be a fallback)
            let mut favicon = match fetch_favicon(&url, size.unwrap_or(DEFAULT_IMAGE_SIZE)).await {
                Ok(favicon) => favicon,
                Err(err) => {
                    eprintln!("failed to fetch favicon: {}", err);
                    return;
                }
            };

            // Can we guess the format from the "out" path?
            let format: Option<image::ImageFormat> = format.map(|f| f.into()).or_else(|| {
                out.as_ref()
                    .and_then(|path| image::ImageFormat::from_path(path).ok())
            });

            // Resize the image
            if let Some(size) = size {
                favicon = favicon.resize(size);
            }

            // Format the image
            if let Some(format) = format {
                favicon = favicon.reformat(format);
            }

            // Write the image
            let mut writer = ImageWriter::new(out);
            writer.write_image(&favicon).unwrap();
            writer.flush().unwrap();
        }

        #[cfg(feature = "server")]
        Some(Command::Serve(options)) => {
            server::start_server(options).await.unwrap();
        }

        None => {}
    }
}
