mod cli_args;
mod favicon_image;
mod image_writer;

#[cfg(feature = "server")]
mod server;

use std::io::Write;

use clap::Parser;
use cli_args::{Cli, Command};
use favicon_image::FaviconImage;
use image::ImageFormat;
use image_writer::ImageWriter;
use reqwest::Client;

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
            // Get favicon (will not gen a fallback)
            let fetch_size = size.unwrap_or(DEFAULT_IMAGE_SIZE);
            let client = Client::new();
            let mut favicon = match FaviconImage::fetch_for_url(&client, &url, fetch_size).await {
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
