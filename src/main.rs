mod cli;
mod get_favicon;

use clap::Parser;
use cli::{Cli, Command};

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
            let favicon = get_favicon(&url);
        }
        Some(serve @ Command::Serve { .. }) => {
            // TODO
        }
        None => {}
    }
}
