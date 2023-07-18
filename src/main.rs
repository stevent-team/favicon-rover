use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Fetch the favicon from a specified url"
)]
struct Cli {
    /// Host to fetch the favicon for
    url: Option<String>,

    /// Square pixel size of the favicon
    #[arg(short, long)]
    size: Option<u16>,

    /// Path to save favicon to if not using stdout
    #[arg(short, long)]
    out: Option<String>,

    /// Image type to save favicon (overrides file extension if provided)
    #[arg(short, long, default_value_t = String::from("webp"))]
    r#type: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start a favicon scout web server
    Serve {
        /// Host to use for http server
        #[arg(long, default_value_t = String::from("localhost"), value_name = "URL")]
        host: String,

        /// Port to use for http server
        #[arg(short, long, default_value_t = 3000)]
        port: i16,

        /// URL or regex allowed by CORS
        #[arg(short, long, default_values_t = [String::from("*")])]
        origin: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    dbg!(cli);
}
