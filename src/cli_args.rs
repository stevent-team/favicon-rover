use std::path::PathBuf;

#[cfg(feature = "server")]
use clap::Args;

use clap::{Parser, Subcommand, ValueEnum};
use url::Url;

#[derive(Clone, ValueEnum, Debug)]
pub enum ImageFormatOutput {
    Png,
    Jpeg,
    Webp,
    Bmp,
    Ico,
    Gif,
    Tiff,
}

impl From<ImageFormatOutput> for image::ImageFormat {
    fn from(value: ImageFormatOutput) -> Self {
        match value {
            ImageFormatOutput::Png => image::ImageFormat::Png,
            ImageFormatOutput::Jpeg => image::ImageFormat::Jpeg,
            ImageFormatOutput::Webp => image::ImageFormat::WebP,
            ImageFormatOutput::Bmp => image::ImageFormat::Bmp,
            ImageFormatOutput::Ico => image::ImageFormat::Ico,
            ImageFormatOutput::Gif => image::ImageFormat::Gif,
            ImageFormatOutput::Tiff => image::ImageFormat::Tiff,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch the favicon for a specified url
    Get {
        /// Host to fetch the favicon for
        url: Url,

        /// Square pixel size of the favicon
        #[arg(short, long)]
        size: Option<u32>,

        /// Path to save favicon to if not using stdout
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Image format to save favicon as (overrides file extension if provided)
        #[arg(value_enum, short, long)]
        format: Option<ImageFormatOutput>,
    },

    /// Start a favicon rover web server
    #[cfg(feature = "server")]
    Serve(ServerOptions),
}

#[cfg(feature = "server")]
#[derive(Args, Debug)]
pub struct ServerOptions {
    /// Host to use for http server
    #[arg(long, default_value_t = String::from("127.0.0.1"), value_name = "URL")]
    pub host: String,

    /// Port to use for http server
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// URL or regex allowed by CORS (multiple allowed)
    #[arg(short, long, default_values_t = [String::from("*")])]
    pub origin: Vec<String>,
}
