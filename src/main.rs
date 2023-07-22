mod cli;
mod get_favicon;

use clap::Parser;
use cli::{Cli, Command};
use get_favicon::get_favicon;
use url::Url;

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
            let favicon = get_favicon(&url).await;
            dbg!(favicon);
        }
        Some(serve @ Command::Serve { .. }) => {
            // TODO
        }
        None => {}
    }
}
