mod cli;
mod get_favicon;

use std::io::{self, BufWriter, Write};

use clap::Parser;
use cli::{Cli, Command};
use get_favicon::get_favicon;

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
            let favicon = get_favicon(&url, size).await;

            // Can we guess the format from the "out" path?
            let format = format
                .map(
                    |f| image::ImageFormat::Png, /* TODO: convert f from cli::format to image::format */
                )
                .or_else(|| out.and_then(|path| image::ImageFormat::from_path(path).ok()));

            // Format the image
            if let Some(format) = format {
                // TODO: format the image and update the internal format
                // favicon.format(format);
            }

            // Determine output format
            // TODO: get from image itself, then fallback to PNG
            let out_format = image::ImageOutputFormat::Png;

            // Write favicon to `out` or stdout if not specified
            // TODO: figure out this mess, should prob just call to differente methods on
            // image.data if out is present or not. the DynamicImage struct has a method for
            // writing to a path which could help
            let image = favicon.image();
            let writer: Box<dyn io::Write + io::Seek> = if let Some(out) = out {
                Box::new(std::fs::File::open(out).unwrap()) // TODO: handle error
            } else {
                Box::new(io::stdout())
            };
            let writer = BufWriter::new(writer);
            let format = image.data.write_to(&mut writer, out_format);
        }
        Some(Command::Serve { .. }) => {
            // TODO
        }
        None => {}
    }
}
