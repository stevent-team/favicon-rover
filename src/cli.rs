use std::path::PathBuf;

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

    /// Start a favicon scout web server
    Serve {
        /// Host to use for http server
        #[arg(long, default_value_t = String::from("localhost"), value_name = "URL")]
        host: String,

        /// Port to use for http server
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// URL or regex allowed by CORS
        #[arg(short, long, default_values_t = [String::from("*")])]
        origin: Vec<String>,
    },
}
